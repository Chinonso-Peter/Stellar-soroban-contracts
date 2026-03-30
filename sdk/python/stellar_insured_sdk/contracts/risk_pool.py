"""Risk Pool contract wrapper with schema-validated payloads."""

from __future__ import annotations

from typing import Any

from ..client import (
    SorobanClient,
    TxOptions,
    to_address,
    to_int128,
    to_uint64,
    to_uint32,
    to_bool,
    to_string,
    from_scval,
)
from ..schemas import (
    DepositParams,
    WithdrawParams,
    SetVestingParams,
    AllocateRewardsParams,
    AppealSlashingParams,
    ResolveAppealParams,
    SetReinsuranceParams,
    SetPauseStateParams,
)


class RiskPoolContract:
    """Typed wrapper for the Risk Pool Soroban contract."""

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

    def deposit(self, sender: str, amount: int, opts: TxOptions) -> None:
        params = DepositParams(**{"from": sender, "amount": amount})
        self._client.invoke(
            self._contract_id,
            "deposit",
            [to_address(params.sender), to_int128(params.amount)],
            opts,
        )

    def withdraw(self, to: str, amount: int, opts: TxOptions) -> None:
        params = WithdrawParams(to=to, amount=amount)
        self._client.invoke(
            self._contract_id,
            "withdraw",
            [to_address(params.to), to_int128(params.amount)],
            opts,
        )

    def set_vesting_parameters(
        self,
        caller: str,
        cliff_secs: int,
        duration_secs: int,
        penalty_bps: int,
        opts: TxOptions,
    ) -> None:
        params = SetVestingParams(
            caller=caller,
            cliff_secs=cliff_secs,
            duration_secs=duration_secs,
            penalty_bps=penalty_bps,
        )
        self._client.invoke(
            self._contract_id,
            "set_vesting_parameters",
            [
                to_address(params.caller),
                to_uint64(params.cliff_secs),
                to_uint64(params.duration_secs),
                to_uint32(params.penalty_bps),
            ],
            opts,
        )

    def get_vesting_parameters(self, opts: TxOptions) -> Any:
        result = self._client.invoke(
            self._contract_id,
            "get_vesting_parameters",
            [],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def allocate_rewards(
        self, caller: str, provider: str, amount: int, opts: TxOptions
    ) -> None:
        params = AllocateRewardsParams(caller=caller, provider=provider, amount=amount)
        self._client.invoke(
            self._contract_id,
            "allocate_rewards",
            [
                to_address(params.caller),
                to_address(params.provider),
                to_int128(params.amount),
            ],
            opts,
        )

    def claim_vested_rewards(self, provider: str, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id,
            "claim_vested_rewards",
            [to_address(provider)],
            opts,
        )

    def get_provider_vested_rewards(self, provider: str, opts: TxOptions) -> int:
        result = self._client.invoke(
            self._contract_id,
            "get_provider_vested_rewards",
            [to_address(provider)],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def get_balance(self, opts: TxOptions) -> int:
        result = self._client.invoke(
            self._contract_id,
            "get_balance",
            [],
            TxOptions(source=opts.source, timeout=opts.timeout, simulate=True),
        )
        return from_scval(result)

    def appeal_slashing(
        self,
        appealer: str,
        claim_id: int,
        deposit: int,
        slashed_amount: int,
        opts: TxOptions,
    ) -> None:
        params = AppealSlashingParams(
            appealer=appealer,
            claim_id=claim_id,
            deposit=deposit,
            slashed_amount=slashed_amount,
        )
        self._client.invoke(
            self._contract_id,
            "appeal_slashing",
            [
                to_address(params.appealer),
                to_int128(params.claim_id),
                to_int128(params.deposit),
                to_int128(params.slashed_amount),
            ],
            opts,
        )

    def resolve_appeal(
        self,
        caller: str,
        claim_id: int,
        approved: bool,
        refund_percentage: int,
        opts: TxOptions,
    ) -> None:
        params = ResolveAppealParams(
            caller=caller,
            claim_id=claim_id,
            approved=approved,
            refund_percentage=refund_percentage,
        )
        self._client.invoke(
            self._contract_id,
            "resolve_appeal",
            [
                to_address(params.caller),
                to_int128(params.claim_id),
                to_bool(params.approved),
                to_uint32(params.refund_percentage),
            ],
            opts,
        )

    def set_reinsurance_config(
        self,
        caller: str,
        reinsurer: str,
        percentage: int,
        credit_score: int,
        opts: TxOptions,
    ) -> None:
        params = SetReinsuranceParams(
            caller=caller,
            reinsurer=reinsurer,
            percentage=percentage,
            credit_score=credit_score,
        )
        self._client.invoke(
            self._contract_id,
            "set_reinsurance_config",
            [
                to_address(params.caller),
                to_address(params.reinsurer),
                to_uint32(params.percentage),
                to_uint32(params.credit_score),
            ],
            opts,
        )

    def cede_risk(self, reinsurer: str, amount: int, opts: TxOptions) -> None:
        self._client.invoke(
            self._contract_id,
            "cede_risk",
            [to_address(reinsurer), to_int128(amount)],
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
