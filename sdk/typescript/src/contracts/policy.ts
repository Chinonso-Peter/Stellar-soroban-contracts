import { Keypair } from "@stellar/stellar-sdk";
import {
  SorobanClient,
  TxOptions,
  toAddress,
  toI128,
  toU64,
  toU32,
  toString,
  toBool,
  fromScVal,
} from "../client";
import {
  InitializePolicySchema,
  IssuePolicySchema,
  IssuePolicyWithDurationSchema,
  CancelPolicySchema,
  ExpirePolicySchema,
  CheckExpirePoliciesSchema,
  CalculateDynamicPremiumSchema,
  CreateAmmPoolSchema,
  SwapPolicyFractionSchema,
  RequestEndorsementSchema,
  ApproveEndorsementSchema,
  RejectEndorsementSchema,
  SetPauseStateSchema,
  type PolicyRecord,
} from "../schemas";

export class PolicyContract {
  constructor(
    private readonly client: SorobanClient,
    private readonly contractId: string
  ) {}

  async initialize(
    admin: string,
    guardian: string,
    opts: TxOptions
  ): Promise<void> {
    const params = InitializePolicySchema.parse({ admin, guardian });
    await this.client.invoke(
      this.contractId,
      "initialize",
      [toAddress(params.admin), toAddress(params.guardian)],
      opts
    );
  }

  async issuePolicy(
    holder: string,
    policyId: bigint,
    coverage: bigint,
    premium: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = IssuePolicySchema.parse({
      holder,
      policy_id: policyId,
      coverage,
      premium,
    });
    await this.client.invoke(
      this.contractId,
      "issue_policy",
      [
        toAddress(params.holder),
        toI128(params.policy_id),
        toI128(params.coverage),
        toI128(params.premium),
      ],
      opts
    );
  }

  async issuePolicyWithDuration(
    holder: string,
    policyId: bigint,
    coverage: bigint,
    premium: bigint,
    durationDays: number,
    opts: TxOptions
  ): Promise<void> {
    const params = IssuePolicyWithDurationSchema.parse({
      holder,
      policy_id: policyId,
      coverage,
      premium,
      duration_days: durationDays,
    });
    await this.client.invoke(
      this.contractId,
      "issue_policy_with_duration",
      [
        toAddress(params.holder),
        toI128(params.policy_id),
        toI128(params.coverage),
        toI128(params.premium),
        toU32(params.duration_days),
      ],
      opts
    );
  }

  async cancelPolicy(
    policyId: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = CancelPolicySchema.parse({ policy_id: policyId });
    await this.client.invoke(
      this.contractId,
      "cancel_policy",
      [toI128(params.policy_id)],
      opts
    );
  }

  async expirePolicy(
    policyId: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = ExpirePolicySchema.parse({ policy_id: policyId });
    await this.client.invoke(
      this.contractId,
      "expire_policy",
      [toI128(params.policy_id)],
      opts
    );
  }

  async checkAndExpirePolicies(
    startIndex: bigint,
    maxItems: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = CheckExpirePoliciesSchema.parse({
      start_index: startIndex,
      max_items: maxItems,
    });
    await this.client.invoke(
      this.contractId,
      "check_and_expire_policies",
      [toI128(params.start_index), toI128(params.max_items)],
      opts
    );
  }

  async isPolicyActive(
    policyId: bigint,
    opts: TxOptions
  ): Promise<boolean> {
    const result = await this.client.invoke(
      this.contractId,
      "is_policy_active",
      [toI128(policyId)],
      { ...opts, simulate: true }
    );
    return fromScVal<boolean>(result);
  }

  async getPolicyCoverage(
    policyId: bigint,
    opts: TxOptions
  ): Promise<bigint> {
    const result = await this.client.invoke(
      this.contractId,
      "get_policy_coverage",
      [toI128(policyId)],
      { ...opts, simulate: true }
    );
    return fromScVal<bigint>(result);
  }

  async calculateDynamicPremium(
    basePremium: bigint,
    riskScore: number,
    locationFactor: number,
    coverageRatio: number,
    opts: TxOptions
  ): Promise<bigint> {
    const params = CalculateDynamicPremiumSchema.parse({
      base_premium: basePremium,
      risk_score: riskScore,
      location_factor: locationFactor,
      coverage_ratio: coverageRatio,
    });
    const result = await this.client.invoke(
      this.contractId,
      "calculate_dynamic_premium",
      [
        toI128(params.base_premium),
        toU32(params.risk_score),
        toU32(params.location_factor),
        toU32(params.coverage_ratio),
      ],
      { ...opts, simulate: true }
    );
    return fromScVal<bigint>(result);
  }

  async createAmmPool(
    policyId: bigint,
    stableAmount: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = CreateAmmPoolSchema.parse({
      policy_id: policyId,
      stable_amount: stableAmount,
    });
    await this.client.invoke(
      this.contractId,
      "create_amm_pool",
      [toI128(params.policy_id), toI128(params.stable_amount)],
      opts
    );
  }

  async swapPolicyFraction(
    buyer: string,
    policyId: bigint,
    stableIn: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = SwapPolicyFractionSchema.parse({
      buyer,
      policy_id: policyId,
      stable_in: stableIn,
    });
    await this.client.invoke(
      this.contractId,
      "swap_policy_fraction",
      [
        toAddress(params.buyer),
        toI128(params.policy_id),
        toI128(params.stable_in),
      ],
      opts
    );
  }

  async requestEndorsement(
    requester: string,
    policyId: bigint,
    newCoverage: bigint,
    newPremium: bigint,
    reason: string,
    opts: TxOptions
  ): Promise<void> {
    const params = RequestEndorsementSchema.parse({
      requester,
      policy_id: policyId,
      new_coverage: newCoverage,
      new_premium: newPremium,
      reason,
    });
    await this.client.invoke(
      this.contractId,
      "request_endorsement",
      [
        toAddress(params.requester),
        toI128(params.policy_id),
        toI128(params.new_coverage),
        toI128(params.new_premium),
        toString(params.reason),
      ],
      opts
    );
  }

  async approveEndorsement(
    caller: string,
    endorsementId: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = ApproveEndorsementSchema.parse({
      caller,
      endorsement_id: endorsementId,
    });
    await this.client.invoke(
      this.contractId,
      "approve_endorsement",
      [toAddress(params.caller), toI128(params.endorsement_id)],
      opts
    );
  }

  async rejectEndorsement(
    caller: string,
    endorsementId: bigint,
    reason: string,
    opts: TxOptions
  ): Promise<void> {
    const params = RejectEndorsementSchema.parse({
      caller,
      endorsement_id: endorsementId,
      reason,
    });
    await this.client.invoke(
      this.contractId,
      "reject_endorsement",
      [
        toAddress(params.caller),
        toI128(params.endorsement_id),
        toString(params.reason),
      ],
      opts
    );
  }

  async setPauseState(
    caller: string,
    isPaused: boolean,
    reason: string,
    opts: TxOptions
  ): Promise<void> {
    const params = SetPauseStateSchema.parse({
      caller,
      is_paused: isPaused,
      reason,
    });
    await this.client.invoke(
      this.contractId,
      "set_pause_state",
      [toAddress(params.caller), toBool(params.is_paused), toString(params.reason)],
      opts
    );
  }
}
