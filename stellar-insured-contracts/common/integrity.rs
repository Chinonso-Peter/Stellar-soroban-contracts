use soroban_sdk::{Env, Address, Map};

/// Run invariant checks to detect corruption or unauthorized modifications
pub fn verify_invariants(
    env: &Env,
    balances: &Map<Address, i128>,
    total_supply: i128,
    escrow_totals: i128,
    deposits_sum: i128,
) -> bool {
    // Check that total supply matches sum of balances
    let mut sum_balances: i128 = 0;
    for (_, balance) in balances.iter() {
        sum_balances += balance;
    }
    if sum_balances != total_supply {
        return false;
    }

    // Check that escrow totals match deposits
    if escrow_totals != deposits_sum {
        return false;
    }

    true
}

/// Verify contract code hash matches expected value
pub fn verify_code_hash(env: &Env, expected_hash: &BytesN<32>) -> bool {
    let current_hash = env.contract_data().code_hash();
    &current_hash == expected_hash
}
