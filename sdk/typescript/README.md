# @stellar-insured/sdk — TypeScript

Schema-validated TypeScript SDK for the Stellar Insured Soroban contracts. Wraps all core contract methods (Policy, Claims, Risk Pool, Governance) with **Zod validation** on every payload and automatic ScVal encoding/decoding.

## Installation

```bash
npm install @stellar-insured/sdk
# or
yarn add @stellar-insured/sdk
```

## Quick Start

```ts
import { Keypair } from "@stellar/stellar-sdk";
import {
  SorobanClient,
  PolicyContract,
  ClaimsContract,
  RiskPoolContract,
  GovernanceContract,
} from "@stellar-insured/sdk";

// 1. Create client
const client = new SorobanClient({
  rpcUrl: "https://soroban-testnet.stellar.org",
  // networkPassphrase defaults to Testnet
});

const admin = Keypair.fromSecret("SCZANGBA5YHTNYVVV3C7CAZMCLXPILHSE62DWTELPWO4FMHWCKHMHHBK");
const txOpts = { source: admin };

// 2. Instantiate contract wrappers with deployed contract IDs
const policy = new PolicyContract(client, "CABC...XYZ");
const claims = new ClaimsContract(client, "CDEF...UVW");
const pool = new RiskPoolContract(client, "CGHI...RST");
const gov = new GovernanceContract(client, "CJKL...OPQ");
```

## Usage Examples

### Policy Lifecycle

```ts
// Initialize the policy contract
await policy.initialize(
  "GADMIN...",
  "GGUARDIAN...",
  txOpts
);

// Issue a new policy: 10,000 coverage, 500 premium
await policy.issuePolicy(
  "GHOLDER...",
  1n,          // policyId
  10_000n,     // coverage
  500n,        // premium
  txOpts
);

// Issue with custom duration (365 days)
await policy.issuePolicyWithDuration(
  "GHOLDER...",
  2n,
  25_000n,
  1_200n,
  365,
  txOpts
);

// Check if policy is active (read-only, simulated)
const isActive = await policy.isPolicyActive(1n, txOpts);
console.log("Active:", isActive);

// Get coverage amount
const coverage = await policy.getPolicyCoverage(1n, txOpts);
console.log("Coverage:", coverage);

// Calculate dynamic premium based on risk factors
const premium = await policy.calculateDynamicPremium(
  500n,  // base premium
  75,    // risk score (0-100)
  120,   // location factor
  80,    // coverage ratio
  txOpts
);

// Cancel a policy
await policy.cancelPolicy(1n, txOpts);

// Request an endorsement (coverage/premium change)
await policy.requestEndorsement(
  "GHOLDER...",
  2n,          // policyId
  30_000n,     // new coverage
  1_500n,      // new premium
  "Increased property value after renovation",
  txOpts
);

// Approve the endorsement (admin)
await policy.approveEndorsement("GADMIN...", 0n, txOpts);
```

### Claims Processing

```ts
// Submit a claim
await claims.submitClaim(
  "CPOLICY_CONTRACT...",  // policy contract address
  0n,                     // claimId
  1n,                     // policyId
  5_000n,                 // amount
  txOpts
);

// Attach evidence
await claims.submitEvidence(
  0n,                                    // claimId
  "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",  // IPFS hash
  false,                                 // not sensitive
  "Photo of water damage in living room",
  "GSUBMITTER...",
  txOpts
);

// Verify evidence (admin/guardian)
await claims.verifyEvidence(
  "GADMIN...",
  0n,          // evidenceId
  true,        // valid
  "Confirmed authentic via metadata analysis",
  txOpts
);

// Approve and settle
await claims.approveClaim(0n, txOpts);
await claims.settleClaim(0n, txOpts);

// Check payout status
const status = await claims.getPayoutStatus(0n, txOpts);
console.log("Payout:", status);

// Flag suspicious claim
await claims.flagClaimForReview("GADMIN...", 1n, 25, txOpts);
```

### Risk Pool

