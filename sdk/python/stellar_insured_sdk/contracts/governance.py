"""Governance contract wrapper with schema-validated payloads."""

from __future__ import annotations

from typing import Any

from ..client import (
    SorobanClient,
    TxOptions,
    to_address,
    to_int128,
    to_uint32,
    to_bool,
    to_string,
    from_scval,
)
from ..schemas import (
    InitializeGovernanceParams,
    CreateProposalParams,
    VoteParams,
    CreateSlashingProposalParams,
)


class GovernanceContract:
    """Typed wrapper for the Governance Soroban contract."""

    def __init__(self, client: SorobanClient, contract_id: str) -> None:
        self._client = client
        self._contract_id = contract_id

    def initialize(
        self,
        admin: str,
        token_contract: str,
        voting_period_days: int,
        min_voting_percentage: int,
        min_quorum_percentage: int,
        slashing_contract: str,
        opts: TxOptions,
    ) -> None:
        params = InitializeGovernanceParams(
            admin=admin,
            token_contract=token_contract,
            voting_period_days=voting_period_days,
            min_voting_percentage=min_voting_percentage,
            min_quorum_percentage=min_quorum_percentage,
            slashing_contract=slashing_contract,
        )
        self._client.invoke(
            self._contract_id,
            "initialize",
            [
                to_address(params.admin),
                to_address(params.token_contract),
                to_uint32(params.voting_period_days),
                to_uint32(params.min_voting_percentage),
                to_uint32(params.min_quorum_percentage),
                to_address(params.slashing_contract),
            ],
            opts,
        )

    def create_proposal(
        self,
        title: str,
        description: str,
        execution_data: str,
        threshold_percentage: int,
        opts: TxOptions,
    ) -> None:
        params = CreateProposalParams(
            title=title,
            description=description,
            execution_data=execution_data,
            threshold_percentage=threshold_percentage,
        )
        self._client.invoke(
            self._contract_id,
            "create_proposal",
            [
                to_string(params.title),
                to_string(params.description),
                to_string(params.execution_data),
                to_uint32(params.threshold_percentage),
            ],
            opts,
        )

    def vote(
        self,
        proposal_id: int,
        vote_weight: int,
        is_yes: bool,
        opts: TxOptions,
    ) -> None:
        params = VoteParams(
            proposal_id=proposal_id, vote_weight=vote_weight, is_yes=is_yes
        )
        self._client.invoke(
            self._contract_id,
            "vote",
            [
                to_int128(params.proposal_id),
                to_int128(params.vote_weight),
                to_bool(params.is_yes),
            ],
            opts,
        )

    def finalize_proposal(self, proposal_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id,
            "finalize_proposal",
            [to_int128(proposal_id)],
            opts,
        )

    def execute_proposal(self, proposal_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id,
            "execute_proposal",
            [to_int128(proposal_id)],
            opts,
        )

    def get_proposal(self, proposal_id: int, opts: TxOptions) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_proposal",
            [to_int128(proposal_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_active_proposals(self, opts: TxOptions) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_active_proposals",
            [],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_all_proposals(self, opts: TxOptions) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_all_proposals",
            [],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_proposal_stats(self, proposal_id: int, opts: TxOptions) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_proposal_stats",
            [to_int128(proposal_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_vote_record(
        self, proposal_id: int, voter: str, opts: TxOptions
    ) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_vote_record",
            [to_int128(proposal_id), to_address(voter)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def create_slashing_proposal(
        self,
        target: str,
        role: str,
        reason: str,
        amount: int,
        evidence: str,
        threshold: int,
        opts: TxOptions,
    ) -> None:
        params = CreateSlashingProposalParams(
            target=target,
            role=role,
            reason=reason,
            amount=amount,
            evidence=evidence,
            threshold=threshold,
        )
        self._client.invoke(
            self._contract_id,
            "create_slashing_proposal",
            [
                to_address(params.target),
                to_string(params.role),
                to_string(params.reason),
                to_int128(params.amount),
                to_string(params.evidence),
                to_uint32(params.threshold),
            ],
            opts,
        )

    def execute_slashing_proposal(
        self, proposal_id: int, opts: TxOptions
    ) -> None:
        self._client.invoke(
            self._contract_id,
            "execute_slashing_proposal",
            [to_int128(proposal_id)],
            opts,
        )
