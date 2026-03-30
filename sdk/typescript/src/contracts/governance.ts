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
  InitializeGovernanceSchema,
  CreateProposalSchema,
  VoteSchema,
  FinalizeProposalSchema,
  CreateSlashingProposalSchema,
  type Proposal,
} from "../schemas";

export class GovernanceContract {
  constructor(
    private readonly client: SorobanClient,
    private readonly contractId: string
  ) {}

  async initialize(
    admin: string,
    tokenContract: string,
    votingPeriodDays: number,
    minVotingPercentage: number,
    minQuorumPercentage: number,
    slashingContract: string,
    opts: TxOptions
  ): Promise<void> {
    const params = InitializeGovernanceSchema.parse({
      admin,
      token_contract: tokenContract,
      voting_period_days: votingPeriodDays,
      min_voting_percentage: minVotingPercentage,
      min_quorum_percentage: minQuorumPercentage,
      slashing_contract: slashingContract,
    });
    await this.client.invoke(
      this.contractId,
      "initialize",
      [
        toAddress(params.admin),
        toAddress(params.token_contract),
        toU32(params.voting_period_days),
        toU32(params.min_voting_percentage),
        toU32(params.min_quorum_percentage),
        toAddress(params.slashing_contract),
      ],
      opts
    );
  }

  async createProposal(
    title: string,
    description: string,
    executionData: string,
    thresholdPercentage: number,
    opts: TxOptions
  ): Promise<void> {
    const params = CreateProposalSchema.parse({
      title,
      description,
      execution_data: executionData,
      threshold_percentage: thresholdPercentage,
    });
    await this.client.invoke(
      this.contractId,
      "create_proposal",
      [
        toString(params.title),
        toString(params.description),
        toString(params.execution_data),
        toU32(params.threshold_percentage),
      ],
      opts
    );
  }

  async vote(
    proposalId: bigint,
    voteWeight: bigint,
    isYes: boolean,
    opts: TxOptions
  ): Promise<void> {
    const params = VoteSchema.parse({
      proposal_id: proposalId,
      vote_weight: voteWeight,
      is_yes: isYes,
    });
    await this.client.invoke(
      this.contractId,
      "vote",
      [toI128(params.proposal_id), toI128(params.vote_weight), toBool(params.is_yes)],
      opts
    );
  }

  async finalizeProposal(
    proposalId: bigint,
    opts: TxOptions
  ): Promise<void> {
    const params = FinalizeProposalSchema.parse({
      proposal_id: proposalId,
    });
    await this.client.invoke(
      this.contractId,
      "finalize_proposal",
      [toI128(params.proposal_id)],
      opts
    );
  }

  async executeProposal(
    proposalId: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "execute_proposal",
      [toI128(proposalId)],
      opts
    );
  }

  async getProposal(
    proposalId: bigint,
    opts: TxOptions
  ): Promise<Proposal> {
    const result = await this.client.invoke(
      this.contractId,
      "get_proposal",
      [toI128(proposalId)],
      { ...opts, simulate: true }
    );
    return fromScVal<Proposal>(result);
  }

  async getActiveProposals(opts: TxOptions): Promise<Proposal[]> {
    const result = await this.client.invoke(
      this.contractId,
      "get_active_proposals",
      [],
      { ...opts, simulate: true }
    );
    return fromScVal<Proposal[]>(result);
  }

  async getAllProposals(opts: TxOptions): Promise<Proposal[]> {
    const result = await this.client.invoke(
      this.contractId,
      "get_all_proposals",
      [],
      { ...opts, simulate: true }
    );
    return fromScVal<Proposal[]>(result);
  }

  async getProposalStats(
    proposalId: bigint,
    opts: TxOptions
  ): Promise<unknown> {
    const result = await this.client.invoke(
      this.contractId,
      "get_proposal_stats",
      [toI128(proposalId)],
      { ...opts, simulate: true }
    );
    return fromScVal(result);
  }

  async getVoteRecord(
    proposalId: bigint,
    voter: string,
    opts: TxOptions
  ): Promise<unknown> {
    const result = await this.client.invoke(
      this.contractId,
      "get_vote_record",
      [toI128(proposalId), toAddress(voter)],
      { ...opts, simulate: true }
    );
    return fromScVal(result);
  }

  async createSlashingProposal(
    target: string,
    role: string,
    reason: string,
    amount: bigint,
    evidence: string,
    threshold: number,
    opts: TxOptions
  ): Promise<void> {
    const params = CreateSlashingProposalSchema.parse({
      target,
      role,
      reason,
      amount,
      evidence,
      threshold,
    });
    await this.client.invoke(
      this.contractId,
      "create_slashing_proposal",
      [
        toAddress(params.target),
        toString(params.role),
        toString(params.reason),
        toI128(params.amount),
        toString(params.evidence),
        toU32(params.threshold),
      ],
      opts
    );
  }

  async executeSlashingProposal(
    proposalId: bigint,
    opts: TxOptions
  ): Promise<void> {
    await this.client.invoke(
      this.contractId,
      "execute_slashing_proposal",
      [toI128(proposalId)],
      opts
    );
  }
}