```ts
// Deposit liquidity
await pool.deposit("GPROVIDER...", 50_000n, txOpts);

// Check pool balance
const balance = await pool.getBalance(txOpts);
console.log("Pool balance:", balance);

// Configure vesting: 30-day cliff, 180-day duration, 15% early penalty
await pool.setVestingParameters(
  "GADMIN...",
  2_592_000n,   // cliff (30 days in seconds)
  15_552_000n,  // duration (180 days in seconds)
  1500,         // 15% penalty in basis points
  txOpts
);

// Allocate rewards to a provider
await pool.allocateRewards("GADMIN...", "GPROVIDER...", 1_000n, txOpts);

// Claim vested rewards
await pool.claimVestedRewards("GPROVIDER...", txOpts);

// Withdraw from pool
await pool.withdraw("GPROVIDER...", 10_000n, txOpts);

// Set up reinsurance
await pool.setReinsuranceConfig(
  "GADMIN...",
  "GREINSURER...",
  25,     // 25% cession
  850,    // credit score
  txOpts
);
```

### Governance

```ts
// Initialize governance
await gov.initialize(
  "GADMIN...",
  "CTOKEN_CONTRACT...",
  7,       // 7-day voting period
  51,      // 51% min voting threshold
  30,      // 30% quorum
  "CSLASHING_CONTRACT...",
  txOpts
);

// Create a proposal
await gov.createProposal(
  "Increase risk pool minimum stake",
  "Proposal to raise the minimum provider stake from 1000 to 5000 to improve pool stability.",
  '{"action":"set_min_stake","value":5000}',
  60,  // 60% approval threshold
  txOpts
);

// Vote on proposal
await gov.vote(0n, 100n, true, txOpts);   // 100 weight, vote YES

// Finalize after voting period ends
await gov.finalizeProposal(0n, txOpts);

// Execute if passed
await gov.executeProposal(0n, txOpts);

// Query proposals
const active = await gov.getActiveProposals(txOpts);
const stats = await gov.getProposalStats(0n, txOpts);
```

### Emergency Controls

```ts
// Pause any contract (admin only)
await policy.setPauseState(
  "GADMIN...",
  true,
  "Maintenance window for upgrade",
  txOpts
);

// Unpause
await policy.setPauseState(
  "GADMIN...",
  false,
  "Maintenance complete",
  txOpts
);
```

### Validation Errors

All payloads are validated before hitting the network. Invalid inputs throw a `ZodError`:

```ts
try {
  await policy.issuePolicy(
    "",     // invalid: empty address
    -1n,    // invalid: negative policy ID
    0n,     // invalid: zero coverage
    500n,
    txOpts
  );
} catch (err) {
  // ZodError with field-level details:
  // - holder: "Address must not be empty"
  // - policy_id: "Policy ID must be non-negative"
  // - coverage: "Value must be a positive integer"
  console.error(err.issues);
}
```

## API Reference

| Contract | Class | Key Methods |
|----------|-------|-------------|
| Policy | `PolicyContract` | `issuePolicy`, `cancelPolicy`, `expirePolicy`, `isPolicyActive`, `calculateDynamicPremium`, `requestEndorsement`, `approveEndorsement` |
| Claims | `ClaimsContract` | `submitClaim`, `submitEvidence`, `verifyEvidence`, `approveClaim`, `settleClaim`, `retryPayout`, `flagClaimForReview`, `reportFraud` |
| Risk Pool | `RiskPoolContract` | `deposit`, `withdraw`, `setVestingParameters`, `allocateRewards`, `claimVestedRewards`, `appealSlashing`, `resolveAppeal`, `setReinsuranceConfig` |
| Governance | `GovernanceContract` | `createProposal`, `vote`, `finalizeProposal`, `executeProposal`, `createSlashingProposal`, `getActiveProposals` |

## Architecture

```
src/
├── client.ts       # SorobanClient — RPC, tx building, ScVal helpers
├── schemas.ts      # Zod schemas for all payloads + result types
├── contracts/
│   ├── policy.ts       # PolicyContract wrapper
│   ├── claims.ts       # ClaimsContract wrapper
│   ├── risk-pool.ts    # RiskPoolContract wrapper
│   └── governance.ts   # GovernanceContract wrapper
└── index.ts        # Public API barrel export
```
