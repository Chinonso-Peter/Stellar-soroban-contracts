"""Low-level Soroban RPC client shared across contract wrappers."""

from __future__ import annotations

import asyncio
import time
from dataclasses import dataclass, field
from typing import Any

from stellar_sdk import (
    Keypair,
    Network,
    SorobanServer,
    TransactionBuilder,
    scval,
)
from stellar_sdk.soroban_rpc import GetTransactionStatus, SendTransactionStatus


@dataclass(frozen=True)
class SorobanClientConfig:
    rpc_url: str
    network_passphrase: str = Network.TESTNET_NETWORK_PASSPHRASE
    default_fee: int = 100


@dataclass(frozen=True)
class TxOptions:
    source: Keypair
    timeout: int = 30
    simulate: bool = False


class SorobanInvocationError(Exception):
    """Raised when a Soroban contract invocation fails."""

    def __init__(self, method: str, message: str) -> None:
        self.method = method
        super().__init__(f"[{method}] {message}")


class SorobanClient:
    """Handles transaction building, simulation, signing, and submission."""

    def __init__(self, config: SorobanClientConfig) -> None:
        self.server = SorobanServer(config.rpc_url)
        self.network_passphrase = config.network_passphrase
        self.default_fee = config.default_fee

    def invoke(
        self,
        contract_id: str,
        method: str,
        args: list[Any],
        opts: TxOptions,
    ) -> Any | None:
        """Build, simulate, sign, and submit a contract call."""
        account = self.server.load_account(opts.source.public_key)

        tx_builder = (
            TransactionBuilder(
                source_account=account,
                network_passphrase=self.network_passphrase,
                base_fee=self.default_fee,
            )
            .append_invoke_contract_function_op(
                contract_id=contract_id,
                function_name=method,
                parameters=args,
            )
            .set_timeout(opts.timeout)
        )

        tx = tx_builder.build()
        simulated = self.server.simulate_transaction(tx)

        if simulated.error:
            raise SorobanInvocationError(
                method, f"Simulation failed: {simulated.error}"
            )

        if opts.simulate:
            if simulated.results:
                return simulated.results[0].xdr
            return None

        tx = self.server.prepare_transaction(tx)
        tx.sign(opts.source)

        response = self.server.send_transaction(tx)

        if response.status == SendTransactionStatus.ERROR:
            raise SorobanInvocationError(
                method, f"Transaction send failed: {response.status}"
            )

        return self._poll_result(response.hash, method)

    def _poll_result(
        self,
        tx_hash: str,
        method: str,
        max_attempts: int = 30,
        interval: float = 1.0,
    ) -> Any | None:
        """Poll for transaction result until terminal status."""
        for _ in range(max_attempts):
            response = self.server.get_transaction(tx_hash)

            if response.status == GetTransactionStatus.SUCCESS:
                return response.return_value
            if response.status == GetTransactionStatus.FAILED:
                raise SorobanInvocationError(method, "Transaction failed on-chain")

            time.sleep(interval)

        raise SorobanInvocationError(method, "Transaction polling timed out")


# ── ScVal helpers ─────────────────────────────────────────────────────────────


def to_address(addr: str) -> Any:
    return scval.to_address(addr)


def to_int128(val: int) -> Any:
    return scval.to_int128(val)


def to_uint64(val: int) -> Any:
    return scval.to_uint64(val)


def to_uint32(val: int) -> Any:
    return scval.to_uint32(val)


def to_bool(val: bool) -> Any:
    return scval.to_bool(val)


def to_symbol(val: str) -> Any:
    return scval.to_symbol(val)


def to_string(val: str) -> Any:
    return scval.to_string(val)


def from_scval(val: Any) -> Any:
    """Decode a Soroban ScVal into a native Python value."""
    if val is None:
        raise ValueError("Cannot decode None ScVal")
    return scval.from_scval(val)
