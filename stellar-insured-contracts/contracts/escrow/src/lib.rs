#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Map, Vec, Val, symbol_short};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow contract
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&symbol_short!("admin")) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("escrow_count"), &0u64);
    }

    /// Create a new escrow
    pub fn create_escrow(
        env: Env,
        property_id: u64,
        buyer: Address,
        seller: Address,
        amount: u128,
    ) -> u64 {
        let admin: Address = env.storage().instance().get(&symbol_short!("admin")).unwrap();
        admin.require_auth();

        let mut escrow_count: u64 = env.storage().instance().get(&symbol_short!("escrow_count")).unwrap_or(0);
        escrow_count += 1;
        env.storage().instance().set(&symbol_short!("escrow_count"), &escrow_count);

        let escrow_key = symbol_short!("escrow");
        let mut escrows: Map<u64, Val> = env.storage().instance().get(&escrow_key).unwrap_or(Map::new(&env));

        let escrow_data = (
            property_id,
            buyer.clone(),
            seller.clone(),
            amount,
            0u128, // deposited_amount
            symbol_short!("created"), // status
            env.ledger().timestamp(), // created_at
        );

        escrows.set(escrow_count, escrow_data.into());
        env.storage().instance().set(&escrow_key, &escrows);

        escrow_count
    }

    /// Deposit funds into escrow
    pub fn deposit_funds(env: Env, escrow_id: u64, amount: u128) {
        let escrow_key = symbol_short!("escrow");
        let mut escrows: Map<u64, Val> = env.storage().instance().get(&escrow_key).unwrap();
        let escrow_data: (u64, Address, Address, u128, u128, Symbol, u64) = escrows.get(escrow_id).unwrap().into();

        let (property_id, buyer, seller, total_amount, deposited_amount, status, created_at) = escrow_data;

        if status != symbol_short!("created") {
            panic!("Escrow not in created state");
        }

        // Transfer tokens (simplified - in real implementation would transfer actual tokens)
        buyer.require_auth();

        let new_deposited = deposited_amount + amount;
        let new_status = if new_deposited >= total_amount {
            symbol_short!("funded")
        } else {
            symbol_short!("created")
        };

        let updated_escrow = (property_id, buyer, seller, total_amount, new_deposited, new_status, created_at);
        escrows.set(escrow_id, updated_escrow.into());
        env.storage().instance().set(&escrow_key, &escrows);
    }

    /// Release escrow funds
    pub fn release_escrow(env: Env, escrow_id: u64) {
        let escrow_key = symbol_short!("escrow");
        let mut escrows: Map<u64, Val> = env.storage().instance().get(&escrow_key).unwrap();
        let escrow_data: (u64, Address, Address, u128, u128, Symbol, u64) = escrows.get(escrow_id).unwrap().into();

        let (property_id, buyer, seller, total_amount, deposited_amount, status, created_at) = escrow_data;

        if status != symbol_short!("funded") {
            panic!("Escrow not funded");
        }

        // Transfer funds to seller (simplified)
        seller.require_auth();

        let updated_escrow = (property_id, buyer, seller, total_amount, deposited_amount, symbol_short!("released"), created_at);
        escrows.set(escrow_id, updated_escrow.into());
        env.storage().instance().set(&escrow_key, &escrows);
    }

    /// Get escrow details
    pub fn get_escrow(env: Env, escrow_id: u64) -> (u64, Address, Address, u128, u128, Symbol, u64) {
        let escrow_key = symbol_short!("escrow");
        let escrows: Map<u64, Val> = env.storage().instance().get(&escrow_key).unwrap();
        escrows.get(escrow_id).unwrap().into()
    }

    /// Get total escrow count
    pub fn escrow_count(env: Env) -> u64 {
        env.storage().instance().get(&symbol_short!("escrow_count")).unwrap_or(0)
    }
}
