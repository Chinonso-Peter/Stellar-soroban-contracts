#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol};

mod policy_client {
    use soroban_sdk::{contractclient, Env};
    #[contractclient(name = "PolicyClient")]
    pub trait PolicyInterface {
        fn is_policy_active(env: Env, policy_id: u64) -> bool;
        fn get_policy_coverage(env: Env, policy_id: u64) -> i128;
    }
}
use policy_client::PolicyClient;

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ClaimStatus { Pending, Approved, Rejected, Settled }

#[contracttype]
#[derive(Clone)]
pub struct ClaimRecord {
    pub policy_id: u64,
    pub amount: i128,
    pub status: ClaimStatus,
    pub claimant: Address,
    pub evidence_count: u32,
    pub fraud_score: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FraudStatus { Clean, Suspicious, Confirmed }

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimFraudInfo {
    pub is_duplicate: bool,
    pub velocity_score: u32,
    pub compliance_checked: bool,
    pub reputation_score: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct EvidenceItem {
    pub id: u64,
    pub claim_id: u64,
    pub ipfs_hash: String,
    pub description: Option<String>,
    pub submitter: Address,
    pub submitted_at: u64,
    pub verified: bool,
    pub verified_by: Option<Address>,
    pub verified_at: Option<u64>,
    pub verification_notes: Option<String>,
    pub sensitive: bool,
}

const CLAIMS: Symbol = symbol_short!("CLAIMS");
const EVIDENCE: Symbol = symbol_short!("EVIDENCE");
const EVIDENCE_BY_CLAIM: Symbol = symbol_short!("EVIDENCE_BY_CLAIM");
const EVIDENCE_SEQ: Symbol = symbol_short!("EVIDENCE_SEQ");
const ADMIN: Symbol = symbol_short!("ADMIN");
const GUARDIAN: Symbol = symbol_short!("GUARDIAN");
const PAUSE_STATE: Symbol = symbol_short!("PAUSED");
const FRAUD_INFO: Symbol = symbol_short!("FRAUD");
const CLAIM_HISTORY: Symbol = symbol_short!("HISTORY");

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
pub enum ClaimsEvent {
    ClaimSubmitted(u64, Address, i128),
    ClaimApproved(u64),
    ClaimSettled(u64),
    EvidenceSubmitted(u64, u64, Address), // claim_id, evidence_id, submitter
    EvidenceVerified(u64, Address, bool), // evidence_id, verifier, is_valid
    ContractPaused(Address, Option<Symbol>),
    ContractUnpaused(Address, Option<Symbol>),
    FraudFlagged(u64, u32),
    FraudConfirmed(u64),
}

#[derive(soroban_sdk::contracterror, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ClaimError {
    PolicyInactive = 1,
    InsufficientCoverage = 2,
    ClaimNotFound = 3,
    EvidenceNotFound = 4,
    EvidenceAlreadyVerified = 5,
    InvalidParameters = 6,
    ClaimNotApproved = 7,
    AlreadySettled = 8,
    ContractPaused = 9,
    Unauthorized = 10,
    ClaimFlaggedAsFraud = 11,
}

#[contract]
pub struct ClaimsContract;

#[contractimpl]
impl ClaimsContract {
    pub fn initialize(env: Env, admin: Address, guardian: Address) {
        if env.storage().instance().has(&ADMIN) { panic!("Already initialized"); }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&GUARDIAN, &guardian);
        env.storage().instance().set(&PAUSE_STATE, &PauseState { is_paused: false, paused_at: None, paused_by: None, reason: None });
    }

