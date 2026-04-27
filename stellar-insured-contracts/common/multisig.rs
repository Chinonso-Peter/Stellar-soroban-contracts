use soroban_sdk::{Env, Address, BytesN, Vec};

/// Configuration for multisig admin control
pub struct AdminConfig {
    pub admins: Vec<Address>,
    pub threshold: u32, // e.g. 2 for 2-of-3, 3 for 3-of-5
}

/// Verify multisig signatures for a given action
pub fn verify_multisig(
    env: &Env,
    action_hash: BytesN<32>,
    signatures: Vec<(Address, BytesN<64>)>,
    config: &AdminConfig,
) -> bool {
    const MAX_SIGNATURES: u32 = 20;

    // Prevent unbounded input processing
    if signatures.len() > MAX_SIGNATURES {
        panic!("Too many signatures");
    }

    let mut valid_count: u32 = 0;

    for (signer, sig) in signatures {
        // Early exit when threshold is reached
        if valid_count >= config.threshold {
            break;
        }

        if config.admins.contains(&signer) {
            if env.crypto().verify(&signer, &action_hash, &sig) {
                valid_count += 1;
            }
        }
    }

    valid_count >= config.threshold
}