#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::needless_borrows_for_generic_args
)]

//! Property insurance contract module wiring, types, and delegated implementations.

use ink::storage::Mapping;

mod rbac;
pub use rbac::{Role, RoleManager};

/// Decentralized Property Insurance Platform
#[ink::contract]
mod propchain_insurance {
    use super::*;
    use ink::prelude::{string::String, vec::Vec};
    use crate::{Role, RoleManager};

    pub use crate::types::{
        ActuarialModel, BatchClaimResult, BatchClaimSummary, ClaimStatus, CoverageType,
        EvidenceItem, EvidenceMetadata, EvidenceVerification, InsuranceClaim, InsuranceError,
        InsurancePolicy, InsuranceToken, PolicyStatus, PolicyType, PoolLiquidityProvider,
        PremiumCalculation, ReinsuranceAgreement, RiskAssessment, RiskLevel, RiskPool,
        UnderwritingCriteria, REWARD_PRECISION,
    };

    // =========================================================================
    // EVENTS  (extracted to events.rs, included here so ink! macros see them)
    // =========================================================================
    include!("events.rs");

    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PolicyStatus {
        Active,
        Renewed,
        Expired,
        Cancelled,
        Claimed,
        Suspended,
    }

    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum PolicyType {
        Standard,
        Parametric,
    }

    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CoverageType {
        Fire,
        Flood,
        Earthquake,
        Theft,
        LiabilityDamage,
        NaturalDisaster,
        Comprehensive,
    }

    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ClaimStatus {
        Pending,
        UnderReview,
        OracleVerifying,
        Approved,
        Rejected,
        Paid,
        Disputed,
    }

    #[derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        scale::Encode,
        scale::Decode,
        ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum RiskLevel {
        VeryLow,
        Low,
        Medium,
        High,
        VeryHigh,
    }

    /// Structured evidence attached to a claim submission.
    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct EvidenceMetadata {
        /// Non-empty nonce / type identifier (e.g. "photo", "report", "sensor")
        pub evidence_type: String,
        /// URI pointing to the evidence artifact (must start with "ipfs://" or "https://")
        pub reference_uri: String,
        /// SHA-256 content hash – exactly 32 bytes
        pub content_hash: Vec<u8>,
        /// Optional human-readable description
        pub description: Option<String>,
    }

    impl From<&str> for EvidenceMetadata {
        /// Convert a legacy evidence URI into structured evidence metadata defaults.
        fn from(s: &str) -> Self {
            EvidenceMetadata {
                evidence_type: "unknown".into(),
                reference_uri: s.into(),
                content_hash: vec![0u8; 32],
                description: None,
            }
        }
    }

