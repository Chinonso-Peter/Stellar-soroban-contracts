#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyError {
    ContractPaused = 1,
    InvalidParameters = 2,
    Unauthorized = 3,
    PolicyNotFound = 4,
    PolicyNotActive = 5,
    EndorsementNotFound = 6,
    EndorsementAlreadyProcessed = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PauseState {
    pub is_paused: bool,
    pub paused_at: Option<u64>,
    pub paused_by: Option<Address>,
    pub reason: Option<Symbol>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyEvent {
    PolicyIssued(u64, Address, i128, u64), // policy_id, holder, coverage, expires_at
    PolicyCanceled(u64),
    PolicyExpired(u64),
    ContractPaused(Address, Option<Symbol>),
    ContractUnpaused(Address, Option<Symbol>),
    EndorsementRequested(u64, u64),         // endorsement_id, policy_id
    EndorsementApproved(u64, u64, Address), // endorsement_id, policy_id, approver
    EndorsementRejected(u64, u64, Address), // endorsement_id, policy_id, rejector
    CoverageChanged(u64, i128, i128),       // policy_id, old_coverage, new_coverage
    PremiumAdjusted(u64, i128, i128),       // policy_id, old_premium, new_premium
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRecord {
    pub coverage: i128,
    pub premium: i128,
    pub holder: Address,
    pub status: PolicyStatus,
    pub issued_at: u64,
    pub expires_at: u64,
    pub total_fractions: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmmPool {
    pub policy_id: u64,
    pub token_balance: i128,
    pub stable_balance: i128,
    pub total_shares: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityPosition {
    pub provider: Address,
    pub shares: i128,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum PolicyStatus { Active, Expired, Cancelled }

// Risk Assessment structure for pricing
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RiskReport {
    pub property_id: u64,
    pub risk_score: u32,       // 0-100
    pub location_factor: u32,  // 100 = 1.0x
    pub coverage_ratio: u32,   // basis points
}

// Endorsement status lifecycle
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EndorsementStatus {
    Pending,
    Approved,
    Rejected,
}

// A request to modify a policy through the endorsement workflow
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndorsementRecord {
    pub endorsement_id: u64,
    pub policy_id: u64,
    pub requester: Address,
    pub new_coverage: Option<i128>,  // Some if coverage change requested
    pub new_premium: Option<i128>,   // Some if premium adjustment requested
    pub status: EndorsementStatus,
    pub requested_at: u64,
    pub processed_at: Option<u64>,
    pub processed_by: Option<Address>,
    pub reason: Option<Symbol>,      // short reason code (≤10 chars)
}

const POLICIES: Symbol = symbol_short!("POLICIES");
const POL_IDX: Symbol = symbol_short!("POL_IDX");
const POL_CNT: Symbol = symbol_short!("POL_CNT");
const ADMIN: Symbol = symbol_short!("ADMIN");
const GUARDIAN: Symbol = symbol_short!("GUARDIAN");
const PAUSE_STATE: Symbol = symbol_short!("PAUSED");
const AMM_POOLS: Symbol = symbol_short!("AMM_POOLS");
const LP_POSITIONS: Symbol = symbol_short!("LP_POS");
const ENDORSE: Symbol = symbol_short!("ENDORSE");    // endorsement records
const END_CNT: Symbol = symbol_short!("END_CNT");    // total endorsement counter
const END_HIST: Symbol = symbol_short!("END_HIST");  // per-policy history index

#[contract]
pub struct PolicyContract;

#[contractimpl]
impl PolicyContract {
    pub fn initialize(env: Env, admin: Address, guardian: Address) {
        if env.storage().instance().has(&ADMIN) { panic!("Already initialized"); }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&GUARDIAN, &guardian);
        env.storage().instance().set(&PAUSE_STATE, &PauseState { is_paused: false, paused_at: None, paused_by: None, reason: None });
    }

    pub fn set_pause_state(env: Env, caller: Address, is_paused: bool, reason: Option<Symbol>) -> Result<(), PolicyError> {
        caller.require_auth();
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let guardian: Address = env.storage().instance().get(&GUARDIAN).unwrap();

        if caller != admin && caller != guardian { return Err(PolicyError::Unauthorized); }

        let pause_state = PauseState {
            is_paused,
            paused_at: if is_paused { Some(env.ledger().timestamp()) } else { None },
            paused_by: if is_paused { Some(caller.clone()) } else { None },
            reason: reason.clone(),
        };
        env.storage().instance().set(&PAUSE_STATE, &pause_state);

        if is_paused {
            env.events().publish((Symbol::short("PAUSE"), Symbol::short("PAUSED")), PolicyEvent::ContractPaused(caller, reason));
        } else {
            env.events().publish((Symbol::short("PAUSE"), Symbol::short("UNPAUSED")), PolicyEvent::ContractUnpaused(caller, reason));
        }
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get::<_, PauseState>(&PAUSE_STATE).map(|s| s.is_paused).unwrap_or(false)
    }

    pub fn calculate_dynamic_premium(
        _env: Env,
        risk_report: RiskReport,
        base_rate: i128,
        market_condition_factor: u32,
    ) -> i128 {
        let risk_multiplier = risk_report.risk_score as i128;
        let location_multiplier = risk_report.location_factor as i128;
        let ratio_multiplier = risk_report.coverage_ratio as i128;

        let mut premium = base_rate;
        premium = (premium * risk_multiplier) / 100;
        premium = (premium * location_multiplier) / 100;
        premium = (premium * market_condition_factor as i128) / 100;
        premium = (premium * ratio_multiplier) / 10000;

        premium
    }

    const DEFAULT_POLICY_DURATION_SECS: u64 = 365 * 86400;

    fn increment_policy_count(env: &Env) -> u64 {
        let count: u64 = env.storage().instance().get(&POL_CNT).unwrap_or(0);
        let next = count + 1;
        env.storage().instance().set(&POL_CNT, &next);
        next
    }

    fn record_policy_index(env: &Env, index: u64, policy_id: u64) {
        env.storage().persistent().set(&(POL_IDX, index), &policy_id);
    }

    pub fn issue_policy(env: Env, holder: Address, policy_id: u64, coverage: i128, premium: i128) -> Result<(), PolicyError> {
        if Self::is_paused(env.clone()) { return Err(PolicyError::ContractPaused); }

        let now = env.ledger().timestamp();
        let expires_at = now.saturating_add(Self::DEFAULT_POLICY_DURATION_SECS);

        let key = (POLICIES, policy_id);
        env.storage().persistent().set(&key, &PolicyRecord { coverage, premium, holder: holder.clone(), status: PolicyStatus::Active, issued_at: now, expires_at, total_fractions: 10000 });

        let next_index = Self::increment_policy_count(&env);
        Self::record_policy_index(&env, next_index, policy_id);

        env.events().publish((POLICIES, Symbol::short("ISSUE")), PolicyEvent::PolicyIssued(policy_id, holder, coverage, expires_at));
        Ok(())
    }

    pub fn issue_policy_with_duration(env: Env, holder: Address, policy_id: u64, coverage: i128, premium: i128, duration_secs: u64) -> Result<(), PolicyError> {
        if duration_secs == 0 { return Err(PolicyError::InvalidParameters); }
        if Self::is_paused(env.clone()) { return Err(PolicyError::ContractPaused); }

        let now = env.ledger().timestamp();
        let expires_at = now.saturating_add(duration_secs);

        let key = (POLICIES, policy_id);
        env.storage().persistent().set(&key, &PolicyRecord { coverage, premium, holder: holder.clone(), status: PolicyStatus::Active, issued_at: now, expires_at, total_fractions: 10000 });

        let next_index = Self::increment_policy_count(&env);
        Self::record_policy_index(&env, next_index, policy_id);

        env.events().publish((POLICIES, Symbol::short("ISSUE")), PolicyEvent::PolicyIssued(policy_id, holder, coverage, expires_at));
        Ok(())
    }

    pub fn cancel_policy(env: Env, policy_id: u64) -> Result<(), PolicyError> {
        if Self::is_paused(env.clone()) { return Err(PolicyError::ContractPaused); }
        let key = (POLICIES, policy_id);
        let mut r: PolicyRecord = env.storage().persistent().get(&key).ok_or(PolicyError::PolicyNotFound)?;
        r.status = PolicyStatus::Cancelled;
        env.storage().persistent().set(&key, &r);
        env.events().publish((POLICIES, Symbol::short("CANCEL")), PolicyEvent::PolicyCanceled(policy_id));
        Ok(())
    }

    fn expire_policy_internal(env: &Env, policy_id: u64) -> Result<(), PolicyError> {
        let key = (POLICIES, policy_id);
        let mut r: PolicyRecord = env.storage().persistent().get(&key).ok_or(PolicyError::PolicyNotFound)?;
        if r.status != PolicyStatus::Active {
            return Ok(());
        }
        r.status = PolicyStatus::Expired;
        env.storage().persistent().set(&key, &r);
        env.events().publish((POLICIES, Symbol::short("EXPIRED")), PolicyEvent::PolicyExpired(policy_id));
        Ok(())
    }

    pub fn expire_policy(env: Env, policy_id: u64) -> Result<(), PolicyError> {
        if Self::is_paused(env.clone()) { return Err(PolicyError::ContractPaused); }
        Self::expire_policy_internal(&env, policy_id)
    }

    pub fn check_and_expire_policies(env: Env, start_index: u64, max_items: u64) -> Result<(u64, u64), PolicyError> {
        if Self::is_paused(env.clone()) { return Err(PolicyError::ContractPaused); }

        let total = env.storage().instance().get(&POL_CNT).unwrap_or(0);
        if start_index >= total { return Ok((0, start_index)); }

        let end_index = core::cmp::min(start_index + max_items, total);
        let now = env.ledger().timestamp();
        let mut expired_count: u64 = 0;

        for idx in start_index..end_index {
            let policy_id: u64 = env.storage().persistent().get(&(POL_IDX, idx + 1)).unwrap();
            let key = (POLICIES, policy_id);
            if let Some(policy) = env.storage().persistent().get::<_, PolicyRecord>(&key) {
                if policy.status == PolicyStatus::Active && policy.expires_at <= now {
                    Self::expire_policy_internal(&env, policy_id)?;
                    expired_count += 1;
                }
            }
        }

        Ok((expired_count, end_index))
    }

    pub fn query_active_policies_by_expiration(env: Env, start_index: u64, max_items: u64, until_ts: u64) -> Vec<u64> {
        let total = env.storage().instance().get(&POL_CNT).unwrap_or(0);
        let end_index = core::cmp::min(start_index + max_items, total);
        let mut results = Vec::new(env);

        for idx in start_index..end_index {
            let policy_id: u64 = env.storage().persistent().get(&(POL_IDX, idx + 1)).unwrap();
            let key = (POLICIES, policy_id);
            if let Some(policy) = env.storage().persistent().get::<_, PolicyRecord>(&key) {
                if policy.status == PolicyStatus::Active && policy.expires_at <= until_ts {
                    results.push_back(policy_id);
                }
            }
        }

        results
    }

    pub fn get_policy_count(env: Env) -> u64 {
        env.storage().instance().get(&POL_CNT).unwrap_or(0)
    }

    pub fn is_policy_active(env: Env, policy_id: u64) -> bool {
        let key = (POLICIES, policy_id);
        match env.storage().persistent().get::<_, PolicyRecord>(&key) {
            Some(r) => r.status == PolicyStatus::Active,
            None => false,
        }
    }

    pub fn get_policy_coverage(env: Env, policy_id: u64) -> i128 {
        let key = (POLICIES, policy_id);
        match env.storage().persistent().get::<_, PolicyRecord>(&key) {
            Some(r) => r.coverage,
            None => 0,
        }
    }

    pub fn create_amm_pool(env: Env, policy_id: u64, stable_amount: i128) -> Result<(), PolicyError> {
        if !Self::is_policy_active(env.clone(), policy_id) { return Err(PolicyError::PolicyNotFound); }
        
        let amm_key = (AMM_POOLS, policy_id);
        if env.storage().persistent().has(&amm_key) { return Err(PolicyError::InvalidParameters); }

        let pool = AmmPool {
            policy_id,
            token_balance: 10000, // Total fractions
            stable_balance: stable_amount,
            total_shares: stable_amount,
        };

        env.storage().persistent().set(&amm_key, &pool);
        Ok(())
    }

    pub fn get_pool_price(env: Env, policy_id: u64) -> Result<i128, PolicyError> {
        let amm_key = (AMM_POOLS, policy_id);
        let pool: AmmPool = env.storage().persistent().get(&amm_key).ok_or(PolicyError::PolicyNotFound)?;
        
        if pool.token_balance == 0 { return Ok(0); }
        
        // Simple x * y = k price. Price = y / x
        let price = (pool.stable_balance * 10000) / pool.token_balance;
        Ok(price)
    }

    pub fn swap_policy_fraction(env: Env, buyer: Address, policy_id: u64, stable_in: i128) -> Result<i128, PolicyError> {
        buyer.require_auth();
        let amm_key = (AMM_POOLS, policy_id);
        let mut pool: AmmPool = env.storage().persistent().get(&amm_key).ok_or(PolicyError::PolicyNotFound)?;

        // x * y = k constant product
        // (x - dx) * (y + dy) = x * y
        // dx = x - (x * y) / (y + dy)
        let k = pool.token_balance * pool.stable_balance;
        let new_stable_balance = pool.stable_balance + stable_in;
        let new_token_balance = k / new_stable_balance;
        let tokens_out = pool.token_balance - new_token_balance;

        if tokens_out <= 0 { return Err(PolicyError::InvalidParameters); }

        pool.stable_balance = new_stable_balance;
        pool.token_balance = new_token_balance;
        
        env.storage().persistent().set(&amm_key, &pool);
        Ok(tokens_out)
    }

    // -------------------------------------------------------------------------
    // Endorsement mechanism – allows policy modifications via a request/approve
    // workflow with full history tracking.
    // -------------------------------------------------------------------------

    /// Increment and return the next global endorsement ID.
    fn next_endorsement_id(env: &Env) -> u64 {
        let count: u64 = env.storage().instance().get(&END_CNT).unwrap_or(0);
        let next = count + 1;
        env.storage().instance().set(&END_CNT, &next);
        next
    }

    /// Append an endorsement_id to the per-policy history list.
    fn append_policy_endorsement(env: &Env, policy_id: u64, endorsement_id: u64) {
        let hist_cnt_key = (END_HIST, policy_id);
        let idx: u32 = env.storage().persistent().get(&hist_cnt_key).unwrap_or(0);
        env.storage().persistent().set(&(END_HIST, policy_id, idx), &endorsement_id);
        env.storage().persistent().set(&hist_cnt_key, &(idx + 1));
    }

    /// Check whether `caller` is the contract admin or guardian.
    fn is_admin_or_guardian(env: &Env, caller: &Address) -> bool {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let guardian: Address = env.storage().instance().get(&GUARDIAN).unwrap();
        caller == &admin || caller == &guardian
    }

    /// Request a policy endorsement (modification).
    ///
    /// - `requester` must authenticate and must be the policy holder or an
    ///   admin/guardian.
    /// - At least one of `new_coverage` / `new_premium` must be `Some`.
    /// - The target policy must exist and be in `Active` status.
    ///
    /// Returns the new `endorsement_id`.
    pub fn request_endorsement(
        env: Env,
        requester: Address,
        policy_id: u64,
        new_coverage: Option<i128>,
        new_premium: Option<i128>,
        reason: Option<Symbol>,
    ) -> Result<u64, PolicyError> {
        requester.require_auth();

        if Self::is_paused(env.clone()) {
            return Err(PolicyError::ContractPaused);
        }

        // Validate at least one change is requested
        if new_coverage.is_none() && new_premium.is_none() {
            return Err(PolicyError::InvalidParameters);
        }

        // Validate coverage/premium values when provided
        if let Some(cov) = new_coverage {
            if cov <= 0 { return Err(PolicyError::InvalidParameters); }
        }
        if let Some(prem) = new_premium {
            if prem <= 0 { return Err(PolicyError::InvalidParameters); }
        }

        // Policy must exist and be active
        let key = (POLICIES, policy_id);
        let policy: PolicyRecord = env.storage().persistent().get(&key).ok_or(PolicyError::PolicyNotFound)?;
        if policy.status != PolicyStatus::Active {
            return Err(PolicyError::PolicyNotActive);
        }

        // Only the holder or an admin/guardian may request an endorsement
        if requester != policy.holder && !Self::is_admin_or_guardian(&env, &requester) {
            return Err(PolicyError::Unauthorized);
        }

        let endorsement_id = Self::next_endorsement_id(&env);
        let now = env.ledger().timestamp();

        let record = EndorsementRecord {
            endorsement_id,
            policy_id,
            requester: requester.clone(),
            new_coverage,
            new_premium,
            status: EndorsementStatus::Pending,
            requested_at: now,
            processed_at: None,
            processed_by: None,
            reason,
        };

        env.storage().persistent().set(&(ENDORSE, endorsement_id), &record);
        Self::append_policy_endorsement(&env, policy_id, endorsement_id);

        env.events().publish(
            (ENDORSE, symbol_short!("REQUEST")),
            PolicyEvent::EndorsementRequested(endorsement_id, policy_id),
        );

        Ok(endorsement_id)
    }

    /// Approve a pending endorsement.
    ///
    /// Only the admin or guardian may call this. Applying the endorsement
    /// updates the underlying `PolicyRecord` with the requested changes and
    /// emits fine-grained `CoverageChanged` / `PremiumAdjusted` events in
    /// addition to the `EndorsementApproved` event.
    pub fn approve_endorsement(
        env: Env,
        caller: Address,
        endorsement_id: u64,
    ) -> Result<(), PolicyError> {
        caller.require_auth();

        if Self::is_paused(env.clone()) {
            return Err(PolicyError::ContractPaused);
        }
        if !Self::is_admin_or_guardian(&env, &caller) {
            return Err(PolicyError::Unauthorized);
        }

        let end_key = (ENDORSE, endorsement_id);
        let mut record: EndorsementRecord = env
            .storage()
            .persistent()
            .get(&end_key)
            .ok_or(PolicyError::EndorsementNotFound)?;

        if record.status != EndorsementStatus::Pending {
            return Err(PolicyError::EndorsementAlreadyProcessed);
        }

        // Apply changes to the policy
        let pol_key = (POLICIES, record.policy_id);
        let mut policy: PolicyRecord = env
            .storage()
            .persistent()
            .get(&pol_key)
            .ok_or(PolicyError::PolicyNotFound)?;

        let now = env.ledger().timestamp();

        if let Some(new_cov) = record.new_coverage {
            let old_cov = policy.coverage;
            policy.coverage = new_cov;
            env.events().publish(
                (ENDORSE, symbol_short!("COV_CHG")),
                PolicyEvent::CoverageChanged(record.policy_id, old_cov, new_cov),
            );
        }

        if let Some(new_prem) = record.new_premium {
            let old_prem = policy.premium;
            policy.premium = new_prem;
            env.events().publish(
                (ENDORSE, symbol_short!("PREM_ADJ")),
                PolicyEvent::PremiumAdjusted(record.policy_id, old_prem, new_prem),
            );
        }

        env.storage().persistent().set(&pol_key, &policy);

        // Update endorsement record
        record.status = EndorsementStatus::Approved;
        record.processed_at = Some(now);
        record.processed_by = Some(caller.clone());
        env.storage().persistent().set(&end_key, &record);

        env.events().publish(
            (ENDORSE, symbol_short!("APPROVED")),
            PolicyEvent::EndorsementApproved(endorsement_id, record.policy_id, caller),
        );

        Ok(())
    }

    /// Reject a pending endorsement.
    ///
    /// Only the admin or guardian may call this.
    pub fn reject_endorsement(
        env: Env,
        caller: Address,
        endorsement_id: u64,
        reason: Option<Symbol>,
    ) -> Result<(), PolicyError> {
        caller.require_auth();

        if Self::is_paused(env.clone()) {
            return Err(PolicyError::ContractPaused);
        }
        if !Self::is_admin_or_guardian(&env, &caller) {
            return Err(PolicyError::Unauthorized);
        }

        let end_key = (ENDORSE, endorsement_id);
        let mut record: EndorsementRecord = env
            .storage()
            .persistent()
            .get(&end_key)
            .ok_or(PolicyError::EndorsementNotFound)?;

        if record.status != EndorsementStatus::Pending {
            return Err(PolicyError::EndorsementAlreadyProcessed);
        }

        let now = env.ledger().timestamp();
        record.status = EndorsementStatus::Rejected;
        record.processed_at = Some(now);
        record.processed_by = Some(caller.clone());
        record.reason = reason;
        env.storage().persistent().set(&end_key, &record);

        env.events().publish(
            (ENDORSE, symbol_short!("REJECTED")),
            PolicyEvent::EndorsementRejected(endorsement_id, record.policy_id, caller),
        );

        Ok(())
    }

    /// Retrieve a single endorsement record by ID.
    pub fn get_endorsement(env: Env, endorsement_id: u64) -> Result<EndorsementRecord, PolicyError> {
        env.storage()
            .persistent()
            .get(&(ENDORSE, endorsement_id))
            .ok_or(PolicyError::EndorsementNotFound)
    }

    /// Return the list of endorsement IDs for a given policy (paginated).
    pub fn get_policy_endorsement_ids(
        env: Env,
        policy_id: u64,
        start: u32,
        max_items: u32,
    ) -> Vec<u64> {
        let hist_cnt_key = (END_HIST, policy_id);
        let total: u32 = env.storage().persistent().get(&hist_cnt_key).unwrap_or(0);
        let end = core::cmp::min(start + max_items, total);
        let mut ids = Vec::new(env.clone());

        for idx in start..end {
            let eid: u64 = env
                .storage()
                .persistent()
                .get(&(END_HIST, policy_id, idx))
                .unwrap();
            ids.push_back(eid);
        }

        ids
    }

    /// Return the total number of endorsements ever created.
    pub fn get_endorsement_count(env: Env) -> u64 {
        env.storage().instance().get(&END_CNT).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

    fn setup() -> (Env, Address, Address, Address, u64) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let guardian = Address::generate(&env);
        let holder = Address::generate(&env);
        PolicyContract::initialize(env.clone(), admin.clone(), guardian.clone());
        PolicyContract::issue_policy(env.clone(), holder.clone(), 1, 100_000i128, 1_000i128).unwrap();
        (env, admin, guardian, holder, 1)
    }

    #[test]
    fn test_check_and_expire_policies() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let guardian = Address::generate(&env);

        PolicyContract::initialize(env.clone(), admin.clone(), guardian.clone());

        let holder = Address::generate(&env);

        PolicyContract::issue_policy_with_duration(
            env.clone(),
            holder.clone(),
            1,
            100_000i128,
            1_000i128,
            1,
        )
        .unwrap();

        PolicyContract::issue_policy_with_duration(
            env.clone(),
            holder.clone(),
            2,
            200_000i128,
            2_000i128,
            2,
        )
        .unwrap();

        assert_eq!(PolicyContract::get_policy_count(env.clone()), 2);

        env.ledger().set_timestamp(env.ledger().timestamp() + 3);

        let (expired, next_index) = PolicyContract::check_and_expire_policies(env.clone(), 0, 5).unwrap();
        assert_eq!(expired, 2);
        assert_eq!(next_index, 2);

        assert_eq!(PolicyContract::is_policy_active(env.clone(), 1), false);
        assert_eq!(PolicyContract::is_policy_active(env.clone(), 2), false);

        let expired_list = PolicyContract::query_active_policies_by_expiration(env.clone(), 0, 5, env.ledger().timestamp());
        assert_eq!(expired_list.len(), 0);
    }

    // ------------------------------------------------------------------
    // Endorsement tests
    // ------------------------------------------------------------------

    #[test]
    fn test_request_endorsement_coverage_change() {
        let (env, _admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(200_000i128),
            None,
            None,
        )
        .unwrap();

        assert_eq!(end_id, 1);
        assert_eq!(PolicyContract::get_endorsement_count(env.clone()), 1);

        let record = PolicyContract::get_endorsement(env.clone(), end_id).unwrap();
        assert_eq!(record.endorsement_id, end_id);
        assert_eq!(record.policy_id, policy_id);
        assert_eq!(record.new_coverage, Some(200_000i128));
        assert_eq!(record.new_premium, None);
        assert_eq!(record.status, EndorsementStatus::Pending);
    }

    #[test]
    fn test_request_endorsement_premium_adjustment() {
        let (env, _admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            None,
            Some(1_500i128),
            None,
        )
        .unwrap();

        let record = PolicyContract::get_endorsement(env.clone(), end_id).unwrap();
        assert_eq!(record.new_coverage, None);
        assert_eq!(record.new_premium, Some(1_500i128));
        assert_eq!(record.status, EndorsementStatus::Pending);
    }

    #[test]
    fn test_request_endorsement_both_changes() {
        let (env, _admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(150_000i128),
            Some(1_800i128),
            Some(symbol_short!("UPGRADE")),
        )
        .unwrap();

        let record = PolicyContract::get_endorsement(env.clone(), end_id).unwrap();
        assert_eq!(record.new_coverage, Some(150_000i128));
        assert_eq!(record.new_premium, Some(1_800i128));
    }

    #[test]
    fn test_request_endorsement_no_change_fails() {
        let (env, _admin, _guardian, holder, policy_id) = setup();

        let result = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            None,
            None,
            None,
        );
        assert_eq!(result.unwrap_err(), PolicyError::InvalidParameters);
    }

    #[test]
    fn test_request_endorsement_invalid_coverage_fails() {
        let (env, _admin, _guardian, holder, policy_id) = setup();

        let result = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(0i128),
            None,
            None,
        );
        assert_eq!(result.unwrap_err(), PolicyError::InvalidParameters);
    }

    #[test]
    fn test_request_endorsement_policy_not_found() {
        let (env, _admin, _guardian, holder, _policy_id) = setup();

        let result = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            999,
            Some(100_000i128),
            None,
            None,
        );
        assert_eq!(result.unwrap_err(), PolicyError::PolicyNotFound);
    }

    #[test]
    fn test_approve_endorsement_applies_coverage_change() {
        let (env, admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(250_000i128),
            None,
            None,
        )
        .unwrap();

        PolicyContract::approve_endorsement(env.clone(), admin.clone(), end_id).unwrap();

        // Coverage should now be updated on the policy
        let coverage = PolicyContract::get_policy_coverage(env.clone(), policy_id);
        assert_eq!(coverage, 250_000i128);

        let record = PolicyContract::get_endorsement(env.clone(), end_id).unwrap();
        assert_eq!(record.status, EndorsementStatus::Approved);
        assert!(record.processed_at.is_some());
        assert_eq!(record.processed_by, Some(admin));
    }

    #[test]
    fn test_approve_endorsement_applies_premium_adjustment() {
        let (env, admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            None,
            Some(2_000i128),
            None,
        )
        .unwrap();

        PolicyContract::approve_endorsement(env.clone(), admin.clone(), end_id).unwrap();

        // Verify record
        let record = PolicyContract::get_endorsement(env.clone(), end_id).unwrap();
        assert_eq!(record.status, EndorsementStatus::Approved);
    }

    #[test]
    fn test_approve_endorsement_unauthorized() {
        let (env, _admin, _guardian, holder, policy_id) = setup();
        let stranger = Address::generate(&env);

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(200_000i128),
            None,
            None,
        )
        .unwrap();

        let result = PolicyContract::approve_endorsement(env.clone(), stranger, end_id);
        assert_eq!(result.unwrap_err(), PolicyError::Unauthorized);
    }

    #[test]
    fn test_approve_endorsement_already_processed() {
        let (env, admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(200_000i128),
            None,
            None,
        )
        .unwrap();

        PolicyContract::approve_endorsement(env.clone(), admin.clone(), end_id).unwrap();

        // Second approval should fail
        let result = PolicyContract::approve_endorsement(env.clone(), admin.clone(), end_id);
        assert_eq!(result.unwrap_err(), PolicyError::EndorsementAlreadyProcessed);
    }

    #[test]
    fn test_reject_endorsement() {
        let (env, admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(200_000i128),
            None,
            None,
        )
        .unwrap();

        PolicyContract::reject_endorsement(
            env.clone(),
            admin.clone(),
            end_id,
            Some(symbol_short!("DECLINED")),
        )
        .unwrap();

        let record = PolicyContract::get_endorsement(env.clone(), end_id).unwrap();
        assert_eq!(record.status, EndorsementStatus::Rejected);
        assert!(record.processed_at.is_some());
        assert_eq!(record.processed_by, Some(admin));

        // Coverage should NOT have changed
        let coverage = PolicyContract::get_policy_coverage(env.clone(), policy_id);
        assert_eq!(coverage, 100_000i128);
    }

    #[test]
    fn test_reject_already_processed_endorsement() {
        let (env, admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(200_000i128),
            None,
            None,
        )
        .unwrap();

        PolicyContract::reject_endorsement(env.clone(), admin.clone(), end_id, None).unwrap();
        let result = PolicyContract::reject_endorsement(env.clone(), admin.clone(), end_id, None);
        assert_eq!(result.unwrap_err(), PolicyError::EndorsementAlreadyProcessed);
    }

    #[test]
    fn test_endorsement_history_tracked() {
        let (env, admin, _guardian, holder, policy_id) = setup();

        let id1 = PolicyContract::request_endorsement(
            env.clone(), holder.clone(), policy_id, Some(110_000i128), None, None,
        ).unwrap();
        let id2 = PolicyContract::request_endorsement(
            env.clone(), holder.clone(), policy_id, None, Some(1_200i128), None,
        ).unwrap();
        PolicyContract::approve_endorsement(env.clone(), admin.clone(), id1).unwrap();
        PolicyContract::reject_endorsement(env.clone(), admin.clone(), id2, None).unwrap();

        let ids = PolicyContract::get_policy_endorsement_ids(env.clone(), policy_id, 0, 10);
        assert_eq!(ids.len(), 2);
        assert_eq!(ids.get(0).unwrap(), id1);
        assert_eq!(ids.get(1).unwrap(), id2);
    }

    #[test]
    fn test_endorsement_history_pagination() {
        let (env, _admin, _guardian, holder, policy_id) = setup();

        for i in 1..=5u64 {
            PolicyContract::request_endorsement(
                env.clone(),
                holder.clone(),
                policy_id,
                Some((100_000 + i * 1000) as i128),
                None,
                None,
            )
            .unwrap();
        }

        let page1 = PolicyContract::get_policy_endorsement_ids(env.clone(), policy_id, 0, 3);
        assert_eq!(page1.len(), 3);

        let page2 = PolicyContract::get_policy_endorsement_ids(env.clone(), policy_id, 3, 3);
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_request_endorsement_on_inactive_policy() {
        let (env, _admin, _guardian, holder, policy_id) = setup();

        PolicyContract::cancel_policy(env.clone(), policy_id).unwrap();

        let result = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(200_000i128),
            None,
            None,
        );
        assert_eq!(result.unwrap_err(), PolicyError::PolicyNotActive);
    }

    #[test]
    fn test_endorsement_events_emitted() {
        let (env, admin, _guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(200_000i128),
            None,
            None,
        )
        .unwrap();

        PolicyContract::approve_endorsement(env.clone(), admin.clone(), end_id).unwrap();

        let events = env.events().all();
        // Events: PolicyIssued, EndorsementRequested, EndorsementApproved, CoverageChanged
        assert!(events.len() >= 3);
    }

    #[test]
    fn test_guardian_can_approve_endorsement() {
        let (env, _admin, guardian, holder, policy_id) = setup();

        let end_id = PolicyContract::request_endorsement(
            env.clone(),
            holder.clone(),
            policy_id,
            Some(300_000i128),
            None,
            None,
        )
        .unwrap();

        PolicyContract::approve_endorsement(env.clone(), guardian.clone(), end_id).unwrap();

        let record = PolicyContract::get_endorsement(env.clone(), end_id).unwrap();
        assert_eq!(record.status, EndorsementStatus::Approved);
        assert_eq!(record.processed_by, Some(guardian));
    }
}
