use soroban_sdk::{Address, Env, Vec};

use crate::storage::DataKey;
use crate::types::BridgeConfig;

/// Panics if the bridge is paused.
///
/// Reads the config from storage via `&Env` — consistent with the escrow
/// contract's `require_not_paused` signature so all validation helpers
/// follow the same `&Env` convention (#353).
pub fn require_not_paused(env: &Env) {
    let config: BridgeConfig = env.storage().instance().get(&DataKey::Config).unwrap();
    if config.emergency_pause {
        panic!("Bridge paused");
    }
}

/// Panics if `destination_chain` is not in the supported chains list.
pub fn require_supported_chain(config: &BridgeConfig, destination_chain: u32) {
    if !config.supported_chains.contains(destination_chain) {
        panic!("Unsupported chain");
    }
}

/// Panics if `required_signatures` is outside the configured [min, max] range.
pub fn require_valid_signatures(config: &BridgeConfig, required_signatures: u32) {
    if required_signatures < config.min_signatures_required
        || required_signatures > config.max_signatures_required
    {
        panic!("Invalid signature requirement");
    }
}

/// Panics if `caller` is not in the operators list.
pub fn require_operator(env: &Env, caller: &Address) {
    let operators: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Operators)
        .unwrap();
    if !operators.contains(caller.clone()) {
        panic!("Not an operator");
    }
}

/// Panics if `caller` is not the stored admin.
pub fn require_admin(env: &Env, caller: &Address) {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    if *caller != admin {
        panic!("Unauthorized");
    }
}