        #[derive(
            Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
        )]
        #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
        pub struct EvidenceItem {
            pub id: u64,
            pub claim_id: u64,
            pub evidence_type: String,
            pub ipfs_hash: String,
            pub ipfs_uri: String,
            pub content_hash: Vec<u8>,
            pub file_size: u64,
            pub submitter: AccountId,
            pub submitted_at: u64,
            pub verified: bool,
            pub verified_by: Option<AccountId>,
            pub verified_at: Option<u64>,
            pub verification_notes: Option<String>,
            pub metadata_url: Option<String>,
        }

        #[derive(
            Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
        )]
        #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
        pub struct EvidenceVerification {
            pub evidence_id: u64,
            pub verifier: AccountId,
            pub verified_at: u64,
            pub is_valid: bool,
            pub notes: String,
            pub ipfs_accessible: bool,
            pub hash_matches: bool,
        }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct InsurancePolicy {
        pub policy_id: u64,
        pub property_id: u64,
        pub policyholder: AccountId,
        pub coverage_type: CoverageType,
        pub coverage_amount: u128, // Max payout in USD (8 decimals)
        pub premium_amount: u128,  // Annual premium in native token
        pub deductible: u128,      // Deductible amount
        pub start_time: u64,
        pub end_time: u64,
        pub status: PolicyStatus,
        pub risk_level: RiskLevel,
        pub pool_id: u64,
        pub claims_count: u32,
        pub total_claimed: u128,
        pub metadata_url: String,
        pub policy_type: PolicyType,
        pub event_id: Option<u64>,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct InsuranceClaim {
        pub claim_id: u64,
        pub policy_id: u64,
        pub claimant: AccountId,
        pub claim_amount: u128,
        pub description: String,
        pub evidence: EvidenceMetadata,
        pub evidence_ids: Vec<u64>,
        pub oracle_report_url: String,
        pub status: ClaimStatus,
        pub submitted_at: u64,
        pub processed_at: Option<u64>,
        pub payout_amount: u128,
        pub assessor: Option<AccountId>,
        pub rejection_reason: String,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RiskPool {
        pub pool_id: u64,
        pub name: String,
        pub coverage_type: CoverageType,
        pub total_capital: u128,
        pub available_capital: u128,
        pub total_premiums_collected: u128,
        pub total_claims_paid: u128,
        pub active_policies: u64,
        pub max_coverage_ratio: u32, // Max exposure as % of pool (basis points, e.g. 8000 = 80%)
        pub reinsurance_threshold: u128, // Claim size above which reinsurance kicks in
        pub created_at: u64,
        pub is_active: bool,
        /// Sum of LP stakes; denominator for reward-per-share accrual.
        pub total_provider_stake: u128,
        /// Scaled accumulated rewards per staked unit ([`REWARD_PRECISION`] fixed-point).
        pub accumulated_reward_per_share: u128,
        /// Vesting cliff in seconds (0 = no cliff)
        pub vesting_cliff_seconds: u64,
        /// Vesting duration in seconds (0 = no vesting; if >0 enables vesting)
        pub vesting_duration_seconds: u64,
        /// Early withdrawal penalty applied to unvested rewards (basis points, e.g. 500 = 5%)
        pub early_withdrawal_penalty_bps: u32,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RiskAssessment {
        pub property_id: u64,
        pub location_risk_score: u32,     // 0-100
        pub construction_risk_score: u32, // 0-100
        pub age_risk_score: u32,          // 0-100
        pub claims_history_score: u32,    // 0-100 (lower = more claims)
        pub overall_risk_score: u32,      // 0-100
        pub risk_level: RiskLevel,
        pub assessed_at: u64,
        pub valid_until: u64,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PremiumCalculation {
        pub base_rate: u32,           // Basis points (e.g. 150 = 1.50%)
        pub risk_multiplier: u32,     // Applied based on risk score (100 = 1.0x)
        pub coverage_multiplier: u32, // Applied based on coverage type
        pub annual_premium: u128,     // Final annual premium
        pub monthly_premium: u128,    // Monthly equivalent
        pub deductible: u128,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ReinsuranceAgreement {
        pub agreement_id: u64,
        pub reinsurer: AccountId,
        pub coverage_limit: u128,
        pub retention_limit: u128, // Our retention before reinsurance activates
        pub premium_ceded_rate: u32, // % of premiums ceded to reinsurer (basis points)
        pub coverage_types: Vec<CoverageType>,
        pub start_time: u64,
        pub end_time: u64,
        pub is_active: bool,
        pub total_ceded_premiums: u128,
        pub total_recoveries: u128,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct InsuranceToken {
        pub token_id: u64,
        pub policy_id: u64,
        pub owner: AccountId,
        pub face_value: u128,
        pub is_tradeable: bool,
        pub created_at: u64,
        pub listed_price: Option<u128>,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ActuarialModel {
        pub model_id: u64,
        pub coverage_type: CoverageType,
        pub loss_frequency: u32, // Expected losses per 1000 policies (basis points)
        pub average_loss_severity: u128, // Average loss size
        pub expected_loss_ratio: u32, // Expected loss ratio (basis points)
        pub confidence_level: u32, // 0-100
        pub last_updated: u64,
        pub data_points: u32,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct UnderwritingCriteria {
        pub max_property_age_years: u32,
        pub min_property_value: u128,
        pub max_property_value: u128,
        pub excluded_locations: Vec<String>,
        pub required_safety_features: bool,
        pub max_previous_claims: u32,
        pub min_risk_score: u32,
    }

    /// Result of a single batch claim operation
    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct BatchClaimResult {
        pub claim_id: u64,
        pub success: bool,
        pub error: Option<InsuranceError>,
    }

    /// Summary of batch claim processing
    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct BatchClaimSummary {
        pub total_processed: u64,
        pub successful: u64,
        pub failed: u64,
        pub results: Vec<BatchClaimResult>,
    }

    #[derive(
        Debug, Clone, PartialEq, scale::Encode, scale::Decode, ink::storage::traits::StorageLayout,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolLiquidityProvider {
        pub provider: AccountId,
        pub pool_id: u64,
        pub provider_stake: u128,
        /// Reward debt in fixed-point units: keeps pending = stake * acc_rps / P - reward_debt.
        pub reward_debt: u128,
        pub deposited_at: u64,
        /// Total rewards moved into vesting schedule (not yet fully claimed)
        pub vesting_total: u128,
        /// Amount of vested rewards already claimed by provider
        pub vesting_claimed: u128,
        /// Vesting schedule start timestamp (seconds)
        pub vesting_start: u64,
    }

    // =========================================================================
    // STORAGE
    // =========================================================================

    #[ink(storage)]
    pub struct PropertyInsurance {
        admin: AccountId,

        // Role-based access control
        role_manager: RoleManager,

        // Policies
        policies: Mapping<u64, InsurancePolicy>,
        policy_count: u64,
        policyholder_policies: Mapping<AccountId, Vec<u64>>,
        property_policies: Mapping<u64, Vec<u64>>,

        // Claims
        claims: Mapping<u64, InsuranceClaim>,
        claim_count: u64,
        policy_claims: Mapping<u64, Vec<u64>>,

        // Risk Pools
        pools: Mapping<u64, RiskPool>,
        pool_count: u64,

        // Risk Assessments
        risk_assessments: Mapping<u64, RiskAssessment>,

        // Reinsurance
        reinsurance_agreements: Mapping<u64, ReinsuranceAgreement>,
        reinsurance_count: u64,

        // Insurance Tokens (secondary market)
        insurance_tokens: Mapping<u64, InsuranceToken>,
        token_count: u64,
        token_listings: Vec<u64>,

        // Actuarial Models
        actuarial_models: Mapping<u64, ActuarialModel>,
        model_count: u64,

        // Underwriting
        underwriting_criteria: Mapping<u64, UnderwritingCriteria>,

        // Liquidity providers
        liquidity_providers: Mapping<(u64, AccountId), PoolLiquidityProvider>,
        pool_providers: Mapping<u64, Vec<AccountId>>,

        // Oracle addresses
        authorized_oracles: Mapping<AccountId, bool>,

        // Assessors
        authorized_assessors: Mapping<AccountId, bool>,

        // Claim cooldown: property_id -> last_claim_timestamp
        claim_cooldowns: Mapping<u64, u64>,
        // Rate limiting: caller -> last_submit_claim_timestamp
        caller_last_claim: Mapping<AccountId, u64>,

        // Evidence tracking
        evidence_count: u64,
        evidence_items: Mapping<u64, EvidenceItem>,
        claim_evidence: Mapping<u64, Vec<u64>>,
        evidence_verifications: Mapping<u64, Vec<EvidenceVerification>>,

        // Oracle contract for parametric claims
        oracle_contract: Option<AccountId>,

        // Platform settings
        platform_fee_rate: u32,
        claim_cooldown_period: u64,
        min_pool_capital: u128,
        dispute_window_seconds: u64,
        arbiter: Option<AccountId>,

        // Security: track used evidence nonces to prevent replay attacks
        used_evidence_nonces: Mapping<(u64, String), bool>, // (property_id, nonce) -> bool
        
        // Per-caller monotonic nonce counter for replay protection (#349)
        // Callers must supply their current nonce; it is incremented on each accepted submit_claim.
        caller_nonces: Mapping<AccountId, u64>,
        
        // Emergency pause mechanism
        is_paused: bool,
        // Time-lock for admin operations
        pending_pause_after: Option<u64>,
        pending_admin: Option<AccountId>,
        pending_admin_after: Option<u64>,
        admin_timelock_delay: u64,

        // Fee tracking
        total_platform_fees_collected: u128,

        // Minimum premium to prevent rounding exploits
        min_premium_amount: u128,
    }

    // =========================================================================
    // IMPLEMENTATION  (extracted to insurance_impl.rs)
    // =========================================================================

    #[ink(event)]
    pub struct PolicyCreated {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        policyholder: AccountId,
        #[ink(topic)]
        property_id: u64,
        coverage_type: CoverageType,
        coverage_amount: u128,
        premium_amount: u128,
        start_time: u64,
        end_time: u64,
    }

    #[ink(event)]
    pub struct PolicyIssued {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        holder: AccountId,
        coverage_amount: u128,
        premium_amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PolicyCancelled {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        policyholder: AccountId,
        cancelled_at: u64,
        reason: Option<String>,
    }

    #[ink(event)]
    pub struct PolicyRenewed {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        holder: AccountId,
        renewal_premium: u128,
        new_end_time: u64,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PolicyExpired {
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        holder: AccountId,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct ClaimSubmitted {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        claimant: AccountId,
        claim_amount: u128,
        submitted_at: u64,
    }

    #[ink(event)]
    pub struct ClaimApproved {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        policy_id: u64,
        payout_amount: u128,
        approved_by: AccountId,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct ClaimRejected {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        policy_id: u64,
        reason: String,
        rejected_by: AccountId,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PayoutExecuted {
        #[ink(topic)]
        claim_id: u64,
        #[ink(topic)]
        recipient: AccountId,
        amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct PoolCapitalized {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct LiquidityDeposited {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct LiquidityWithdrawn {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        principal: u128,
        rewards_paid: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct RewardsReinvested {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        new_stake: u128,
        accumulated_reward_per_share: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct RewardsVestingStarted {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        vesting_start: u64,
        vesting_cliff: u64,
        vesting_duration: u64,
    }

    #[ink(event)]
    pub struct VestedRewardsClaimed {
        #[ink(topic)]
        pool_id: u64,
        #[ink(topic)]
        provider: AccountId,
        amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct ReinsuranceActivated {
        #[ink(topic)]
        claim_id: u64,
        agreement_id: u64,
        recovery_amount: u128,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct InsuranceTokenMinted {
        #[ink(topic)]
        token_id: u64,
        #[ink(topic)]
        policy_id: u64,
        #[ink(topic)]
        owner: AccountId,
        face_value: u128,
    }

    #[ink(event)]
    pub struct InsuranceTokenTransferred {
        #[ink(topic)]
        token_id: u64,
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        price: u128,
    }

    #[ink(event)]
    pub struct RiskAssessmentUpdated {
        #[ink(topic)]
        property_id: u64,
        overall_score: u32,
        risk_level: RiskLevel,
        timestamp: u64,
    }

    #[ink(event)]
    pub struct EvidenceSubmitted {
        #[ink(topic)]
        evidence_id: u64,
        #[ink(topic)]
        raised_by: AccountId,
        dispute_deadline: u64,
        previous_status: ClaimStatus,
        timestamp: u64,
        claim_id: u64,
        evidence_type: String,
        ipfs_hash: String,
        submitter: AccountId,
        submitted_at: u64,
    }

    #[ink(event)]
    pub struct EvidenceVerified {
        #[ink(topic)]
        evidence_id: u64,
        verified_by: AccountId,
        is_valid: bool,
        verified_at: u64,
    }
    
    #[ink(event)]
    pub struct ContractPaused {
        #[ink(topic)]
        paused_by: AccountId,
        timestamp: u64,
    }
    
    #[ink(event)]
    pub struct ContractUnpaused {
        #[ink(topic)]
        unpaused_by: AccountId,
        timestamp: u64,
    }

    /// Emitted when a pause is proposed; executes after time-lock delay (#301)
    #[ink(event)]
    pub struct PauseProposed {
        #[ink(topic)]
        proposed_by: AccountId,
        earliest_execution: u64,
    }

    /// Emitted when a new admin is proposed; executes after time-lock delay (#301)
    #[ink(event)]
    pub struct AdminProposed {
        #[ink(topic)]
        proposed_by: AccountId,
        #[ink(topic)]
        new_admin: AccountId,
        earliest_execution: u64,
    }

    /// Emitted when a pending admin change is executed (#301)
    #[ink(event)]
    pub struct AdminChanged {
        #[ink(topic)]
        old_admin: AccountId,
        #[ink(topic)]
        new_admin: AccountId,
        timestamp: u64,
    }

    /// Emitted when a claim submission is accepted and the caller nonce is consumed (#349)
    #[ink(event)]
    pub struct ReplayProtected {
        #[ink(topic)]
        caller: AccountId,
        nonce: u64,
        claim_id: u64,
    }

    /// Emitted when a role is granted to an account (#346)
    #[ink(event)]
    pub struct RoleGranted {
        #[ink(topic)]
        account: AccountId,
        role: Role,
        granted_by: AccountId,
    }

    /// Emitted when a role is revoked from an account (#346)
    #[ink(event)]
    pub struct RoleRevoked {
        #[ink(topic)]
        account: AccountId,
        role: Role,
        revoked_by: AccountId,
    }

    // =========================================================================
    // IMPLEMENTATION
    // =========================================================================

    // Core contract behavior is extracted to keep the root module focused on types and wiring.
    include!("insurance_impl.rs");

    impl Default for PropertyInsurance {
        /// Build a default instance with a zero admin for tests and tooling.
        fn default() -> Self {
            Self::new(AccountId::from([0x0; 32]))
        }
    }
}

pub use crate::propchain_insurance::{InsuranceError, PropertyInsurance};

#[cfg(test)]
mod insurance_tests {
    include!("insurance_tests.rs");
}
