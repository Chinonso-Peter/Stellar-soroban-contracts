use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyStatus {
    Active,
    Renewed,
    Expired,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyType {
    Standard,
    Parametric,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePolicy {
    pub policy_id: u64,
    pub holder: Address,
    pub coverage_amount: i128,
    pub premium_amount: i128,
    pub start_time: u64,
    pub duration_days: u32,
    pub policy_type: PolicyType,
    pub status: PolicyStatus,
    pub risk_pool: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimStatus {
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Settled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaim {
    pub claim_id: u64,
    pub policy_id: u64,
    pub claimant: Address,
    pub amount: i128,
    pub status: ClaimStatus,
    pub submitted_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolStats {
    pub total_capital: i128,
    pub available_capital: i128,
    pub total_claims_paid: i128,
    pub provider_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub execution_data: String,
    pub creator: Address,
    pub expires_at: u64,
    pub threshold_percentage: u32,
    pub yes_votes: i128,
    pub no_votes: i128,
    pub is_finalized: bool,
    pub is_executed: bool,
}
