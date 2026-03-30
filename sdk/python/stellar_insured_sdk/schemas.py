"""Pydantic models for schema-validated contract payloads and result decoding."""

from __future__ import annotations

from enum import Enum
from typing import Annotated

from pydantic import BaseModel, Field, field_validator

# ── Common validators ─────────────────────────────────────────────────────────

StellarAddress = Annotated[str, Field(min_length=1, pattern=r"^[A-Z0-9]+$")]
PositiveInt = Annotated[int, Field(gt=0)]
NonNegativeInt = Annotated[int, Field(ge=0)]
Percentage = Annotated[int, Field(ge=0, le=100)]
BasisPoints = Annotated[int, Field(ge=0, le=10000)]


# ── Policy ────────────────────────────────────────────────────────────────────


class PolicyStatus(str, Enum):
    ACTIVE = "Active"
    EXPIRED = "Expired"
    CANCELLED = "Cancelled"


class PolicyRecord(BaseModel):
    coverage: PositiveInt
    premium: PositiveInt
    holder: StellarAddress
    status: PolicyStatus
    issue_date: NonNegativeInt
    expiry_date: NonNegativeInt
    total_fractions: NonNegativeInt


class InitializePolicyParams(BaseModel):
    admin: StellarAddress
    guardian: StellarAddress


class IssuePolicyParams(BaseModel):
    holder: StellarAddress
    policy_id: NonNegativeInt
    coverage: PositiveInt
    premium: PositiveInt


class IssuePolicyWithDurationParams(IssuePolicyParams):
    duration_days: PositiveInt


class CalculateDynamicPremiumParams(BaseModel):
    base_premium: PositiveInt
    risk_score: Annotated[int, Field(ge=0, le=100)]
    location_factor: NonNegativeInt
    coverage_ratio: NonNegativeInt


class CreateAmmPoolParams(BaseModel):
    policy_id: NonNegativeInt
    stable_amount: PositiveInt


class SwapPolicyFractionParams(BaseModel):
    buyer: StellarAddress
    policy_id: NonNegativeInt
    stable_in: PositiveInt


class RequestEndorsementParams(BaseModel):
    requester: StellarAddress
    policy_id: NonNegativeInt
    new_coverage: PositiveInt
    new_premium: PositiveInt
    reason: Annotated[str, Field(min_length=1)]


class ApproveEndorsementParams(BaseModel):
    caller: StellarAddress
    endorsement_id: NonNegativeInt


class RejectEndorsementParams(BaseModel):
    caller: StellarAddress
    endorsement_id: NonNegativeInt
    reason: Annotated[str, Field(min_length=1)]


# ── Claims ────────────────────────────────────────────────────────────────────


class ClaimStatus(str, Enum):
    PENDING = "Pending"
    APPROVED = "Approved"
    REJECTED = "Rejected"
    SETTLED = "Settled"


class FraudStatus(str, Enum):
    CLEAN = "Clean"
    SUSPICIOUS = "Suspicious"
    CONFIRMED = "Confirmed"


class PayoutStatus(str, Enum):
    PENDING = "Pending"
    COMPLETED = "Completed"
    FAILED = "Failed"


class ClaimRecord(BaseModel):
    policy_id: NonNegativeInt
    amount: PositiveInt
    status: ClaimStatus
    claimant: StellarAddress
    evidence_count: NonNegativeInt
    fraud_score: NonNegativeInt


class PayoutRecord(BaseModel):
    claim_id: NonNegativeInt
    recipient: StellarAddress
    amount: PositiveInt
    status: PayoutStatus
    retry_count: NonNegativeInt


class SubmitClaimParams(BaseModel):
    policy_address: StellarAddress
    claim_id: NonNegativeInt
    policy_id: NonNegativeInt
    amount: PositiveInt


class SubmitEvidenceParams(BaseModel):
    claim_id: NonNegativeInt
    ipfs_hash: Annotated[str, Field(min_length=1)]
    sensitive: bool
    description: Annotated[str, Field(min_length=1)]
    submitter: StellarAddress


class VerifyEvidenceParams(BaseModel):
    caller: StellarAddress
    evidence_id: NonNegativeInt
    is_valid: bool
    notes: str


class FlagClaimParams(BaseModel):
    caller: StellarAddress
    claim_id: NonNegativeInt
    score_adjustment: int


class SetPayoutTokenParams(BaseModel):
    caller: StellarAddress
    token_address: StellarAddress


# ── Risk Pool ─────────────────────────────────────────────────────────────────


class LiquidityProvider(BaseModel):
    deposited: NonNegativeInt
    allocated_rewards: NonNegativeInt
    claimed_rewards: NonNegativeInt
    vesting_start: NonNegativeInt


class VestingConfig(BaseModel):
    cliff_period: NonNegativeInt
    duration: NonNegativeInt
    early_withdrawal_penalty_bps: BasisPoints


class DepositParams(BaseModel):
    sender: StellarAddress = Field(alias="from")
    amount: PositiveInt


class WithdrawParams(BaseModel):
    to: StellarAddress
    amount: PositiveInt


class SetVestingParams(BaseModel):
    caller: StellarAddress
    cliff_secs: NonNegativeInt
    duration_secs: PositiveInt
    penalty_bps: BasisPoints


class AllocateRewardsParams(BaseModel):
    caller: StellarAddress
    provider: StellarAddress
    amount: PositiveInt


class AppealSlashingParams(BaseModel):
    appealer: StellarAddress
    claim_id: NonNegativeInt
    deposit: PositiveInt
    slashed_amount: PositiveInt


class ResolveAppealParams(BaseModel):
    caller: StellarAddress
    claim_id: NonNegativeInt
    approved: bool
    refund_percentage: Percentage


class SetReinsuranceParams(BaseModel):
    caller: StellarAddress
    reinsurer: StellarAddress
    percentage: Percentage
    credit_score: NonNegativeInt


# ── Governance ────────────────────────────────────────────────────────────────


class ProposalStatus(str, Enum):
    ACTIVE = "Active"
    PASSED = "Passed"
    REJECTED = "Rejected"
    EXECUTED = "Executed"
    COMMITTED = "Committed"


class Proposal(BaseModel):
    id: NonNegativeInt
    proposer: StellarAddress
    title: str
    description: str
    status: ProposalStatus
    voting_deadline: NonNegativeInt


class InitializeGovernanceParams(BaseModel):
    admin: StellarAddress
    token_contract: StellarAddress
    voting_period_days: PositiveInt
    min_voting_percentage: Percentage
    min_quorum_percentage: Percentage
    slashing_contract: StellarAddress


class CreateProposalParams(BaseModel):
    title: Annotated[str, Field(min_length=1, max_length=256)]
    description: Annotated[str, Field(min_length=1)]
    execution_data: str
    threshold_percentage: Percentage


class VoteParams(BaseModel):
    proposal_id: NonNegativeInt
    vote_weight: PositiveInt
    is_yes: bool


class CreateSlashingProposalParams(BaseModel):
    target: StellarAddress
    role: Annotated[str, Field(min_length=1)]
    reason: Annotated[str, Field(min_length=1)]
    amount: PositiveInt
    evidence: str
    threshold: Percentage


# ── Shared ────────────────────────────────────────────────────────────────────


class SetPauseStateParams(BaseModel):
    caller: StellarAddress
    is_paused: bool
    reason: Annotated[str, Field(min_length=1)]