    pub fn set_pause_state(env: Env, caller: Address, is_paused: bool, reason: Option<Symbol>) -> Result<(), ClaimError> {
        caller.require_auth();
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let guardian: Address = env.storage().instance().get(&GUARDIAN).unwrap();

        if caller != admin && caller != guardian { return Err(ClaimError::Unauthorized); }

        let pause_state = PauseState {
            is_paused,
            paused_at: if is_paused { Some(env.ledger().timestamp()) } else { None },
            paused_by: if is_paused { Some(caller.clone()) } else { None },
            reason: reason.clone(),
        };
        env.storage().instance().set(&PAUSE_STATE, &pause_state);

        if is_paused {
            env.events().publish((Symbol::short("PAUSE"), Symbol::short("PAUSED")), ClaimsEvent::ContractPaused(caller, reason));
        } else {
            env.events().publish((Symbol::short("PAUSE"), Symbol::short("UNPAUSED")), ClaimsEvent::ContractUnpaused(caller, reason));
        }
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get::<_, PauseState>(&PAUSE_STATE).map(|s| s.is_paused).unwrap_or(false)
    }

    fn next_evidence_id(env: &Env) -> u64 {
        let next: u64 = env.storage().persistent().get(&EVIDENCE_SEQ).unwrap_or(1);
        env.storage().persistent().set(&EVIDENCE_SEQ, &(next + 1));
        next
    }

    fn load_claim_record(env: &Env, claim_id: u64) -> Result<ClaimRecord, ClaimError> {
        env.storage().persistent().get(&(CLAIMS, claim_id)).ok_or(ClaimError::ClaimNotFound)
    }

    fn is_admin_or_guardian(env: &Env, caller: &Address) -> bool {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let guardian: Address = env.storage().instance().get(&GUARDIAN).unwrap();
        caller == &admin || caller == &guardian
    }

    fn is_claim_access_allowed(env: &Env, caller: &Address, claim: &ClaimRecord) -> bool {
        caller == &claim.claimant || Self::is_admin_or_guardian(env, caller)
    }

    pub fn submit_claim(env: Env, policy_address: Address, claim_id: u64, policy_id: u64, amount: i128) -> Result<(), ClaimError> {
        if Self::is_paused(env.clone()) { return Err(ClaimError::ContractPaused); }
        let policy = PolicyClient::new(&env, &policy_address);
        if !policy.is_policy_active(&policy_id) { return Err(ClaimError::PolicyInactive); }
        let coverage = policy.get_policy_coverage(&policy_id);
        let fee = amount / 100;
        if coverage <= amount + fee { return Err(ClaimError::InsufficientCoverage); }

        // Duplicate Claim Detection
        let mut fraud_score = 0;
        let history_key = (CLAIM_HISTORY, policy_id, amount);
        if env.storage().persistent().has(&history_key) {
            fraud_score += 50; // High suspicion for same amount on same policy
        }

        // Velocity Check (simplified)
        let policy_claims_count: u32 = env.storage().persistent().get(&(CLAIM_HISTORY, policy_id)).unwrap_or(0);
        if policy_claims_count > 3 {
            fraud_score += 20;
        }

        env.storage().persistent().set(&(CLAIMS, claim_id), &ClaimRecord {
            policy_id,
            amount,
            status: ClaimStatus::Pending,
            claimant: policy_address.clone(),
            evidence_count: 0,
            fraud_score,
        });

        // Update history
        env.storage().persistent().set(&history_key, &claim_id);
        env.storage().persistent().set(&(CLAIM_HISTORY, policy_id), &(policy_claims_count + 1));

        if fraud_score >= 70 {
            env.events().publish((CLAIMS, symbol_short!("FRAUD")), ClaimsEvent::FraudFlagged(claim_id, fraud_score));
        }

        env.events().publish((CLAIMS, symbol_short!("SUBMIT")), ClaimsEvent::ClaimSubmitted(claim_id, policy_address, amount));
        Ok(())
    }

