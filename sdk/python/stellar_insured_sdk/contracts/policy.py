"""Policy contract wrapper with schema-validated payloads."""

from __future__ import annotations

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
    InitializePolicyParams,
    IssuePolicyParams,
    IssuePolicyWithDurationParams,
    CalculateDynamicPremiumParams,
    CreateAmmPoolParams,
    SwapPolicyFractionParams,
    RequestEndorsementParams,
    ApproveEndorsementParams,
    RejectEndorsementParams,
    SetPauseStateParams,
    PolicyRecord,
)


class PolicyContract:
    """Typed wrapper for the Policy Soroban contract."""

    def __init__(self, client: SorobanClient, contract_id: str) -> None:
        self._client = client
        self._contract_id = contract_id

    def initialize(self, admin: str, guardian: str, opts: TxOptions) -> None:
        params = InitializePolicyParams(admin=admin, guardian=guardian)
        self._client.invoke(
            self._contract_id,
            "initialize",
            [to_address(params.admin), to_address(params.guardian)],
            opts,
        )

    def issue_policy(
        self,
        holder: str,
        policy_id: int,
        coverage: int,
        premium: int,
        opts: TxOptions,
    ) -> None:
        params = IssuePolicyParams(
            holder=holder, policy_id=policy_id, coverage=coverage, premium=premium
        )
        self._client.invoke(
            self._contract_id,
            "issue_policy",
            [
                to_address(params.holder),
                to_int128(params.policy_id),
                to_int128(params.coverage),
                to_int128(params.premium),
            ],
            opts,
        )

    def issue_policy_with_duration(
        self,
        holder: str,
        policy_id: int,
        coverage: int,
        premium: int,
        duration_days: int,
        opts: TxOptions,
    ) -> None:
        params = IssuePolicyWithDurationParams(
            holder=holder,
            policy_id=policy_id,
            coverage=coverage,
            premium=premium,
            duration_days=duration_days,
        )
        self._client.invoke(
            self._contract_id,
            "issue_policy_with_duration",
            [
                to_address(params.holder),
                to_int128(params.policy_id),
                to_int128(params.coverage),
                to_int128(params.premium),
                to_uint32(params.duration_days),
            ],
            opts,
        )

    def cancel_policy(self, policy_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id, "cancel_policy", [to_int128(policy_id)], opts
        )

    def expire_policy(self, policy_id: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id, "expire_policy", [to_int128(policy_id)], opts
        )

    def check_and_expire_policies(
        self, start_index: int, max_items: int, opts: TxOptions
    ) -> None:
        self._client.invoke(
            self._contract_id,
            "check_and_expire_policies",
            [to_int128(start_index), to_int128(max_items)],
            opts,
        )

    def is_policy_active(self, policy_id: int, opts: TxOptions) -> bool:
        result = self._client.invoke(
            self._contract_id,
            "is_policy_active",
            [to_int128(policy_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_policy_coverage(self, policy_id: int, opts: TxOptions) -> int:
        result = self._client.invoke(
            self._contract_id,
            "get_policy_coverage",
            [to_int128(policy_id)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def calculate_dynamic_premium(
        self,
        base_premium: int,
        risk_score: int,
        location_factor: int,
        coverage_ratio: int,
        opts: TxOptions,
    ) -> int:
        params = CalculateDynamicPremiumParams(
            base_premium=base_premium,
            risk_score=risk_score,
            location_factor=location_factor,
            coverage_ratio=coverage_ratio,
        )
        result = self._client.invoke(
            self._contract_id,
            "calculate_dynamic_premium",
            [
                to_int128(params.base_premium),
                to_uint32(params.risk_score),
                to_uint32(params.location_factor),
                to_uint32(params.coverage_ratio),
            ],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def create_amm_pool(
        self, policy_id: int, stable_amount: int, opts: TxOptions
    ) -> None:
        params = CreateAmmPoolParams(policy_id=policy_id, stable_amount=stable_amount)
        self._client.invoke(
            self._contract_id,
            "create_amm_pool",
            [to_int128(params.policy_id), to_int128(params.stable_amount)],
            opts,
        )

    def swap_policy_fraction(
        self, buyer: str, policy_id: int, stable_in: int, opts: TxOptions
    ) -> None:
        params = SwapPolicyFractionParams(
            buyer=buyer, policy_id=policy_id, stable_in=stable_in
        )
        self._client.invoke(
            self._contract_id,
            "swap_policy_fraction",
            [
                to_address(params.buyer),
                to_int128(params.policy_id),
                to_int128(params.stable_in),
            ],
            opts,
        )

    def request_endorsement(
        self,
        requester: str,
        policy_id: int,
        new_coverage: int,
        new_premium: int,
        reason: str,
        opts: TxOptions,
    ) -> None:
        params = RequestEndorsementParams(
            requester=requester,
            policy_id=policy_id,
            new_coverage=new_coverage,
            new_premium=new_premium,
            reason=reason,
        )
        self._client.invoke(
            self._contract_id,
            "request_endorsement",
            [
                to_address(params.requester),
                to_int128(params.policy_id),
                to_int128(params.new_coverage),
                to_int128(params.new_premium),
                to_string(params.reason),
            ],
            opts,
        )

    def approve_endorsement(
        self, caller: str, endorsement_id: int, opts: TxOptions
    ) -> None:
        params = ApproveEndorsementParams(
            caller=caller, endorsement_id=endorsement_id
        )
        self._client.invoke(
            self._contract_id,
            "approve_endorsement",
            [to_address(params.caller), to_int128(params.endorsement_id)],
            opts,
        )

    def reject_endorsement(
        self, caller: str, endorsement_id: int, reason: str, opts: TxOptions
    ) -> None:
        params = RejectEndorsementParams(
            caller=caller, endorsement_id=endorsement_id, reason=reason
        )
        self._client.invoke(
            self._contract_id,
            "reject_endorsement",
            [
                to_address(params.caller),
                to_int128(params.endorsement_id),
                to_string(params.reason),
            ],
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
