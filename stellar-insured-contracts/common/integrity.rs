use soroban_sdk::{Env, Address, Map, BytesN};

/// Run invariant checks to detect corruption or unauthorized modifications
pub fn verify_invariants(
    env: &Env,
    balances: &Map<Address, i128>,
    total_supply: i128,
    escrow_totals: i128,
    deposits_sum: i128,
) -> bool {
    // Bounded iteration to prevent unbounded looping
    let mut sum_balances: i128 = 0;
    let mut count: u32 = 0;
    const MAX_ITER: u32 = 100;

    for (_, balance) in balances.iter() {
        if count >= MAX_ITER {
            break;
        }
        sum_balances += balance;
        count += 1;
    }

    // NOTE:
    // If balances exceed MAX_ITER, this is a partial check.
    // Full invariant enforcement should happen during state updates.

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
