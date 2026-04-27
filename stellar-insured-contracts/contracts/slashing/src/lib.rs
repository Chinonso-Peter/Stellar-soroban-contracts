#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, Symbol};

// Maximum slashing history entries per (target, role) to prevent storage bloat (#380)
const MAX_HISTORY: u32 = 50;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Governance,
    RiskPool,
    PenaltyParams(Symbol),
    ViolationCount(Address, Symbol),
    History(Address, Symbol),
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

// --- Storage helpers (#378: data access abstraction) ---

fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

fn get_governance(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Governance).unwrap()
}

fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
}

fn get_slashable_roles(env: &Env) -> Vec<Symbol> {
    env.storage().instance().get(&DataKey::SlashableRoles).unwrap_or(Vec::new(env))
}

fn get_violation_count_inner(env: &Env, target: &Address, role: &Symbol) -> u32 {
    env.storage().persistent().get(&DataKey::ViolationCount(target.clone(), role.clone())).unwrap_or(0)
}

fn get_history_inner(env: &Env, target: &Address, role: &Symbol) -> Vec<SlashingRecord> {
    env.storage().persistent().get(&DataKey::History(target.clone(), role.clone())).unwrap_or(Vec::new(env))
}

fn set_history(env: &Env, target: &Address, role: &Symbol, history: &Vec<SlashingRecord>) {
    env.storage().persistent().set(&DataKey::History(target.clone(), role.clone()), history);
}

// --------------------------------------------------------

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
        
        env.events().publish(
            (symbol_short!("slash"), symbol_short!("init")),
            (admin, governance, risk_pool),
        );
    }

    pub fn configure_penalty_parameters(env: Env, role: Symbol, params: PenaltyParams) {
        let admin = get_admin(&env);
        admin.require_auth();

        env.storage().persistent().set(&DataKey::PenaltyParams(role.clone()), &params);
        
        env.events().publish(
            (symbol_short!("slash"), symbol_short!("config")),
            (role, params.percentage, params.multiplier),

        // #379: emit event for admin action
        env.events().publish(
            (symbol_short!("admin"), symbol_short!("cfg_pen")),
            role,
        );
    }

    pub fn slash_funds(env: Env, target: Address, role: Symbol, reason: String, amount: i128) {
        let governance = get_governance(&env);
        governance.require_auth();

        if is_paused(&env) {
            panic!("Contract paused");
        }

        if !Self::can_be_slashed(env.clone(), target.clone(), role.clone()) {
            panic!("Target not eligible for slashing");
        }

        let mut count = get_violation_count_inner(&env, &target, &role);
        count += 1;
        env.storage().persistent().set(&DataKey::ViolationCount(target.clone(), role.clone()), &count);

        let record = SlashingRecord {
            target: target.clone(),
            role: role.clone(),
            reason,
            amount,
            timestamp: env.ledger().timestamp(),
        };

        // #380: cap history to MAX_HISTORY entries to prevent storage bloat
        let mut history = get_history_inner(&env, &target, &role);
        if history.len() >= MAX_HISTORY {
            // Remove oldest entry (index 0)
            let mut trimmed = Vec::new(&env);
            for i in 1..history.len() {
                trimmed.push_back(history.get(i).unwrap());
            }
            history = trimmed;
        }
        history.push_back(record);
        set_history(&env, &target, &role, &history);

        env.events().publish(
            (symbol_short!("slash"), role),
            amount,
        );
    }

    pub fn add_slashable_role(env: Env, role: Symbol) {
        let admin = get_admin(&env);
        admin.require_auth();

        let mut roles = get_slashable_roles(&env);
        if !roles.contains(role.clone()) {
            roles.push_back(role.clone());
            env.storage().instance().set(&DataKey::SlashableRoles, &roles);
            
            env.events().publish(
                (symbol_short!("slash"), symbol_short!("roleadd")),
                role,
            );
        }

        // #379: emit event for admin action
        env.events().publish(
            (symbol_short!("admin"), symbol_short!("role_add")),
            role,
        );
    }

    pub fn remove_slashable_role(env: Env, role: Symbol) {
        let admin = get_admin(&env);
        admin.require_auth();

        let roles = get_slashable_roles(&env);
        let mut new_roles = Vec::new(&env);
        for r in roles.iter() {
            if r != role {
                new_roles.push_back(r);
            }
        }
        env.storage().instance().set(&DataKey::SlashableRoles, &new_roles);
        
        env.events().publish(
            (symbol_short!("slash"), symbol_short!("rolerm")),

        // #379: emit event for admin action
        env.events().publish(
            (symbol_short!("admin"), symbol_short!("role_rm")),
            role,
        );
    }

    pub fn get_slashing_history(env: Env, target: Address, role: Symbol) -> Vec<SlashingRecord> {
        get_history_inner(&env, &target, &role)
    }

    pub fn get_violation_count(env: Env, target: Address, role: Symbol) -> u32 {
        get_violation_count_inner(&env, &target, &role)
    }

    pub fn can_be_slashed(env: Env, target: Address, role: Symbol) -> bool {
        let roles = get_slashable_roles(&env);
        if !roles.contains(role.clone()) {
            return false;
        }

        if let Some(params) = env.storage().persistent().get::<DataKey, PenaltyParams>(&DataKey::PenaltyParams(role.clone())) {
            let history = get_history_inner(&env, &target, &role);
            if let Some(last) = history.last() {
                if env.ledger().timestamp() < last.timestamp + params.cooldown_seconds {
                    return false;
                }
            }
        }

        true
    }

    pub fn pause(env: Env) {
        let admin = get_admin(&env);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &true);
        
        env.events().publish(
            (symbol_short!("slash"), symbol_short!("pause")),

        // #379: emit event for admin action
        env.events().publish(
            (symbol_short!("admin"), symbol_short!("paused")),
            true,
        );
    }

    pub fn unpause(env: Env) {
        let admin = get_admin(&env);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Paused, &false);
        
        env.events().publish(
            (symbol_short!("slash"), symbol_short!("unpause")),

        // #379: emit event for admin action
        env.events().publish(
            (symbol_short!("admin"), symbol_short!("paused")),
            false,
        );
    }
}
