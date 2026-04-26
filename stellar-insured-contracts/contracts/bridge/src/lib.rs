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
    require_admin, require_not_paused, require_operator, require_supported_chain,
    require_valid_signatures,
};

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
    ) -> u64 {
        caller.require_auth();

        let config: BridgeConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        require_not_paused(&config);
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
        require_operator(&env, &operator);

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

        request.signatures.push_back(operator);

        if !approve {
            request.status = BridgeOperationStatus::Failed;
        } else if request.signatures.len() >= request.required_signatures {
            request.status = BridgeOperationStatus::Locked;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Request(request_id), &request);
    }

    pub fn execute_bridge(env: Env, operator: Address, request_id: u64) {
        operator.require_auth();
        require_operator(&env, &operator);

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

        let transaction = BridgeTransaction {
            transaction_id: tx_counter,
            token_id: request.token_id,
            source_chain: request.source_chain,
            destination_chain: request.destination_chain,
            sender: request.sender.clone(),
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
            .get(&DataKey::History(request.sender.clone()))
            .unwrap_or(Vec::new(&env));

        if history.len() >= MAX_HISTORY_ITEMS {
            history.remove(0);
        }
        history.push_back(transaction);
        env.storage()
            .persistent()
            .set(&DataKey::History(request.sender.clone()), &history);

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
        require_admin(&env, &admin);

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
    }

    pub fn set_pause(env: Env, admin: Address, paused: bool) {
        admin.require_auth();
        require_admin(&env, &admin);

        let mut config: BridgeConfig = env.storage().instance().get(&DataKey::Config).unwrap();
        config.emergency_pause = paused;
        env.storage().instance().set(&DataKey::Config, &config);
    }

    pub fn get_history(env: Env, account: Address) -> Vec<BridgeTransaction> {
        env.storage()
            .persistent()
            .get(&DataKey::History(account))
            .unwrap_or(Vec::new(&env))
    }

    pub fn add_operator(env: Env, admin: Address, operator: Address) {
        admin.require_auth();
        require_admin(&env, &admin);

        let mut operators: Vec<Address> =
            env.storage().instance().get(&DataKey::Operators).unwrap();
        if !operators.contains(operator.clone()) {
            operators.push_back(operator);
            env.storage().instance().set(&DataKey::Operators, &operators);
        }
    }

    pub fn remove_operator(env: Env, admin: Address, operator: Address) {
        admin.require_auth();
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
    }
}
