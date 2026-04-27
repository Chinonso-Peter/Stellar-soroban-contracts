#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, log};
use stellar_insured_lib::{InsuranceClaim, ClaimStatus, InsurancePolicy};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PolicyContract,
    RiskPool,
    Claim(u64),
    ClaimCounter,
}

#[contract]
pub struct ClaimsContract;

#[contractimpl]
impl ClaimsContract {
    pub fn initialize(env: Env, admin: Address, policy_contract: Address, risk_pool: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PolicyContract, &policy_contract);
        env.storage().instance().set(&DataKey::RiskPool, &risk_pool);
        env.storage().instance().set(&DataKey::ClaimCounter, &0u64);
    }

    pub fn submit_claim(env: Env, policy_id: u64, amount: i128) -> u64 {
        // In a real scenario, we'd call PolicyContract to verify the policy exists and belongs to the caller
        // For this modular example, we assume the caller is authorized via their address
        
        let mut counter: u64 = env.storage().instance().get(&DataKey::ClaimCounter).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&DataKey::ClaimCounter, &counter);

        let claimant = env.current_contract_address(); // Simplified: using contract address or passed address
        // Ideally: policy_contract.get_policy(policy_id).holder.require_auth();

        let claim = InsuranceClaim {
            claim_id: counter,
            policy_id,
            claimant: env.current_contract_address(), // Placeholder
            amount,
            status: ClaimStatus::Submitted,
            submitted_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Claim(counter), &claim);

        env.events().publish(
            (symbol_short!("claim"), symbol_short!("submitted")),
            (counter, policy_id),
        );

        counter
    }

    pub fn start_review(env: Env, claim_id: u64) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        let mut claim: InsuranceClaim = env.storage().persistent().get(&DataKey::Claim(claim_id)).expect("Claim not found");
        if claim.status != ClaimStatus::Submitted {
            panic!("Invalid claim status for review");
        }

        claim.status = ClaimStatus::UnderReview;
        env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);

        env.events().publish(
            (symbol_short!("claim"), symbol_short!("review")),
            claim_id,
        );
    }

    pub fn approve_claim(env: Env, claim_id: u64) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        let mut claim: InsuranceClaim = env.storage().persistent().get(&DataKey::Claim(claim_id)).expect("Claim not found");
        if claim.status != ClaimStatus::UnderReview {
            panic!("Claim must be under review to approve");
        }

        claim.status = ClaimStatus::Approved;
        env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);

        env.events().publish(
            (symbol_short!("claim"), symbol_short!("approved")),
            claim_id,
        );
    }

    pub fn reject_claim(env: Env, claim_id: u64) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        let mut claim: InsuranceClaim = env.storage().persistent().get(&DataKey::Claim(claim_id)).expect("Claim not found");
        if claim.status != ClaimStatus::UnderReview {
            panic!("Claim must be under review to reject");
        }

        claim.status = ClaimStatus::Rejected;
        env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);

        env.events().publish(
            (symbol_short!("claim"), symbol_short!("rejected")),
            claim_id,
        );
    }

    pub fn settle_claim(env: Env, claim_id: u64) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        admin.require_auth();

        let mut claim: InsuranceClaim = env.storage().persistent().get(&DataKey::Claim(claim_id)).expect("Claim not found");
        if claim.status != ClaimStatus::Approved {
            panic!("Only approved claims can be settled");
        }

        // Cross-contract call to Risk Pool to payout
        let risk_pool: Address = env.storage().instance().get(&DataKey::RiskPool)
            .unwrap_or_else(|| panic!("Contract not initialized"));
        
        // payout_claim(recipient, amount)
        env.invoke_contract::<()>(
            &risk_pool,
            &symbol_short!("payout"),
            (claim.claimant.clone(), claim.amount).into(),
        );

        claim.status = ClaimStatus::Settled;
        env.storage().persistent().set(&DataKey::Claim(claim_id), &claim);

        env.events().publish(
            (symbol_short!("claim"), symbol_short!("settled")),
            claim_id,
        );
    }

    pub fn get_claim(env: Env, claim_id: u64) -> InsuranceClaim {
        env.storage().persistent().get(&DataKey::Claim(claim_id)).expect("Claim not found")
    }

    pub fn get_stats(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::ClaimCounter).unwrap_or(0)
    }
}
