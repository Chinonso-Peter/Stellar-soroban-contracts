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
  InitializeRiskPoolSchema,
  DepositSchema,
  WithdrawSchema,
  SetVestingSchema,
  AllocateRewardsSchema,
  AppealSlashingSchema,
  ResolveAppealSchema,
  SetReinsuranceSchema,
  SetPauseStateSchema,
  type LiquidityProvider,
  type VestingConfig,
} from "../schemas";

export class RiskPoolContract {
  constructor(
    private readonly client: SorobanClient,
    private readonly contractId: string
  ) {}

  async initialize(
    admin: string,
    guardian: string,
    opts: TxOptions
  ): Promise<void> {
    const params = InitializeRiskPoolSchema.parse({ admin, guardian });
    await this.client.invoke(
      this.contractId,
      "initialize",
      [toAddress(params.admin), toAddress(params.guardian)],
      opts
    );
  }

  async deposit(
    from: string,
    amount: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = DepositSchema.parse({ from, amount });
    await this.client.invoke(
      this.contractId,
      "deposit",
      [toAddress(params.from), toI128(params.amount)],
      opts
    );
  }

  async withdraw(
    to: string,
    amount: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = WithdrawSchema.parse({ to, amount });
    await this.client.invoke(
      this.contractId,
      "withdraw",
      [toAddress(params.to), toI128(params.amount)],
      opts
    );
  }

  async setVestingParameters(
    caller: string,
    cliffSecs: bigint,
    durationSecs: bigint,
    penaltyBps: number,
    opts: TxOptions
  ): Promise<void> {
    const params = SetVestingSchema.parse({
      caller,
      cliff_secs: cliffSecs,
      duration_secs: durationSecs,
      penalty_bps: penaltyBps,
    });
    await this.client.invoke(
      this.contractId,
      "set_vesting_parameters",
      [
        toAddress(params.caller),
        toU64(params.cliff_secs),
        toU64(params.duration_secs),
        toU32(params.penalty_bps),
      ],
      opts
    );
  }

  async getVestingParameters(opts: TxOptions): Promise<VestingConfig> {
    const result = await this.client.invoke(
      this.contractId,
      "get_vesting_parameters",
      [],
      { ...opts, simulate: true }
    );
    return fromScVal<VestingConfig>(result);
  }

  async allocateRewards(
    caller: string,
    provider: string,
    amount: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = AllocateRewardsSchema.parse({
      caller,
      provider,
      amount,
    });
    await this.client.invoke(
      this.contractId,
      "allocate_rewards",
      [
        toAddress(params.caller),
        toAddress(params.provider),
        toI128(params.amount),
      ],
      opts
    );
  }

  async claimVestedRewards(
    provider: string,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "claim_vested_rewards",
      [toAddress(provider)],
      opts
    );
  }

  async getProviderVestedRewards(
    provider: string,
    opts: TxOptions
  ): Promise<bigint> {
    const result = await this.client.invoke(
      this.contractId,
      "get_provider_vested_rewards",
      [toAddress(provider)],
      { ...opts, simulate: true }
    );
    return fromScVal<bigint>(result);
  }

  async getBalance(opts: TxOptions): Promise<bigint> {
    const result = await this.client.invoke(
      this.contractId,
      "get_balance",
      [],
      { ...opts, simulate: true }
    );
    return fromScVal<bigint>(result);
  }

  async appealSlashing(
    appealer: string,
    claimId: bigint,
    deposit: bigint,
    slashedAmount: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = AppealSlashingSchema.parse({
      appealer,
      claim_id: claimId,
      deposit,
      slashed_amount: slashedAmount,
    });
    await this.client.invoke(
      this.contractId,
      "appeal_slashing",
      [
        toAddress(params.appealer),
        toI128(params.claim_id),
        toI128(params.deposit),
        toI128(params.slashed_amount),
      ],
      opts
    );
  }

  async resolveAppeal(
    caller: string,
    claimId: bigint,
    approved: boolean,
    refundPercentage: number,
    opts: TxOptions
  ): Promise<void> {
    const params = ResolveAppealSchema.parse({
      caller,
      claim_id: claimId,
      approved,
      refund_percentage: refundPercentage,
    });
    await this.client.invoke(
      this.contractId,
      "resolve_appeal",
      [
        toAddress(params.caller),
        toI128(params.claim_id),
        toBool(params.approved),
        toU32(params.refund_percentage),
      ],
      opts
    );
  }

  async setReinsuranceConfig(
    caller: string,
    reinsurer: string,
    percentage: number,
    creditScore: number,
    opts: TxOptions
  ): Promise<void> {
    const params = SetReinsuranceSchema.parse({
      caller,
      reinsurer,
      percentage,
      credit_score: creditScore,
    });
    await this.client.invoke(
      this.contractId,
      "set_reinsurance_config",
      [
        toAddress(params.caller),
        toAddress(params.reinsurer),
        toU32(params.percentage),
        toU32(params.credit_score),
      ],
      opts
    );
  }

  async cedeRisk(
    reinsurer: string,
    amount: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "cede_risk",
      [toAddress(reinsurer), toI128(amount)],
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
