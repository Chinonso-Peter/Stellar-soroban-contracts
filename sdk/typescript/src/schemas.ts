import { z } from "zod";

// ─── Common ──────────────────────────────────────────────────────────────────

const SorobanAddress = z
  .string()
  .min(1, "Address must not be empty")
  .regex(/^[A-Z0-9]+$/, "Invalid Stellar address format");

const PositiveU64 = z.coerce.bigint().refine((v) => v > 0n, {
  message: "Value must be a positive integer",
});

const NonNegativeU64 = z.coerce.bigint().refine((v) => v >= 0n, {
  message: "Value must be non-negative",
});

const PolicyId = z.coerce.bigint().refine((v) => v >= 0n, {
  message: "Policy ID must be non-negative",
});

const ClaimId = z.coerce.bigint().refine((v) => v >= 0n, {
  message: "Claim ID must be non-negative",
});

// ─── Policy Contract ─────────────────────────────────────────────────────────

export const PolicyStatusSchema = z.enum([
  "Active",
  "Expired",
  "Cancelled",
]);

export const PolicyRecordSchema = z.object({
  coverage: PositiveU64,
  premium: PositiveU64,
  holder: SorobanAddress,
  status: PolicyStatusSchema,
  issue_date: NonNegativeU64,
  expiry_date: NonNegativeU64,
  total_fractions: NonNegativeU64,
});

export const InitializePolicySchema = z.object({
  admin: SorobanAddress,
  guardian: SorobanAddress,
});

export const IssuePolicySchema = z.object({
  holder: SorobanAddress,
  policy_id: PolicyId,
  coverage: PositiveU64,
  premium: PositiveU64,
});

export const IssuePolicyWithDurationSchema = IssuePolicySchema.extend({
  duration_days: z.number().int().positive("Duration must be positive"),
});

export const CancelPolicySchema = z.object({
  policy_id: PolicyId,
});

export const ExpirePolicySchema = z.object({
  policy_id: PolicyId,
});

export const CheckExpirePoliciesSchema = z.object({
  start_index: NonNegativeU64,
  max_items: PositiveU64,
});

export const CalculateDynamicPremiumSchema = z.object({
  base_premium: PositiveU64,
  risk_score: z.number().int().min(0).max(100),
  location_factor: z.number().int().min(0),
  coverage_ratio: z.number().int().min(0),
});

export const CreateAmmPoolSchema = z.object({
  policy_id: PolicyId,
  stable_amount: PositiveU64,
});

export const SwapPolicyFractionSchema = z.object({
  buyer: SorobanAddress,
  policy_id: PolicyId,
  stable_in: PositiveU64,
});

export const RequestEndorsementSchema = z.object({
  requester: SorobanAddress,
  policy_id: PolicyId,
  new_coverage: PositiveU64,
  new_premium: PositiveU64,
  reason: z.string().min(1, "Reason must not be empty"),
});

export const ApproveEndorsementSchema = z.object({
  caller: SorobanAddress,
  endorsement_id: NonNegativeU64,
});

export const RejectEndorsementSchema = z.object({
  caller: SorobanAddress,
  endorsement_id: NonNegativeU64,
  reason: z.string().min(1, "Reason must not be empty"),
});

// ─── Claims Contract ─────────────────────────────────────────────────────────

export const ClaimStatusSchema = z.enum([
  "Pending",
  "Approved",
  "Rejected",
  "Settled",
]);

export const FraudStatusSchema = z.enum([
  "Clean",
  "Suspicious",
  "Confirmed",
]);

export const PayoutStatusSchema = z.enum([
  "Pending",
  "Completed",
  "Failed",
]);

export const ClaimRecordSchema = z.object({
  policy_id: PolicyId,
  amount: PositiveU64,
  status: ClaimStatusSchema,
  claimant: SorobanAddress,
  evidence_count: NonNegativeU64,
  fraud_score: z.number().int().min(0),
});

export const PayoutRecordSchema = z.object({
  claim_id: ClaimId,
  recipient: SorobanAddress,
  amount: PositiveU64,
  status: PayoutStatusSchema,
  retry_count: z.number().int().min(0),
});

export const InitializeClaimsSchema = z.object({
  admin: SorobanAddress,
  guardian: SorobanAddress,
});

export const SubmitClaimSchema = z.object({
  policy_address: SorobanAddress,
  claim_id: ClaimId,
  policy_id: PolicyId,
  amount: PositiveU64,
});

export const SubmitEvidenceSchema = z.object({
  claim_id: ClaimId,
  ipfs_hash: z.string().min(1, "IPFS hash required"),
  sensitive: z.boolean(),
  description: z.string().min(1, "Description required"),
  submitter: SorobanAddress,
});

