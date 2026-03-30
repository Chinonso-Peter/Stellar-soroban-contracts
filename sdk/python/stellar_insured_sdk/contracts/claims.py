"""Claims contract wrapper with schema-validated payloads."""

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
    SubmitClaimParams,
    SubmitEvidenceParams,
    VerifyEvidenceParams,
    FlagClaimParams,
    SetPayoutTokenParams,
    SetPauseStateParams,
    PayoutRecord,
)


class ClaimsContract:
    """Typed wrapper for the Claims Soroban contract."""

    def __init__(self, client: SorobanClient, contract_id: str) -> None:
        self._client = client
        self._contract_id = contract_id

    def initialize(self, admin: str, guardian: str, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id,
            "initialize",
            [to_address(admin), to_address(guardian)],
            opts,
        )

    def set_payout_token(
        self, caller: str, token_address: str, opts: TxOptions
    ) -> None:
        params = SetPayoutTokenParams(caller=caller, token_address=token_address)
        self._client.invoke(
            self._contract_id,
            "set_payout_token",
            [to_address(params.caller), to_address(params.token_address)],
            opts,
        )

    def set_max_retries(self, caller: str, max_retries: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id,
            "set_max_retries",
            [to_address(caller), to_uint32(max_retries)],
            opts,
        )

    def submit_claim(
        self,
        policy_address: str,
        claim_id: int,
        policy_id: int,
        amount: int,
        opts: TxOptions,
    ) -> None:
        params = SubmitClaimParams(
            policy_address=policy_address,
            claim_id=claim_id,
            policy_id=policy_id,
            amount=amount,
        )
        self._client.invoke(
            self._contract_id,
            "submit_claim",
            [
                to_address(params.policy_address),
                to_int128(params.claim_id),
                to_int128(params.policy_id),
                to_int128(params.amount),
            ],
            opts,
        )

    def submit_evidence(
        self,
        claim_id: int,
        ipfs_hash: str,
        sensitive: bool,
        description: str,
        submitter: str,
        opts: TxOptions,
    ) -> None:
        params = SubmitEvidenceParams(
            claim_id=claim_id,
            ipfs_hash=ipfs_hash,
            sensitive=sensitive,
            description=description,
            submitter=submitter,
        )
        self._client.invoke(
            self._contract_id,
            "submit_evidence",
            [
                to_int128(params.claim_id),
                to_string(params.ipfs_hash),
                to_bool(params.sensitive),
                to_string(params.description),
                to_address(params.submitter),
            ],
            opts,
        )

    def get_evidence(self, caller: str, evidence_id: int, opts: TxOptions) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_evidence",
            [to_address(caller), to_int128(evidence_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_claim_evidence_ids(self, claim_id: int, opts: TxOptions) -> list[int]:
        result = self._client.invoke(
            self._contract_id,
            "get_claim_evidence_ids",
            [to_int128(claim_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def verify_evidence(
        self,
        caller: str,
        evidence_id: int,
        is_valid: bool,
        notes: str,
        opts: TxOptions,
    ) -> None:
        params = VerifyEvidenceParams(
            caller=caller, evidence_id=evidence_id, is_valid=is_valid, notes=notes
        )
        self._client.invoke(
            self._contract_id,
            "verify_evidence",
            [
                to_address(params.caller),
                to_int128(params.evidence_id),
                to_bool(params.is_valid),
                to_string(params.notes),
            ],
            opts,
        )

    def approve_claim(self, claim_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id, "approve_claim", [to_int128(claim_id)], opts
        )

    def reject_claim(self, claim_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id, "reject_claim", [to_int128(claim_id)], opts
        )

    def settle_claim(self, claim_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id, "settle_claim", [to_int128(claim_id)], opts
        )

    def retry_payout(self, claim_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id, "retry_payout", [to_int128(claim_id)], opts
        )

    def get_payout(self, claim_id: int, opts: TxOptions) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_payout",
            [to_int128(claim_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_payout_status(self, claim_id: int, opts: TxOptions) -> str:
        result = self._client.invoke(
            self._contract_id,
            "get_payout_status",
            [to_int128(claim_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def flag_claim_for_review(
        self, caller: str, claim_id: int, score_adjustment: int, opts: TxOptions
    ) -> None:
        params = FlagClaimParams(
            caller=caller, claim_id=claim_id, score_adjustment=score_adjustment
        )
        self._client.invoke(
            self._contract_id,
            "flag_claim_for_review",
            [
                to_address(params.caller),
                to_int128(params.claim_id),
                to_uint32(params.score_adjustment),
            ],
            opts,
        )

    def report_fraud(self, caller: str, claim_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id,
            "report_fraud",
            [to_address(caller), to_int128(claim_id)],
            opts,
        )

    def set_pause_state(
        self, caller: str, is_paused: bool, reason: str, opts: TxOptions
    ) -> None:
        params = SetPauseStateParams(
            caller=caller, is_paused=is_paused, reason=reason
        )
        self._client.invoke(
            self._contract_id,
            "set_pause_state",
            [
                to_address(params.caller),
                to_bool(params.is_paused),
                to_string(params.reason),
            ],
            opts,
        )
