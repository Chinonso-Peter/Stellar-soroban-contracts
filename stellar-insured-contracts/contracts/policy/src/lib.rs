#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, log};
use stellar_insured_lib::{InsurancePolicy, PolicyStatus, PolicyType};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    RiskPool,
    Policy(u64),
    PolicyCounter,
}

#[contract]
pub struct PolicyContract;

#[contractimpl]
impl PolicyContract {
    pub fn initialize(env: Env, admin: Address, risk_pool: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::RiskPool, &risk_pool);
        env.storage().instance().set(&DataKey::PolicyCounter, &0u64);
    }

    pub fn issue_policy(
        env: Env,
        holder: Address,
        coverage_amount: i128,
        premium_amount: i128,
        duration_days: u32,
        policy_type: PolicyType,
    ) -> u64 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut counter: u64 = env.storage().instance().get(&DataKey::PolicyCounter).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::PolicyCounter, &counter);

        let risk_pool: Address = env.storage().instance().get(&DataKey::RiskPool).unwrap();

        let policy = InsurancePolicy {
            policy_id: counter,
            holder: holder.clone(),
            coverage_amount,
            premium_amount,
            start_time: env.ledger().timestamp(),
            duration_days,
            policy_type,
            status: PolicyStatus::Active,
            risk_pool,
        };

        env.storage().persistent().set(&DataKey::Policy(counter), &policy);

        env.events().publish(
            (symbol_short!("policy"), symbol_short!("issued")),
            (counter, holder),
        );

        counter
    }

    pub fn get_policy(env: Env, policy_id: u64) -> InsurancePolicy {
        env.storage().persistent().get(&DataKey::Policy(policy_id)).expect("Policy not found")
    }

    pub fn renew_policy(env: Env, policy_id: u64, duration_days: u32) {
        let mut policy: InsurancePolicy = env.storage().persistent().get(&DataKey::Policy(policy_id)).expect("Policy not found");
        policy.holder.require_auth();

        if policy.status != PolicyStatus::Active && policy.status != PolicyStatus::Renewed {
            panic!("Policy not active");
        }

        policy.duration_days += duration_days;
        policy.status = PolicyStatus::Renewed;

        env.storage().persistent().set(&DataKey::Policy(policy_id), &policy);

        env.events().publish(
            (symbol_short!("policy"), symbol_short!("renewed")),
            policy_id,
        );
    }

    pub fn cancel_policy(env: Env, policy_id: u64) {
        let mut policy: InsurancePolicy = env.storage().persistent().get(&DataKey::Policy(policy_id)).expect("Policy not found");
        policy.holder.require_auth();

        policy.status = PolicyStatus::Cancelled;
        env.storage().persistent().set(&DataKey::Policy(policy_id), &policy);

        env.events().publish(
            (symbol_short!("policy"), symbol_short!("cancelled")),
            policy_id,
        );
    }

    pub fn expire_policy(env: Env, policy_id: u64) {
        let mut policy: InsurancePolicy = env.storage().persistent().get(&DataKey::Policy(policy_id)).expect("Policy not found");
        
        let now = env.ledger().timestamp();
        let expiry = policy.start_time + (policy.duration_days as u64 * 86400);

        if now < expiry {
            panic!("Policy not yet expired");
        }

        policy.status = PolicyStatus::Expired;
        env.storage().persistent().set(&DataKey::Policy(policy_id), &policy);

        env.events().publish(
            (symbol_short!("policy"), symbol_short!("expired")),
            policy_id,
        );
    }

    pub fn get_stats(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::PolicyCounter).unwrap_or(0)
    }
}
