#![no_std]

mod storage;
mod types;
mod validation;

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, String, Vec};

use storage::{DataKey, MAX_HISTORY_ITEMS};
use types::{
    BridgeConfig, BridgeOperationStatus, BridgeTransaction, ChainBridgeInfo,
    MultisigBridgeRequest, PropertyMetadata, RecoveryAction,
};
use validation::{
    require_admin, require_non_zero_address, require_not_paused, require_operator, require_supported_chain,
    require_valid_signatures,
};

const CONTRACT_VERSION: u32 = 1;
const MAX_SUPPORTED_CHAINS: u32 = 20;
const MAX_OPERATORS: u32 = 10;

#[contract]
pub struct PropertyBridge;

#[contractimpl]
impl PropertyBridge {
    pub fn init(
        env: Env,
        admin: Address,
        supported_chains: Vec<u32>,
        min_signatures: u32,
        max_signatures: u32,
        default_timeout: u64,
        gas_limit: u64,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        require_non_zero_address(&admin);

        if supported_chains.len() > MAX_SUPPORTED_CHAINS {
            panic!("Too many chains");
        }

        let config = BridgeConfig {
            supported_chains: supported_chains.clone(),
            min_signatures_required: min_signatures,
            max_signatures_required: max_signatures,
            default_timeout_blocks: default_timeout,
            gas_limit_per_bridge: gas_limit,
            emergency_pause: false,
            metadata_preservation: true,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &CONTRACT_VERSION);
        env.storage().instance().set(&DataKey::ReqCounter, &0u64);
        env.storage().instance().set(&DataKey::TxCounter, &0u64);

        let mut operators = Vec::new(&env);
        operators.push_back(admin.clone());
        env.storage().instance().set(&DataKey::Operators, &operators);

        for chain_id in supported_chains.iter() {
            let chain_info = ChainBridgeInfo {
                chain_id,
                chain_name: String::from_str(&env, "Chain"),
                bridge_contract_address: None,
                is_active: true,
                gas_multiplier: 100,
                confirmation_blocks: 6,
                supported_tokens: Vec::new(&env),
            };
            env.storage()
                .persistent()
                .set(&DataKey::ChainInfo(chain_id), &chain_info);
        }
        
        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("init")),
            (admin, min_signatures, max_signatures),
        );
    }

    pub fn version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Version)
            .unwrap_or(CONTRACT_VERSION)
    }

    pub fn initiate_bridge_multisig(
        env: Env,
        caller: Address,
        token_id: u64,
        destination_chain: u32,
        recipient: Address,
        required_signatures: u32,
        timeout_blocks: Option<u64>,
        metadata: PropertyMetadata,
        nonce: u64,
    ) -> u64 {
        caller.require_auth();
        require_non_zero_address(&caller);
        require_non_zero_address(&recipient);

        // Nonce validation for replay protection (#349)
        let current_nonce: u64 = env.storage().persistent().get(&DataKey::Nonce(caller.clone())).unwrap_or(0);
        if nonce != current_nonce + 1 {
            panic!("Invalid nonce");
        }
        env.storage().persistent().set(&DataKey::Nonce(caller.clone()), &nonce);

        // Read config once and reuse — avoids redundant storage reads (#351, #353).
        let config: BridgeConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        require_not_paused(&env);
        require_supported_chain(&config, destination_chain);
        require_valid_signatures(&config, required_signatures);

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ReqCounter)
            .unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::ReqCounter, &counter);

        let current_block = env.ledger().sequence() as u64;
        let expires_at = timeout_blocks.map(|b| current_block + b);

        let request = MultisigBridgeRequest {
            request_id: counter,
            token_id,
            source_chain: 1,
            destination_chain,
            sender: caller.clone(),
            recipient,
            required_signatures,
            signatures: Vec::new(&env),
            created_at: current_block,
            expires_at,
            status: BridgeOperationStatus::Pending,
            metadata,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Request(counter), &request);

        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("created")),
            (counter, token_id, caller),
        );

        counter
    }

    pub fn sign_bridge_request(env: Env, operator: Address, request_id: u64, approve: bool) {
        operator.require_auth();
        require_non_zero_address(&operator);
        require_operator(&env, &operator);
        require_not_paused(&env);

        let mut request: MultisigBridgeRequest = env
            .storage()
            .persistent()
            .get(&DataKey::Request(request_id))
            .expect("Request not found");

        if let Some(expires_at) = request.expires_at {
            if (env.ledger().sequence() as u64) > expires_at {
                panic!("Request expired");
            }
        }

        if request.signatures.contains(operator.clone()) {
            panic!("Already signed");
        }

        request.signatures.push_back(operator.clone());

        if !approve {
            request.status = BridgeOperationStatus::Failed;
        } else if request.signatures.len() >= request.required_signatures {
            request.status = BridgeOperationStatus::Locked;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Request(request_id), &request);
        
        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("signed")),
            (request_id, operator, approve),
        );
    }

    pub fn execute_bridge(env: Env, operator: Address, request_id: u64) {
        operator.require_auth();
        require_non_zero_address(&operator);
        require_operator(&env, &operator);
        require_not_paused(&env);

        let mut request: MultisigBridgeRequest = env
            .storage()
            .persistent()
            .get(&DataKey::Request(request_id))
            .expect("Request not found");

        if request.status != BridgeOperationStatus::Locked {
            panic!("Request not ready");
        }

        let tx_hash = env
            .crypto()
            .sha256(&Bytes::from_slice(&env, &request_id.to_be_bytes()));

        let mut tx_counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TxCounter)
            .unwrap_or(0);
        tx_counter += 1;
        env.storage().instance().set(&DataKey::TxCounter, &tx_counter);

        // Cache sender once to avoid repeated clones on history key lookups (#351).
        let sender = request.sender.clone();

        let transaction = BridgeTransaction {
            transaction_id: tx_counter,
            token_id: request.token_id,
            source_chain: request.source_chain,
            destination_chain: request.destination_chain,
            sender: sender.clone(),
            recipient: request.recipient.clone(),
            transaction_hash: tx_hash.clone(),
            timestamp: env.ledger().timestamp(),
            gas_used: 0,
            status: BridgeOperationStatus::InTransit,
            metadata: request.metadata.clone(),
        };

        request.status = BridgeOperationStatus::Completed;
        env.storage()
            .persistent()
            .set(&DataKey::Request(request_id), &request);
        env.storage()
            .persistent()
            .set(&DataKey::VerifiedTx(tx_hash.clone()), &true);

        let mut history: Vec<BridgeTransaction> = env
            .storage()
            .persistent()
            .get(&DataKey::History(sender.clone()))
            .unwrap_or(Vec::new(&env));

        if history.len() >= MAX_HISTORY_ITEMS {
            history.remove(0);
        }
        history.push_back(transaction);
        env.storage()
            .persistent()
            .set(&DataKey::History(sender), &history);

        // Standardized event: (contract, action) topics with structured payload (#352).
        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("executed")),
            (request_id, tx_hash),
        );
    }

    pub fn recover_failed_bridge(
        env: Env,
        admin: Address,
        request_id: u64,
        recovery_action: RecoveryAction,
    ) {
        admin.require_auth();
        require_non_zero_address(&admin);
        require_admin(&env, &admin);
        require_not_paused(&env);

        let mut request: MultisigBridgeRequest = env
            .storage()
            .persistent()
            .get(&DataKey::Request(request_id))
            .expect("Request not found");

        if !matches!(
            request.status,
            BridgeOperationStatus::Failed | BridgeOperationStatus::Expired
        ) {
            panic!("Request not in failed state");
        }

        match recovery_action {
            RecoveryAction::RetryBridge => {
                request.status = BridgeOperationStatus::Pending;
                request.signatures = Vec::new(&env);
            }
            RecoveryAction::CancelBridge
            | RecoveryAction::UnlockToken
            | RecoveryAction::RefundGas => {
                request.status = BridgeOperationStatus::Failed;
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::Request(request_id), &request);
        
        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("recover")),
            request_id,
        );

            false
        }

        /// Return the local bridge chain ID used when creating source-chain requests.
        fn get_current_chain_id(&self) -> ChainId {
            // This should return the current chain ID
            // For now, we'll use a default value
            1
        }

        /// Hash the bridge request fields into a transaction identifier.
        fn generate_transaction_hash(&self, request: &MultisigBridgeRequest) -> Hash {
            // Generate a cryptographic SHA-256 hash of the bridge request to
            // ensure collision resistance and prevent trivial forgery or replay.
            use scale::Encode;
            use ink::env::hash::{Sha2x256, HashOutput};

            let data = (
                request.request_id,
                request.token_id,
                request.source_chain,
                request.destination_chain,
                request.sender,
                request.recipient,
                self.env().block_timestamp(),
            );

            let encoded_data = data.encode();

            // Compute SHA-256 over the encoded bytes
            let mut output: <Sha2x256 as HashOutput>::Type = <Sha2x256 as HashOutput>::Type::default();
            ink::env::hash_bytes::<Sha2x256>(&encoded_data, &mut output);

            // Convert the hash output to the contract `Hash` type
            Hash::from(output)
        }

        /// Estimate bridge gas from the configured base limit and metadata size.
        fn estimate_gas_usage(&self, request: &MultisigBridgeRequest) -> u64 {
            let base_gas = self.config.gas_limit_per_bridge;
            let per_byte = self
                .chain_info
                .get(request.destination_chain)
                .map(|c| c.gas_multiplier as u64)
                .unwrap_or(100);
            let metadata_gas = request.metadata.legal_description.len() as u64 * per_byte / 100;
            base_gas + metadata_gas
        }

        /// Verify a Merkle proof.
        ///
        /// Leaf = SHA-256(message_hash_bytes || leaf_index_le_bytes).
        /// Each step: if the current index is even, node = SHA-256(current || sibling),
        /// otherwise node = SHA-256(sibling || current). Matches standard binary Merkle trees.
        fn verify_merkle_proof(&self, message_hash: Hash, proof: &MerkleProof) -> bool {
            use ink::env::hash::{HashOutput, Sha2x256};

            let mut current: [u8; 32] = *message_hash.as_ref();
            // Mix in the leaf index to bind the proof to a specific position
            let index_bytes = proof.leaf_index.to_le_bytes();
            let mut leaf_input = [0u8; 40];
            leaf_input[..32].copy_from_slice(&current);
            leaf_input[32..].copy_from_slice(&index_bytes);
            let mut leaf_hash = <Sha2x256 as HashOutput>::Type::default();
            ink::env::hash_bytes::<Sha2x256>(&leaf_input, &mut leaf_hash);
            current = leaf_hash;

            let mut index = proof.leaf_index;
            for sibling in &proof.proof {
                let sibling_bytes: [u8; 32] = *sibling.as_ref();
                let mut node_input = [0u8; 64];
                if index % 2 == 0 {
                    node_input[..32].copy_from_slice(&current);
                    node_input[32..].copy_from_slice(&sibling_bytes);
                } else {
                    node_input[..32].copy_from_slice(&sibling_bytes);
                    node_input[32..].copy_from_slice(&current);
                }
                let mut node_hash = <Sha2x256 as HashOutput>::Type::default();
                ink::env::hash_bytes::<Sha2x256>(&node_input, &mut node_hash);
                current = node_hash;
                index /= 2;
            }

            Hash::from(current) == proof.root
        }
    }

    pub fn set_pause(env: Env, admin: Address, paused: bool) {
        admin.require_auth();
        require_non_zero_address(&admin);
        require_admin(&env, &admin);

        let mut config: BridgeConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        config.emergency_pause = paused;
        env.storage().instance().set(&DataKey::Config, &config);
        
        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("pause")),
            paused,
        );
    }

    pub fn add_operator(env: Env, admin: Address, operator: Address) {
        admin.require_auth();
        require_non_zero_address(&admin);
        require_non_zero_address(&operator);
        require_admin(&env, &admin);

        let mut operators: Vec<Address> =
            env.storage().instance().get(&DataKey::Operators).unwrap();
        
        if operators.len() >= MAX_OPERATORS {
            panic!("Too many operators");
        }

        if !operators.contains(operator.clone()) {
            operators.push_back(operator.clone());
            env.storage().instance().set(&DataKey::Operators, &operators);
            
            env.events().publish(
                (symbol_short!("bridge"), symbol_short!("opadd")),
                operator,
            );
        }
    }

    pub fn remove_operator(env: Env, admin: Address, operator: Address) {
        admin.require_auth();
        require_non_zero_address(&admin);
        require_non_zero_address(&operator);
        require_admin(&env, &admin);

        let operators: Vec<Address> =
            env.storage().instance().get(&DataKey::Operators).unwrap();
        let mut new_operators = Vec::new(&env);
        for op in operators.iter() {
            if op != operator {
                new_operators.push_back(op);
            }
        }
        env.storage().instance().set(&DataKey::Operators, &new_operators);
        
        env.events().publish(
            (symbol_short!("bridge"), symbol_short!("oprm")),
            operator,
        );
    }
}
