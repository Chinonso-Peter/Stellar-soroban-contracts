#![no_std]
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolError {
    ContractPaused = 1,
    Unauthorized = 2,
    InsufficientFunds = 3,
    PoolNotFound = 4,
    InvalidParameters = 5,
    InsufficientVestedRewards = 6,
    AppealNotFound = 7,
    AppealAlreadyResolved = 8,
    InvalidAppealStatus = 9,
    ReinsurerNotFound = 10,
    InvalidPercentage = 11,
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
pub struct VestingConfig {
    pub cliff_secs: u64,
    pub duration_secs: u64,
    pub early_withdrawal_penalty_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingStats {
    pub total_allocated_rewards: i128,
    pub total_claimed_rewards: i128,
    pub total_penalty_collected: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiquidityProvider {
    pub deposited_amount: i128,
    pub total_allocated_rewards: i128,
    pub total_claimed_rewards: i128,
    pub vesting_start: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppealStatus { Pending, Approved, Rejected }

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlashingAppeal {
    pub claim_id: u64,
    pub appealer: Address,
    pub deposit: i128,
    pub slashed_amount: i128,
    pub status: AppealStatus,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReinsuranceConfig {
    pub reinsurer: Address,
    pub percentage: u32, // basis points
    pub ceded_premiums: i128,
    pub credit_score: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolEvent {
    Deposit(Address, i128),
    Withdraw(Address, i128),
    RewardAllocated(Address, i128),
    VestedRewardsClaimed(Address, i128, i128), // provider, amount, penalty
    ContractPaused(Address, Option<Symbol>),
    ContractUnpaused(Address, Option<Symbol>),
    SlashingAppealed(u64, Address, i128),
    AppealResolved(u64, bool, i128),
    RiskCeded(Address, u32, i128),
}

const ADMIN: Symbol = symbol_short!("ADMIN");
const GUARDIAN: Symbol = symbol_short!("GUARDIAN");
const PAUSE_STATE: Symbol = symbol_short!("PAUSED");
const BALANCE: Symbol = symbol_short!("BALANCE");
const LP_ACCOUNT: Symbol = symbol_short!("LP_ACC");
const VESTING_CONFIG: Symbol = symbol_short!("VEST_CONF");
const VESTING_STATS: Symbol = symbol_short!("VEST_STATS");
const APPEALS: Symbol = symbol_short!("APPEALS");
const REINSURANCE: Symbol = symbol_short!("REINSURE");

#[contract]
pub struct RiskPoolContract;

#[contractimpl]
impl RiskPoolContract {
    pub fn initialize(env: Env, admin: Address, guardian: Address) {
        if env.storage().instance().has(&ADMIN) { panic!("Already initialized"); }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&GUARDIAN, &guardian);
        env.storage().instance().set(&PAUSE_STATE, &PauseState { is_paused: false, paused_at: None, paused_by: None, reason: None });
        env.storage().instance().set(&BALANCE, &0i128);
    }

    pub fn set_pause_state(env: Env, caller: Address, is_paused: bool, reason: Option<Symbol>) -> Result<(), PoolError> {
        caller.require_auth();
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let guardian: Address = env.storage().instance().get(&GUARDIAN).unwrap();

        if caller != admin && caller != guardian { return Err(PoolError::Unauthorized); }

        let pause_state = PauseState {
            is_paused,
            paused_at: if is_paused { Some(env.ledger().timestamp()) } else { None },
            paused_by: if is_paused { Some(caller.clone()) } else { None },
            reason: reason.clone(),
        };
        env.storage().instance().set(&PAUSE_STATE, &pause_state);

        if is_paused {
            env.events().publish((Symbol::short("PAUSE"), Symbol::short("PAUSED")), PoolEvent::ContractPaused(caller, reason));
        } else {
            env.events().publish((Symbol::short("PAUSE"), Symbol::short("UNPAUSED")), PoolEvent::ContractUnpaused(caller, reason));
        }
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get::<_, PauseState>(&PAUSE_STATE).map(|s| s.is_paused).unwrap_or(false)
    }

    fn is_admin_or_guardian(env: &Env, caller: &Address) -> bool {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let guardian: Address = env.storage().instance().get(&GUARDIAN).unwrap();
        caller == &admin || caller == &guardian
    }

    fn get_provider(env: &Env, provider: &Address) -> LiquidityProvider {
        env.storage().persistent().get(&(LP_ACCOUNT, provider)).unwrap_or(
            LiquidityProvider {
                deposited_amount: 0,
                total_allocated_rewards: 0,
                total_claimed_rewards: 0,
                vesting_start: 0,
            }
        )
    }

    fn save_provider(env: &Env, provider: &Address, account: &LiquidityProvider) {
        env.storage().persistent().set(&(LP_ACCOUNT, provider), account);
    }

    fn get_vesting_config(env: &Env) -> VestingConfig {
        env.storage().persistent().get(&VESTING_CONFIG).unwrap_or(
            VestingConfig { cliff_secs: 0, duration_secs: 0, early_withdrawal_penalty_bps: 0 }
        )
    }

    fn get_vesting_stats(env: &Env) -> VestingStats {
        env.storage().persistent().get(&VESTING_STATS).unwrap_or(
            VestingStats { total_allocated_rewards: 0, total_claimed_rewards: 0, total_penalty_collected: 0 }
        )
    }

    fn save_vesting_stats(env: &Env, stats: &VestingStats) {
        env.storage().persistent().set(&VESTING_STATS, stats);
    }

    fn vested_amount_for(pool: &LiquidityProvider, config: &VestingConfig, now: u64) -> i128 {
        if pool.total_allocated_rewards <= 0 || pool.vesting_start == 0 || config.duration_secs == 0 {
            return 0;
        }

        let cliff_end = pool.vesting_start.saturating_add(config.cliff_secs);
        let vest_end = pool.vesting_start.saturating_add(config.duration_secs);

        if now < cliff_end {
            return 0;
        }

        if now >= vest_end {
            pool.total_allocated_rewards
        } else {
            let elapsed = now.saturating_sub(pool.vesting_start);
            (pool.total_allocated_rewards as i128 * elapsed as i128) / config.duration_secs as i128
        }
    }

    pub fn set_vesting_parameters(env: Env, caller: Address, cliff_secs: u64, duration_secs: u64, early_withdrawal_penalty_bps: u32) -> Result<(), PoolError> {
        caller.require_auth();
        if !Self::is_admin_or_guardian(&env, &caller) { return Err(PoolError::Unauthorized); }
        if duration_secs == 0 || cliff_secs > duration_secs || early_withdrawal_penalty_bps > 10000 {
            return Err(PoolError::InvalidParameters);
        }

        env.storage().persistent().set(&VESTING_CONFIG, &VestingConfig { cliff_secs, duration_secs, early_withdrawal_penalty_bps });
        Ok(())
    }

    pub fn get_vesting_parameters(env: Env) -> VestingConfig {
        Self::get_vesting_config(&env)
    }

    pub fn allocate_rewards(env: Env, caller: Address, provider: Address, amount: i128) -> Result<(), PoolError> {
        caller.require_auth();
        if !Self::is_admin_or_guardian(&env, &caller) { return Err(PoolError::Unauthorized); }
        if amount <= 0 { return Err(PoolError::InvalidParameters); }

        let mut provider_record = Self::get_provider(&env, &provider);
        if provider_record.vesting_start == 0 {
            provider_record.vesting_start = env.ledger().timestamp();
        }

        provider_record.total_allocated_rewards = provider_record.total_allocated_rewards.saturating_add(amount);
        let mut stats = Self::get_vesting_stats(&env);
        stats.total_allocated_rewards = stats.total_allocated_rewards.saturating_add(amount);

        Self::save_provider(&env, &provider, &provider_record);
        Self::save_vesting_stats(&env, &stats);

        env.events().publish((Symbol::short("VEST"), Symbol::short("ALLOC")), PoolEvent::RewardAllocated(provider, amount));
        Ok(())
    }

    pub fn get_provider_vested_rewards(env: Env, provider: Address) -> i128 {
        let provider_record = Self::get_provider(&env, &provider);
        let config = Self::get_vesting_config(&env);
        let now = env.ledger().timestamp();
        let vested = Self::vested_amount_for(&provider_record, &config, now);
        let available = vested.saturating_sub(provider_record.total_claimed_rewards);
        available
    }

    pub fn claim_vested_rewards(env: Env, provider: Address) -> Result<i128, PoolError> {
        provider.require_auth();
        let mut provider_record = Self::get_provider(&env, &provider);
        let config = Self::get_vesting_config(&env);
        let now = env.ledger().timestamp();

        let vested_total = Self::vested_amount_for(&provider_record, &config, now);
        let available = vested_total.saturating_sub(provider_record.total_claimed_rewards);
        if available <= 0 { return Err(PoolError::InsufficientVestedRewards); }

        let vest_end = provider_record.vesting_start.saturating_add(config.duration_secs);
        let penalty = if now < vest_end {
            available * (config.early_withdrawal_penalty_bps as i128) / 10000
        } else {
            0
        };

        let claim_amount = available.saturating_sub(penalty);
        provider_record.total_claimed_rewards = provider_record.total_claimed_rewards.saturating_add(available);

        let mut stats = Self::get_vesting_stats(&env);
        stats.total_claimed_rewards = stats.total_claimed_rewards.saturating_add(claim_amount);
        stats.total_penalty_collected = stats.total_penalty_collected.saturating_add(penalty);

        Self::save_provider(&env, &provider, &provider_record);
        Self::save_vesting_stats(&env, &stats);

        let mut pool_balance: i128 = env.storage().instance().get(&BALANCE).unwrap_or(0);
        if pool_balance < claim_amount { return Err(PoolError::InsufficientFunds); }
        pool_balance -= claim_amount;
        env.storage().instance().set(&BALANCE, &pool_balance);

        env.events().publish((Symbol::short("VEST"), Symbol::short("CLAIM")), PoolEvent::VestedRewardsClaimed(provider, claim_amount, penalty));
        Ok(claim_amount)
    }

    pub fn get_vesting_statistics(env: Env) -> VestingStats {
        Self::get_vesting_stats(&env)
    }

    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), PoolError> {
        if Self::is_paused(env.clone()) { return Err(PoolError::ContractPaused); }
        from.require_auth();
        let mut balance: i128 = env.storage().instance().get(&BALANCE).unwrap_or(0);
        balance += amount;
        env.storage().instance().set(&BALANCE, &balance);

        let mut provider_record = Self::get_provider(&env, &from);
        provider_record.deposited_amount = provider_record.deposited_amount.saturating_add(amount);
        Self::save_provider(&env, &from, &provider_record);

        env.events().publish((BALANCE, Symbol::short("DEPOSIT")), PoolEvent::Deposit(from, amount));
        Ok(())
    }

    pub fn withdraw(env: Env, to: Address, amount: i128) -> Result<(), PoolError> {
        if Self::is_paused(env.clone()) { return Err(PoolError::ContractPaused); }
        to.require_auth();
        let mut balance: i128 = env.storage().instance().get(&BALANCE).unwrap_or(0);
        if balance < amount { return Err(PoolError::InsufficientFunds); }

        let mut provider_record = Self::get_provider(&env, &to);
        if provider_record.deposited_amount < amount { return Err(PoolError::InsufficientFunds); }

        provider_record.deposited_amount = provider_record.deposited_amount.saturating_sub(amount);
        Self::save_provider(&env, &to, &provider_record);

        balance -= amount;
        env.storage().instance().set(&BALANCE, &balance);
        env.events().publish((BALANCE, Symbol::short("WITHDRAW")), PoolEvent::Withdraw(to, amount));
        Ok(())
    }

    pub fn get_balance(env: Env) -> i128 {
        env.storage().instance().get(&BALANCE).unwrap_or(0)
    }

    pub fn appeal_slashing(env: Env, appealer: Address, claim_id: u64, deposit: i128, slashed_amount: i128) -> Result<(), PoolError> {
        if Self::is_paused(env.clone()) { return Err(PoolError::ContractPaused); }
        appealer.require_auth();

        let key = (APPEALS, claim_id);
        if env.storage().persistent().has(&key) { return Err(PoolError::InvalidParameters); }

        let deadline = env.ledger().timestamp().saturating_add(7 * 86400); // 7 days voting period
        let appeal = SlashingAppeal {
            claim_id,
            appealer: appealer.clone(),
            deposit,
            slashed_amount,
            status: AppealStatus::Pending,
            deadline,
        };

        env.storage().persistent().set(&key, &appeal);
        env.events().publish((APPEALS, symbol_short!("NEW")), PoolEvent::SlashingAppealed(claim_id, appealer, deposit));
        Ok(())
    }

    pub fn resolve_appeal(env: Env, caller: Address, claim_id: u64, approved: bool, refund_percentage: u32) -> Result<(), PoolError> {
        caller.require_auth();
        if !Self::is_admin_or_guardian(&env, &caller) { return Err(PoolError::Unauthorized); }
        if refund_percentage > 10000 { return Err(PoolError::InvalidPercentage); }

        let key = (APPEALS, claim_id);
        let mut appeal: SlashingAppeal = env.storage().persistent().get(&key).ok_or(PoolError::AppealNotFound)?;
        if appeal.status != AppealStatus::Pending { return Err(PoolError::AppealAlreadyResolved); }

        let mut refund_amount = 0;
        if approved {
            appeal.status = AppealStatus::Approved;
            refund_amount = (appeal.slashed_amount * refund_percentage as i128) / 10000;
            refund_amount += appeal.deposit; // Full refund of deposit on success
        } else {
            appeal.status = AppealStatus::Rejected;
            // Deposit is forfeited on failure
        }

        env.storage().persistent().set(&key, &appeal);
        env.events().publish((APPEALS, symbol_short!("RESOLVE")), PoolEvent::AppealResolved(claim_id, approved, refund_amount));
        Ok(())
    }

    pub fn set_reinsurance_config(env: Env, caller: Address, reinsurer: Address, percentage: u32, credit_score: u32) -> Result<(), PoolError> {
        caller.require_auth();
        if !Self::is_admin_or_guardian(&env, &caller) { return Err(PoolError::Unauthorized); }
        if percentage > 10000 { return Err(PoolError::InvalidPercentage); }

        let config = ReinsuranceConfig { reinsurer: reinsurer.clone(), percentage, ceded_premiums: 0, credit_score };
        env.storage().persistent().set(&(REINSURANCE, reinsurer), &config);
        Ok(())
    }

    pub fn cede_risk(env: Env, reinsurer: Address, amount: i128) -> Result<(), PoolError> {
        if Self::is_paused(env.clone()) { return Err(PoolError::ContractPaused); }

        let key = (REINSURANCE, reinsurer.clone());
        let mut config: ReinsuranceConfig = env.storage().persistent().get(&key).ok_or(PoolError::ReinsurerNotFound)?;

        let ceded_amount = (amount * config.percentage as i128) / 10000;
        config.ceded_premiums = config.ceded_premiums.saturating_add(ceded_amount);

        env.storage().persistent().set(&key, &config);
        env.events().publish((REINSURANCE, symbol_short!("CEDE")), PoolEvent::RiskCeded(reinsurer, config.percentage, ceded_amount));
        Ok(())
    }
}