    pub fn submit_evidence(
        env: Env,
        claim_id: u64,
        ipfs_hash: String,
        sensitive: bool,
        description: Option<String>,
        submitter: Address,
    ) -> Result<u64, ClaimError> {
        if Self::is_paused(env.clone()) { return Err(ClaimError::ContractPaused); }
        let mut claim = Self::load_claim_record(&env, claim_id)?;

        if ipfs_hash.len() < 10 {
            return Err(ClaimError::InvalidParameters);
        }

        let evidence_id = Self::next_evidence_id(&env);
        let timestamp = env.ledger().timestamp();
        let evidence = EvidenceItem {
            id: evidence_id,
            claim_id,
            ipfs_hash: ipfs_hash.clone(),
            description,
            submitter: submitter.clone(),
            submitted_at: timestamp,
            verified: false,
            verified_by: None,
            verified_at: None,
            verification_notes: None,
            sensitive,
        };

        env.storage().persistent().set(&(EVIDENCE, evidence_id), &evidence);
        env.storage().persistent().set(&(EVIDENCE_BY_CLAIM, claim_id, claim.evidence_count), &evidence_id);
        claim.evidence_count = claim.evidence_count.checked_add(1).unwrap_or(claim.evidence_count);
        env.storage().persistent().set(&(CLAIMS, claim_id), &claim);

        env.events().publish((CLAIMS, Symbol::short("EVIDENCE")), ClaimsEvent::EvidenceSubmitted(claim_id, evidence_id, submitter));
        Ok(evidence_id)
    }

    pub fn get_evidence(env: Env, caller: Address, evidence_id: u64) -> Result<EvidenceItem, ClaimError> {
        let evidence: EvidenceItem = env.storage().persistent().get(&(EVIDENCE, evidence_id)).ok_or(ClaimError::EvidenceNotFound)?;
        let claim = Self::load_claim_record(&env, evidence.claim_id)?;

        if evidence.sensitive && !Self::is_claim_access_allowed(&env, &caller, &claim) {
            return Err(ClaimError::Unauthorized);
        }

        Ok(evidence)
    }

    pub fn get_claim_evidence_ids(env: Env, claim_id: u64) -> Result<Vec<u64>, ClaimError> {
        let claim = Self::load_claim_record(&env, claim_id)?;
        let mut ids: Vec<u64> = Vec::new();
        for idx in 0..claim.evidence_count {
            let evidence_id: u64 = env.storage().persistent().get(&(EVIDENCE_BY_CLAIM, claim_id, idx)).unwrap();
            ids.push(evidence_id);
        }
        Ok(ids)
    }

    pub fn get_claim_evidence(env: Env, caller: Address, claim_id: u64) -> Result<Vec<EvidenceItem>, ClaimError> {
        let claim = Self::load_claim_record(&env, claim_id)?;
        let mut items: Vec<EvidenceItem> = Vec::new();

        for idx in 0..claim.evidence_count {
            let evidence_id: u64 = env.storage().persistent().get(&(EVIDENCE_BY_CLAIM, claim_id, idx)).unwrap();
            let evidence: EvidenceItem = env.storage().persistent().get(&(EVIDENCE, evidence_id)).unwrap();
            if evidence.sensitive && !Self::is_claim_access_allowed(&env, &caller, &claim) {
                continue;
            }
            items.push(evidence);
        }

        Ok(items)
    }

    pub fn verify_evidence(
        env: Env,
        caller: Address,
        evidence_id: u64,
        is_valid: bool,
        notes: Option<String>,
    ) -> Result<(), ClaimError> {
        if Self::is_paused(env.clone()) { return Err(ClaimError::ContractPaused); }
        if !Self::is_admin_or_guardian(&env, &caller) { return Err(ClaimError::Unauthorized); }

        let mut evidence: EvidenceItem = env.storage().persistent().get(&(EVIDENCE, evidence_id)).ok_or(ClaimError::EvidenceNotFound)?;
        if evidence.verified { return Err(ClaimError::EvidenceAlreadyVerified); }

        evidence.verified = is_valid;
        evidence.verified_by = Some(caller.clone());
        evidence.verified_at = Some(env.ledger().timestamp());
        evidence.verification_notes = notes.clone();
        env.storage().persistent().set(&(EVIDENCE, evidence_id), &evidence);

        env.events().publish((CLAIMS, Symbol::short("VERIFY")), ClaimsEvent::EvidenceVerified(evidence_id, caller, is_valid));
        Ok(())
    }

