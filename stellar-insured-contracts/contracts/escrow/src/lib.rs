#![no_std]

mod storage;
mod types;
mod validation;

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};

use storage::DataKey;
use types::{ApprovalType, EscrowData, EscrowStatus, MultiSigConfig};
use validation::{require_not_paused, require_valid_multisig};

#[contract]
pub struct AdvancedEscrow;

#[contractimpl]
impl AdvancedEscrow {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::EscrowCount, &0u64);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    pub fn set_pause(env: Env, admin: Address, paused: bool) {
        admin.require_auth();
        let current_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != current_admin {
            panic!("Unauthorized");
        }
        env.storage().instance().set(&DataKey::Paused, &paused);
    }

    pub fn create_escrow_advanced(
        env: Env,
        property_id: u64,
        amount: i128,
        buyer: Address,
        seller: Address,
        participants: Vec<Address>,
        required_signatures: u32,
        release_time_lock: Option<u64>,
    ) -> u64 {
        require_not_paused(&env);
        require_valid_multisig(required_signatures, participants.len());

        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::EscrowCount)
            .unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::EscrowCount, &count);

        let escrow_data = EscrowData {
            id: count,
            property_id,
            buyer,
            seller,
            amount,
            deposited_amount: 0,
            status: EscrowStatus::Created,
            created_at: env.ledger().timestamp(),
            release_time_lock,
            participants: participants.clone(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(count), &escrow_data);

        let config = MultiSigConfig {
            required_signatures,
            signers: participants,
        };
        env.storage()
            .persistent()
            .set(&DataKey::MultiSig(count), &config);

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("created")),
            (count, property_id, amount),
        );

        count
    }

    pub fn deposit_funds(env: Env, escrow_id: u64, amount: i128) {
        require_not_paused(&env);

        let mut escrow: EscrowData = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("Escrow not found");

        if escrow.status != EscrowStatus::Created && escrow.status != EscrowStatus::Funded {
            panic!("Invalid status");
        }

        escrow.deposited_amount += amount;
        escrow.status = if escrow.deposited_amount >= escrow.amount {
            EscrowStatus::Active
        } else {
            EscrowStatus::Funded
        };

        env.storage()
            .persistent()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("funded")),
            (escrow_id, amount),
        );
    }

    pub fn release_funds(env: Env, escrow_id: u64) {
        require_not_paused(&env);

        let mut escrow: EscrowData = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(escrow_id))
            .expect("Escrow not found");

        if escrow.status != EscrowStatus::Active {
            panic!("Invalid status");
        }

        if let Some(time_lock) = escrow.release_time_lock {
            if env.ledger().timestamp() < time_lock {
                panic!("Time lock active");
            }
        }

        let sig_count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::SigCount(escrow_id, ApprovalType::Release))
            .unwrap_or(0);
        let config: MultiSigConfig = env
            .storage()
            .persistent()
            .get(&DataKey::MultiSig(escrow_id))
            .unwrap();

        if sig_count < config.required_signatures {
            panic!("Signature threshold not met");
        }

        let amount = escrow.deposited_amount;
        escrow.status = EscrowStatus::Released;
        escrow.deposited_amount = 0;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(escrow_id), &escrow);

        env.events().publish(
            (symbol_short!("escrow"), symbol_short!("released")),
            (escrow_id, amount),
        );
    }

    pub fn sign_approval(env: Env, escrow_id: u64, approval_type: ApprovalType, signer: Address) {
        require_not_paused(&env);
        signer.require_auth();

        let config: MultiSigConfig = env
            .storage()
            .persistent()
            .get(&DataKey::MultiSig(escrow_id))
            .expect("Escrow not found");

        if !config.signers.contains(signer.clone()) {
            panic!("Unauthorized");
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Signature(escrow_id, approval_type, signer.clone()))
        {
            panic!("Already signed");
        }

        env.storage().persistent().set(
            &DataKey::Signature(escrow_id, approval_type, signer),
            &true,
        );

        let mut count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::SigCount(escrow_id, approval_type))
            .unwrap_or(0);
        count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::SigCount(escrow_id, approval_type), &count);
    }
}
