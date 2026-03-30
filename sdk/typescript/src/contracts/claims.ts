import {
  SorobanClient,
  TxOptions,
  toAddress,
  toI128,
  toU64,
  toU32,
  toBool,
  toString,
  fromScVal,
} from "../client";
import {
  InitializeClaimsSchema,
  SubmitClaimSchema,
  SubmitEvidenceSchema,
  VerifyEvidenceSchema,
  FlagClaimSchema,
  SetPayoutTokenSchema,
  SetPauseStateSchema,
  type ClaimRecord,
  type PayoutRecord,
} from "../schemas";

export class ClaimsContract {
  constructor(
    private readonly client: SorobanClient,
    private readonly contractId: string
  ) {}

  async initialize(
    admin: string,
    guardian: string,
    opts: TxOptions
  ): Promise<void> {
    const params = InitializeClaimsSchema.parse({ admin, guardian });
    await this.client.invoke(
      this.contractId,
      "initialize",
      [toAddress(params.admin), toAddress(params.guardian)],
      opts
    );
  }

  async setPayoutToken(
    caller: string,
    tokenAddress: string,
    opts: TxOptions
  ): Promise<void> {
    const params = SetPayoutTokenSchema.parse({
      caller,
      token_address: tokenAddress,
    });
    await this.client.invoke(
      this.contractId,
      "set_payout_token",
      [toAddress(params.caller), toAddress(params.token_address)],
      opts
    );
  }

  async setMaxRetries(
    caller: string,
    max: number,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "set_max_retries",
      [toAddress(caller), toU32(max)],
      opts
    );
  }

  async submitClaim(
    policyAddress: string,
    claimId: bigint,
    policyId: bigint,
    amount: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = SubmitClaimSchema.parse({
      policy_address: policyAddress,
      claim_id: claimId,
      policy_id: policyId,
      amount,
    });
    await this.client.invoke(
      this.contractId,
      "submit_claim",
      [
        toAddress(params.policy_address),
        toI128(params.claim_id),
        toI128(params.policy_id),
        toI128(params.amount),
      ],
      opts
    );
  }

  async submitEvidence(
    claimId: bigint,
    ipfsHash: string,
    sensitive: boolean,
    description: string,
    submitter: string,
    opts: TxOptions
  ): Promise<void> {
    const params = SubmitEvidenceSchema.parse({
      claim_id: claimId,
      ipfs_hash: ipfsHash,
      sensitive,
      description,
      submitter,
    });
    await this.client.invoke(
      this.contractId,
      "submit_evidence",
      [
        toI128(params.claim_id),
        toString(params.ipfs_hash),
        toBool(params.sensitive),
        toString(params.description),
        toAddress(params.submitter),
      ],
      opts
    );
  }

  async getEvidence(
    caller: string,
    evidenceId: bigint,
    opts: TxOptions
  ): Promise<unknown> {
    const result = await this.client.invoke(
      this.contractId,
      "get_evidence",
      [toAddress(caller), toI128(evidenceId)],
      { ...opts, simulate: true }
    );
    return fromScVal(result);
  }

  async getClaimEvidenceIds(
    claimId: bigint,
    opts: TxOptions
  ): Promise<bigint[]> {
    const result = await this.client.invoke(
      this.contractId,
      "get_claim_evidence_ids",
      [toI128(claimId)],
      { ...opts, simulate: true }
    );
    return fromScVal<bigint[]>(result);
  }

  async verifyEvidence(
    caller: string,
    evidenceId: bigint,
    isValid: boolean,
    notes: string,
    opts: TxOptions
  ): Promise<void> {
    const params = VerifyEvidenceSchema.parse({
      caller,
      evidence_id: evidenceId,
      is_valid: isValid,
      notes,
    });
    await this.client.invoke(
      this.contractId,
      "verify_evidence",
      [
        toAddress(params.caller),
        toI128(params.evidence_id),
        toBool(params.is_valid),
        toString(params.notes),
      ],
      opts
    );
  }

  async approveClaim(
    claimId: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "approve_claim",
      [toI128(claimId)],
      opts
    );
  }

  async rejectClaim(
    claimId: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "reject_claim",
      [toI128(claimId)],
      opts
    );
  }

  async settleClaim(
    claimId: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "settle_claim",
      [toI128(claimId)],
      opts
    );
  }

  async retryPayout(
    claimId: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "retry_payout",
      [toI128(claimId)],
      opts
    );
  }

  async getPayout(
    claimId: bigint,
    opts: TxOptions
  ): Promise<PayoutRecord> {
    const result = await this.client.invoke(
      this.contractId,
      "get_payout",
      [toI128(claimId)],
      { ...opts, simulate: true }
    );
    return fromScVal<PayoutRecord>(result);
  }

  async getPayoutStatus(
    claimId: bigint,
    opts: TxOptions
  ): Promise<string> {
    const result = await this.client.invoke(
      this.contractId,
      "get_payout_status",
      [toI128(claimId)],
      { ...opts, simulate: true }
    );
    return fromScVal<string>(result);
  }

  async flagClaimForReview(
    caller: string,
    claimId: bigint,
    scoreAdjustment: number,
    opts: TxOptions
  ): Promise<void> {
    const params = FlagClaimSchema.parse({
      caller,
      claim_id: claimId,
      score_adjustment: scoreAdjustment,
    });
    await this.client.invoke(
      this.contractId,
      "flag_claim_for_review",
      [
        toAddress(params.caller),
        toI128(params.claim_id),
        toU32(params.score_adjustment),
      ],
      opts
    );
  }

  async reportFraud(
    caller: string,
    claimId: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "report_fraud",
      [toAddress(caller), toI128(claimId)],
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
