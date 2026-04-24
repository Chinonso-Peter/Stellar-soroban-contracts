use soroban_sdk::{Env, Address, BytesN};

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
    let mut valid_count = 0;

    for (signer, sig) in signatures {
        if config.admins.contains(&signer) {
            let verified = env.crypto().verify(&signer, &action_hash, &sig);
            if verified {
                valid_count += 1;
            }
        }
    }

    valid_count >= config.threshold
}
