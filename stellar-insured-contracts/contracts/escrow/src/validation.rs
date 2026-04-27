use soroban_sdk::Env;

use crate::storage::DataKey;

/// Panics if the contract is paused.
///
/// Always pass `&Env` (by reference) to avoid unnecessary clones — `Env` is
/// cheap to clone but passing by reference is the idiomatic pattern for
/// helper/validation functions that do not need ownership (#353).
pub fn require_not_paused(env: &Env) {
    let paused: bool = env
        .storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false);
    if paused {
        panic!("Contract is paused");
    }
}

/// Panics if `required_signatures` is zero, `participants` is empty,
/// or `required_signatures` exceeds the number of participants.
pub fn require_valid_multisig(required_signatures: u32, participant_count: u32) {
    if required_signatures == 0
        || participant_count == 0
        || required_signatures > participant_count
    {
        panic!("Invalid configuration");
    }
}

/// Reads and returns the stored admin address.
///
/// Centralises the admin lookup so callers avoid a raw `.get(&DataKey::Admin)`
/// scattered across functions — one read, one place (#353, #351).
pub fn get_admin(env: &Env) -> soroban_sdk::Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Not initialized")
}
