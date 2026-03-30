import {
  Contract,
  Networks,
  TransactionBuilder,
  Keypair,
  SorobanRpc,
  xdr,
  nativeToScVal,
  scValToNative,
  Address,
  BASE_FEE,
} from "@stellar/stellar-sdk";

export interface SorobanClientConfig {
  rpcUrl: string;
  networkPassphrase?: string;
  defaultFee?: string;
}

export interface TxOptions {
  source: Keypair;
  timeout?: number;
  simulate?: boolean;
}

/**
 * Low-level Soroban RPC client shared across contract wrappers.
 * Handles transaction building, simulation, signing, and submission.
 */
export class SorobanClient {
  readonly server: SorobanRpc.Server;
  readonly networkPassphrase: string;
  readonly defaultFee: string;

  constructor(config: SorobanClientConfig) {
    this.server = new SorobanRpc.Server(config.rpcUrl);
    this.networkPassphrase =
      config.networkPassphrase ?? Networks.TESTNET;
    this.defaultFee = config.defaultFee ?? BASE_FEE;
  }

  /** Build, simulate, sign, and submit a contract invocation. */
  async invoke(
    contractId: string,
    method: string,
    args: xdr.ScVal[],
    opts: TxOptions
  ): Promise<xdr.ScVal | undefined> {
    const account = await this.server.getAccount(
      opts.source.publicKey()
    );
    const contract = new Contract(contractId);

    const tx = new TransactionBuilder(account, {
      fee: this.defaultFee,
      networkPassphrase: this.networkPassphrase,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(opts.timeout ?? 30)
      .build();

    const simulated = await this.server.simulateTransaction(tx);

    if (
      SorobanRpc.Api.isSimulationError(simulated)
    ) {
      throw new SorobanInvocationError(
        method,
        `Simulation failed: ${simulated.error}`
      );
    }

    if (opts.simulate) {
      const result = SorobanRpc.Api.isSimulationSuccess(simulated)
        ? simulated.result?.retval
        : undefined;
      return result;
    }

    const prepared = SorobanRpc.assembleTransaction(
      tx,
      simulated
    ).build();
    prepared.sign(opts.source);

    const sendResponse = await this.server.sendTransaction(prepared);

    if (sendResponse.status === "ERROR") {
      throw new SorobanInvocationError(
        method,
        `Transaction send failed: ${sendResponse.errorResult?.toXDR("base64") ?? "unknown"}`
      );
    }

    return this.pollResult(sendResponse.hash);
  }

  /** Poll for transaction result until terminal status. */
  private async pollResult(
    hash: string,
    maxAttempts = 30,
    intervalMs = 1000
  ): Promise<xdr.ScVal | undefined> {
    for (let i = 0; i < maxAttempts; i++) {
      const response = await this.server.getTransaction(hash);

      if (response.status === "SUCCESS") {
        return response.returnValue;
      }
      if (response.status === "FAILED") {
        throw new SorobanInvocationError(
          hash,
          "Transaction failed on-chain"
        );
      }
      await new Promise((r) => setTimeout(r, intervalMs));
    }
    throw new SorobanInvocationError(
      hash,
      "Transaction polling timed out"
    );
  }
}

export class SorobanInvocationError extends Error {
  constructor(
    public readonly method: string,
    message: string
  ) {
    super(`[${method}] ${message}`);
    this.name = "SorobanInvocationError";
  }
}

// ─── ScVal helpers ───────────────────────────────────────────────────────────

export function toAddress(addr: string): xdr.ScVal {
  return new Address(addr).toScVal();
}

export function toI128(val: bigint): xdr.ScVal {
  return nativeToScVal(val, { type: "i128" });
}

export function toU64(val: bigint): xdr.ScVal {
  return nativeToScVal(val, { type: "u64" });
}

export function toU32(val: number): xdr.ScVal {
  return nativeToScVal(val, { type: "u32" });
}

export function toBool(val: boolean): xdr.ScVal {
  return xdr.ScVal.scvBool(val);
}

export function toSymbol(val: string): xdr.ScVal {
  return xdr.ScVal.scvSymbol(val);
}

export function toString(val: string): xdr.ScVal {
  return nativeToScVal(val, { type: "string" });
}

export function fromScVal<T>(val: xdr.ScVal | undefined): T {
  if (!val) {
    throw new Error("Cannot decode undefined ScVal");
  }
  return scValToNative(val) as T;
}
