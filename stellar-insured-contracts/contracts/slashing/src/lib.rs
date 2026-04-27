#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, Symbol, Map};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Governance,
    RiskPool,
    PenaltyParams(Symbol), // Role -> Params
    ViolationCount(Address, Symbol), // (Target, Role) -> Count
    History(Address, Symbol), // (Target, Role) -> History
    SlashableRoles,
    Paused,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PenaltyParams {
    pub percentage: u32,
    pub multiplier: u32,
    pub cooldown_seconds: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlashingRecord {
    pub target: Address,
    pub role: Symbol,
    pub reason: String,
    pub amount: i128,
    pub timestamp: u64,
}

#[contract]
pub struct SlashingContract;

#[contractimpl]
impl SlashingContract {
    pub fn initialize(env: Env, admin: Address, governance: Address, risk_pool: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Governance, &governance);
        env.storage().instance().set(&DataKey::RiskPool, &risk_pool);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::SlashableRoles, &Vec::<Symbol>::new(&env));
    }

    pub fn configure_penalty_parameters(env: Env, role: Symbol, params: PenaltyParams) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        env.storage().persistent().set(&DataKey::PenaltyParams(role), &params);
    }

    pub fn slash_funds(env: Env, target: Address, role: Symbol, reason: String, amount: i128) {
        let governance: Address = env.storage().instance().get(&DataKey::Governance)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        governance.require_auth();

        if env.storage().instance().get(&DataKey::Paused).unwrap_or(false) {
            panic!("Contract paused");
        }

        if !self::SlashingContract::can_be_slashed(env.clone(), target.clone(), role.clone()) {
            panic!("Target not eligible for slashing");
        }

        // Record violation
        let mut count: u32 = env.storage().persistent().get(&DataKey::ViolationCount(target.clone(), role.clone())).unwrap_or(0);
        count += 1;
        env.storage().persistent().set(&DataKey::ViolationCount(target.clone(), role.clone()), &count);

        let record = SlashingRecord {
            target: target.clone(),
            role: role.clone(),
            reason,
            amount,
            timestamp: env.ledger().timestamp(),
        };

        let mut history: Vec<SlashingRecord> = env.storage().persistent().get(&DataKey::History(target.clone(), role.clone())).unwrap_or(Vec::new(&env));
        history.push_back(record);
        env.storage().persistent().set(&DataKey::History(target, role.clone()), &history);

        // Notify risk pool or transfer funds
        // risk_pool.slash_stake(target, amount)
        
        env.events().publish(
            (symbol_short!("slash"), role),
            amount,
        );
    }

    pub fn add_slashable_role(env: Env, role: Symbol) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        let mut roles: Vec<Symbol> = env.storage().instance().get(&DataKey::SlashableRoles).unwrap_or(Vec::new(&env));
        if !roles.contains(role.clone()) {
            roles.push_back(role);
            env.storage().instance().set(&DataKey::SlashableRoles, &roles);
        }
    }

    pub fn remove_slashable_role(env: Env, role: Symbol) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        let roles: Vec<Symbol> = env.storage().instance().get(&DataKey::SlashableRoles).unwrap_or(Vec::new(&env));
        let mut new_roles = Vec::new(&env);
        for r in roles.iter() {
            if r != role {
                new_roles.push_back(r);
            }
        }
        env.storage().instance().set(&DataKey::SlashableRoles, &new_roles);
    }

    pub fn get_slashing_history(env: Env, target: Address, role: Symbol) -> Vec<SlashingRecord> {
        env.storage().persistent().get(&DataKey::History(target, role)).unwrap_or(Vec::new(&env))
    }

    pub fn get_violation_count(env: Env, target: Address, role: Symbol) -> u32 {
        env.storage().persistent().get(&DataKey::ViolationCount(target, role)).unwrap_or(0)
    }

    pub fn can_be_slashed(env: Env, target: Address, role: Symbol) -> bool {
        let roles: Vec<Symbol> = env.storage().instance().get(&DataKey::SlashableRoles).unwrap_or(Vec::new(&env));
        if !roles.contains(role.clone()) {
            return false;
        }

        // Check cooldown
        if let Some(params) = env.storage().persistent().get::<DataKey, PenaltyParams>(&DataKey::PenaltyParams(role.clone())) {
            let history = self::SlashingContract::get_slashing_history(env.clone(), target, role);
            if let Some(last) = history.last() {
                if env.ledger().timestamp() < last.timestamp + params.cooldown_seconds {
                    return false;
                }
            }
        }

        true
    }

    pub fn pause(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
    }

    pub fn unpause(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
    }
}
