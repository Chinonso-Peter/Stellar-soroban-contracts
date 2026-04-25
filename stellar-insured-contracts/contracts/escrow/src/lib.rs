#![no_std]
//! Escrow contract for property transactions on Soroban.
//!
//! The contract manages escrow lifecycle states (`created`, `funded`, `released`) and
//! stores escrow records in a single instance storage map keyed by escrow id.
//! Storage reads are intentionally cached in local variables per operation to avoid
//! redundant instance lookups in storage-heavy paths.

use soroban_sdk::{contract, contractimpl, contracterror, Address, Env, Symbol, Map, Vec, Val, symbol_short};

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    BuyerSellerSame = 5,
    EscrowNotFound = 6,
    InvalidState = 7,
    EscrowNotCreated = 8,
    EscrowNotFunded = 9,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    fn load_escrows(env: &Env) -> Result<Map<u64, Val>, Error> {
        env.storage()
            .instance()
            .get(&symbol_short!("escrow"))
            .ok_or(Error::NotInitialized)
    }

    fn save_escrows(env: &Env, escrows: &Map<u64, Val>) {
        env.storage()
            .instance()
            .set(&symbol_short!("escrow"), escrows);
    }

    /// Initialize the escrow contract
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&symbol_short!("admin")) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&symbol_short!("admin"), &admin);
        env.storage().instance().set(&symbol_short!("escrow_count"), &0u64);
        Ok(())
    }

    /// Transfer admin to a new address (admin only)
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();
        env.storage().instance().set(&symbol_short!("admin"), &new_admin);
        Ok(())
    }

    /// Create a new escrow
    #[must_use]
    pub fn create_escrow(
        env: Env,
        property_id: u64,
        buyer: Address,
        seller: Address,
        amount: u128,
    ) -> Result<u64, Error> {
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }
        if buyer == seller {
            return Err(Error::BuyerSellerSame);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("admin"))
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let mut escrow_count: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("escrow_count"))
            .unwrap_or(0);
        escrow_count += 1;
        env.storage()
            .instance()
            .set(&symbol_short!("escrow_count"), &escrow_count);

        let escrow_key = symbol_short!("escrow");
        let mut escrows: Map<u64, Val> = env
            .storage()
            .instance()
            .get(&escrow_key)
            .unwrap_or(Map::new(&env));

        let escrow_data = (
            property_id,
            buyer.clone(),
            seller.clone(),
            amount,
            0u128,                    // deposited_amount
            symbol_short!("created"), // status
            env.ledger().timestamp(), // created_at
        );

        escrows.set(escrow_count, escrow_data.into());
        env.storage().instance().set(&escrow_key, &escrows);

        Ok(escrow_count)
    }

    /// Deposit funds into escrow
    pub fn deposit_funds(env: Env, escrow_id: u64, amount: u128) -> Result<(), Error> {
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }

        // Read escrow map once, operate in memory, write back once.
        let mut escrows: Map<u64, Val> = Self::load_escrows(&env)?;
        let escrow_data: (u64, Address, Address, u128, u128, Symbol, u64) = escrows
            .get(escrow_id)
            .ok_or(Error::EscrowNotFound)?
            .into();

        let (property_id, buyer, seller, total_amount, deposited_amount, status, created_at) =
            escrow_data;

        if status != symbol_short!("created") {
            return Err(Error::EscrowNotCreated);
        }

        buyer.require_auth();

        let new_deposited = deposited_amount + amount;
        let new_status = if new_deposited >= total_amount {
            symbol_short!("funded")
        } else {
            symbol_short!("created")
        };

        let updated_escrow = (
            property_id,
            buyer,
            seller,
            total_amount,
            new_deposited,
            new_status,
            created_at,
        );
        escrows.set(escrow_id, updated_escrow.into());
        Self::save_escrows(&env, &escrows);
        Ok(())
    }

    /// Release escrow funds
    pub fn release_escrow(env: Env, escrow_id: u64) -> Result<(), Error> {
        // Read escrow map once, operate in memory, write back once.
        let mut escrows: Map<u64, Val> = Self::load_escrows(&env)?;
        let escrow_data: (u64, Address, Address, u128, u128, Symbol, u64) = escrows
            .get(escrow_id)
            .ok_or(Error::EscrowNotFound)?
            .into();

        let (property_id, buyer, seller, total_amount, deposited_amount, status, created_at) =
            escrow_data;

        if status != symbol_short!("funded") {
            return Err(Error::EscrowNotFunded);
        }

        seller.require_auth();

        let updated_escrow = (
            property_id,
            buyer,
            seller,
            total_amount,
            deposited_amount,
            symbol_short!("released"),
            created_at,
        );
        escrows.set(escrow_id, updated_escrow.into());
        Self::save_escrows(&env, &escrows);
        Ok(())
    }

    /// Get escrow details
    #[must_use]
    pub fn get_escrow(
        env: Env,
        escrow_id: u64,
    ) -> Result<(u64, Address, Address, u128, u128, Symbol, u64), Error> {
        let escrows: Map<u64, Val> = Self::load_escrows(&env)?;
        Ok(escrows.get(escrow_id).ok_or(Error::EscrowNotFound)?.into())
    }

    /// Get total escrow count
    #[must_use]
    pub fn escrow_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&symbol_short!("escrow_count"))
            .unwrap_or(0)
    }
}
