#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, log};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    MinStake,
    TotalCapital,
    AvailableCapital,
    ClaimsPaid,
    ProviderStake(Address),
    Providers,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolStats {
    pub total_capital: i128,
    pub available_capital: i128,
    pub total_claims_paid: i128,
}

#[contract]
pub struct RiskPoolContract;

#[contractimpl]
impl RiskPoolContract {
    pub fn initialize(env: Env, admin: Address, token: Address, min_stake: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::MinStake, &min_stake);
        env.storage().instance().set(&DataKey::TotalCapital, &0i128);
        env.storage().instance().set(&DataKey::AvailableCapital, &0i128);
        env.storage().instance().set(&DataKey::ClaimsPaid, &0i128);
    }

    pub fn deposit_liquidity(env: Env, provider: Address, amount: i128) {
        provider.require_auth();
        
        let min_stake: i128 = env.storage().instance().get(&DataKey::MinStake)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        if amount < min_stake {
            panic!("Amount below minimum stake");
        }

        let token: Address = env.storage().instance().get(&DataKey::Token)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        
        // Transfer tokens from provider to this contract
        // Note: In a real implementation, we'd use the token interface
        // For this demo, we assume the token is a standard SAC
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&provider, &env.current_contract_address(), &amount);

        let mut current_stake: i128 = env.storage().persistent().get(&DataKey::ProviderStake(provider.clone())).unwrap_or(0);
        current_stake += amount;
        env.storage().persistent().set(&DataKey::ProviderStake(provider), &current_stake);

        let mut total_cap: i128 = env.storage().instance().get(&DataKey::TotalCapital)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        let mut avail_cap: i128 = env.storage().instance().get(&DataKey::AvailableCapital)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        
        total_cap += amount;
        avail_cap += amount;

        env.storage().instance().set(&DataKey::TotalCapital, &total_cap);
        env.storage().instance().set(&DataKey::AvailableCapital, &avail_cap);

        env.events().publish((symbol_short!("pool"), symbol_short!("deposit")), amount);
    }

    pub fn withdraw_liquidity(env: Env, provider: Address, amount: i128) {
        provider.require_auth();

        let mut current_stake: i128 = env.storage().persistent().get(&DataKey::ProviderStake(provider.clone())).unwrap_or(0);
        if current_stake < amount {
            panic!("Insufficient stake");
        }

        let mut avail_cap: i128 = env.storage().instance().get(&DataKey::AvailableCapital)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        if avail_cap < amount {
            panic!("Insufficient available capital in pool");
        }

        let token: Address = env.storage().instance().get(&DataKey::Token)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &provider, &amount);

        current_stake -= amount;
        env.storage().persistent().set(&DataKey::ProviderStake(provider), &current_stake);

        let mut total_cap: i128 = env.storage().instance().get(&DataKey::TotalCapital)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        total_cap -= amount;
        avail_cap -= amount;

        env.storage().instance().set(&DataKey::TotalCapital, &total_cap);
        env.storage().instance().set(&DataKey::AvailableCapital, &avail_cap);

        env.events().publish((symbol_short!("pool"), symbol_short!("withdraw")), amount);
    }

    pub fn payout_claim(env: Env, recipient: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        let mut avail_cap: i128 = env.storage().instance().get(&DataKey::AvailableCapital)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        if avail_cap < amount {
            panic!("Insufficient pool funds for payout");
        }

        let token: Address = env.storage().instance().get(&DataKey::Token)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &recipient, &amount);

        avail_cap -= amount;
        env.storage().instance().set(&DataKey::AvailableCapital, &avail_cap);

        let mut paid: i128 = env.storage().instance().get(&DataKey::ClaimsPaid)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        paid += amount;
        env.storage().instance().set(&DataKey::ClaimsPaid, &paid);

        env.events().publish((symbol_short!("pool"), symbol_short!("payout")), amount);
    }

    pub fn get_pool_stats(env: Env) -> PoolStats {
        PoolStats {
            total_capital: env.storage().instance().get(&DataKey::TotalCapital).unwrap_or(0),
            available_capital: env.storage().instance().get(&DataKey::AvailableCapital).unwrap_or(0),
            total_claims_paid: env.storage().instance().get(&DataKey::ClaimsPaid).unwrap_or(0),
        }
    }

    pub fn get_provider_info(env: Env, provider: Address) -> i128 {
        env.storage().persistent().get(&DataKey::ProviderStake(provider)).unwrap_or(0)
    }
}