    pub fn is_evidence_verified(env: Env, evidence_id: u64) -> Result<bool, ClaimError> {
        let evidence: EvidenceItem = env.storage().persistent().get(&(EVIDENCE, evidence_id)).ok_or(ClaimError::EvidenceNotFound)?;
        Ok(evidence.verified)
    }

    pub fn get_evidence_verification_details(env: Env, evidence_id: u64) -> Result<(bool, Option<Address>, Option<u64>, Option<String>), ClaimError> {
        let evidence: EvidenceItem = env.storage().persistent().get(&(EVIDENCE, evidence_id)).ok_or(ClaimError::EvidenceNotFound)?;
        Ok((evidence.verified, evidence.verified_by, evidence.verified_at, evidence.verification_notes))
    }

    pub fn approve_claim(env: Env, claim_id: u64) -> Result<(), ClaimError> {
        if Self::is_paused(env.clone()) { return Err(ClaimError::ContractPaused); }
        let key = (CLAIMS, claim_id);
        let mut r: ClaimRecord = env.storage().persistent().get(&key).ok_or(ClaimError::ClaimNotFound)?;
        r.status = ClaimStatus::Approved;
        env.storage().persistent().set(&key, &r);
        env.events().publish((CLAIMS, Symbol::short("APPROVE")), ClaimsEvent::ClaimApproved(claim_id));
        Ok(())
    }

    pub fn settle_claim(env: Env, claim_id: u64) -> Result<(), ClaimError> {
        if Self::is_paused(env.clone()) { return Err(ClaimError::ContractPaused); }
        let key = (CLAIMS, claim_id);
        let mut r: ClaimRecord = env.storage().persistent().get(&key).ok_or(ClaimError::ClaimNotFound)?;
        if r.status == ClaimStatus::Settled { return Err(ClaimError::AlreadySettled); }
        if r.status != ClaimStatus::Approved { return Err(ClaimError::ClaimNotApproved); }
        if r.fraud_score >= 80 { return Err(ClaimError::ClaimFlaggedAsFraud); }
        r.status = ClaimStatus::Settled;
        env.storage().persistent().set(&key, &r);
        env.events().publish((CLAIMS, Symbol::short("SETTLE")), ClaimsEvent::ClaimSettled(claim_id));
        Ok(())
    }

    pub fn flag_claim_for_review(env: Env, caller: Address, claim_id: u64, score_adjustment: i32) -> Result<(), ClaimError> {
        caller.require_auth();
        if !Self::is_admin_or_guardian(&env, &caller) { return Err(ClaimError::Unauthorized); }

        let key = (CLAIMS, claim_id);
        let mut r: ClaimRecord = env.storage().persistent().get(&key).ok_or(ClaimError::ClaimNotFound)?;
        
        let new_score = (r.fraud_score as i32).saturating_add(score_adjustment);
        r.fraud_score = if new_score < 0 { 0 } else { new_score as u32 };
        
        env.storage().persistent().set(&key, &r);
        env.events().publish((CLAIMS, symbol_short!("REVIEW")), ClaimsEvent::FraudFlagged(claim_id, r.fraud_score));
        Ok(())
    }

    pub fn report_fraud(env: Env, caller: Address, claim_id: u64) -> Result<(), ClaimError> {
        caller.require_auth();
        if !Self::is_admin_or_guardian(&env, &caller) { return Err(ClaimError::Unauthorized); }

        let key = (CLAIMS, claim_id);
        let mut r: ClaimRecord = env.storage().persistent().get(&key).ok_or(ClaimError::ClaimNotFound)?;
        r.status = ClaimStatus::Rejected;
        r.fraud_score = 100;

        env.storage().persistent().set(&key, &r);
        env.events().publish((CLAIMS, symbol_short!("FRAUD_CFM")), ClaimsEvent::FraudConfirmed(claim_id));
        Ok(())
    }
}