export const VerifyEvidenceSchema = z.object({
  caller: SorobanAddress,
  evidence_id: NonNegativeU64,
  is_valid: z.boolean(),
  notes: z.string(),
});

export const FlagClaimSchema = z.object({
  caller: SorobanAddress,
  claim_id: ClaimId,
  score_adjustment: z.number().int(),
});

export const SetPayoutTokenSchema = z.object({
  caller: SorobanAddress,
  token_address: SorobanAddress,
});

// ─── Risk Pool Contract ──────────────────────────────────────────────────────

export const LiquidityProviderSchema = z.object({
  deposited: NonNegativeU64,
  allocated_rewards: NonNegativeU64,
  claimed_rewards: NonNegativeU64,
  vesting_start: NonNegativeU64,
});

export const VestingConfigSchema = z.object({
  cliff_period: NonNegativeU64,
  duration: NonNegativeU64,
  early_withdrawal_penalty_bps: z.number().int().min(0).max(10000),
});

export const InitializeRiskPoolSchema = z.object({
  admin: SorobanAddress,
  guardian: SorobanAddress,
});

export const DepositSchema = z.object({
  from: SorobanAddress,
  amount: PositiveU64,
});

export const WithdrawSchema = z.object({
  to: SorobanAddress,
  amount: PositiveU64,
});

export const SetVestingSchema = z.object({
  caller: SorobanAddress,
  cliff_secs: NonNegativeU64,
  duration_secs: PositiveU64,
  penalty_bps: z.number().int().min(0).max(10000),
});

export const AllocateRewardsSchema = z.object({
  caller: SorobanAddress,
  provider: SorobanAddress,
  amount: PositiveU64,
});

export const AppealSlashingSchema = z.object({
  appealer: SorobanAddress,
  claim_id: ClaimId,
  deposit: PositiveU64,
  slashed_amount: PositiveU64,
});

export const ResolveAppealSchema = z.object({
  caller: SorobanAddress,
  claim_id: ClaimId,
  approved: z.boolean(),
  refund_percentage: z.number().int().min(0).max(100),
});

export const SetReinsuranceSchema = z.object({
  caller: SorobanAddress,
  reinsurer: SorobanAddress,
  percentage: z.number().int().min(0).max(100),
  credit_score: z.number().int().min(0),
});

// ─── Governance Contract ─────────────────────────────────────────────────────

export const ProposalStatusSchema = z.enum([
  "Active",
  "Passed",
  "Rejected",
  "Executed",
  "Committed",
]);

export const ProposalSchema = z.object({
  id: NonNegativeU64,
  proposer: SorobanAddress,
  title: z.string(),
  description: z.string(),
  status: ProposalStatusSchema,
  voting_deadline: NonNegativeU64,
});

export const InitializeGovernanceSchema = z.object({
  admin: SorobanAddress,
  token_contract: SorobanAddress,
  voting_period_days: z.number().int().positive(),
  min_voting_percentage: z.number().int().min(0).max(100),
  min_quorum_percentage: z.number().int().min(0).max(100),
  slashing_contract: SorobanAddress,
});

export const CreateProposalSchema = z.object({
  title: z.string().min(1, "Title required").max(256, "Title too long"),
  description: z.string().min(1, "Description required"),
  execution_data: z.string(),
  threshold_percentage: z.number().int().min(0).max(100),
});

export const VoteSchema = z.object({
  proposal_id: NonNegativeU64,
  vote_weight: PositiveU64,
  is_yes: z.boolean(),
});

export const FinalizeProposalSchema = z.object({
  proposal_id: NonNegativeU64,
});

export const CreateSlashingProposalSchema = z.object({
  target: SorobanAddress,
  role: z.string().min(1),
  reason: z.string().min(1),
  amount: PositiveU64,
  evidence: z.string(),
  threshold: z.number().int().min(0).max(100),
});

// ─── Pause State (shared) ────────────────────────────────────────────────────

export const SetPauseStateSchema = z.object({
  caller: SorobanAddress,
  is_paused: z.boolean(),
  reason: z.string().min(1, "Reason required"),
});

// ─── Type exports ────────────────────────────────────────────────────────────

export type PolicyRecord = z.infer<typeof PolicyRecordSchema>;
export type ClaimRecord = z.infer<typeof ClaimRecordSchema>;
export type PayoutRecord = z.infer<typeof PayoutRecordSchema>;
export type LiquidityProvider = z.infer<typeof LiquidityProviderSchema>;
export type VestingConfig = z.infer<typeof VestingConfigSchema>;
export type Proposal = z.infer<typeof ProposalSchema>;
