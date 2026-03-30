# stellar-insured-sdk — Python

Schema-validated Python SDK for the Stellar Insured Soroban contracts. Wraps all core contract methods (Policy, Claims, Risk Pool, Governance) with **Pydantic validation** on every payload and automatic ScVal encoding/decoding.

## Installation

```bash
pip install stellar-insured-sdk
# or
pip install -e sdk/python  # local development
```

## Quick Start

```python
from stellar_sdk import Keypair
from stellar_insured_sdk import (
    SorobanClient,
    SorobanClientConfig,
    TxOptions,
    PolicyContract,
    ClaimsContract,
    RiskPoolContract,
    GovernanceContract,
)

# 1. Create client
client = SorobanClient(SorobanClientConfig(
    rpc_url="https://soroban-testnet.stellar.org",
))

admin = Keypair.from_secret("SCZANGBA5YHTNYVVV3C7CAZMCLXPILHSE62DWTELPWO4FMHWCKHMHHBK")
tx_opts = TxOptions(source=admin)

# 2. Instantiate contract wrappers with deployed contract IDs
policy = PolicyContract(client, "CABC...XYZ")
claims = ClaimsContract(client, "CDEF...UVW")
pool = RiskPoolContract(client, "CGHI...RST")
gov = GovernanceContract(client, "CJKL...OPQ")
```

## Usage Examples

### Policy Lifecycle

```python
# Initialize the policy contract
policy.initialize("GADMIN...", "GGUARDIAN...", tx_opts)

# Issue a new policy: 10,000 coverage, 500 premium
policy.issue_policy(
    holder="GHOLDER...",
    policy_id=1,
    coverage=10_000,
    premium=500,
    opts=tx_opts,
)

# Issue with custom duration (365 days)
policy.issue_policy_with_duration(
    holder="GHOLDER...",
    policy_id=2,
    coverage=25_000,
    premium=1_200,
    duration_days=365,
    opts=tx_opts,
)

# Check if policy is active (read-only, simulated)
is_active = policy.is_policy_active(1, tx_opts)
print(f"Active: {is_active}")

# Get coverage amount
coverage = policy.get_policy_coverage(1, tx_opts)
print(f"Coverage: {coverage}")

# Calculate dynamic premium based on risk factors
premium = policy.calculate_dynamic_premium(
    base_premium=500,
    risk_score=75,        # 0-100
    location_factor=120,
    coverage_ratio=80,
    opts=tx_opts,
)

# Cancel a policy
policy.cancel_policy(1, tx_opts)

# Request an endorsement (coverage/premium change)
policy.request_endorsement(
    requester="GHOLDER...",
    policy_id=2,
    new_coverage=30_000,
    new_premium=1_500,
    reason="Increased property value after renovation",
    opts=tx_opts,
)

# Approve the endorsement (admin)
policy.approve_endorsement("GADMIN...", 0, tx_opts)
```

### Claims Processing

```python
# Submit a claim
claims.submit_claim(
    policy_address="CPOLICY_CONTRACT...",
    claim_id=0,
    policy_id=1,
    amount=5_000,
    opts=tx_opts,
)

# Attach evidence
claims.submit_evidence(
    claim_id=0,
    ipfs_hash="QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
    sensitive=False,
    description="Photo of water damage in living room",
    submitter="GSUBMITTER...",
    opts=tx_opts,
)

# Verify evidence (admin/guardian)
claims.verify_evidence(
    caller="GADMIN...",
    evidence_id=0,
    is_valid=True,
    notes="Confirmed authentic via metadata analysis",
    opts=tx_opts,
)

# Approve and settle
claims.approve_claim(0, tx_opts)
claims.settle_claim(0, tx_opts)

# Check payout status
status = claims.get_payout_status(0, tx_opts)
print(f"Payout: {status}")

# Flag suspicious claim
claims.flag_claim_for_review("GADMIN...", 1, score_adjustment=25, opts=tx_opts)
```

### Risk Pool

```python
# Deposit liquidity
pool.deposit("GPROVIDER...", 50_000, tx_opts)

# Check pool balance
balance = pool.get_balance(tx_opts)
print(f"Pool balance: {balance}")

# Configure vesting: 30-day cliff, 180-day duration, 15% early penalty
pool.set_vesting_parameters(
    caller="GADMIN...",
    cliff_secs=2_592_000,     # 30 days
    duration_secs=15_552_000, # 180 days
    penalty_bps=1500,         # 15%
    opts=tx_opts,
)

# Allocate rewards to a provider
pool.allocate_rewards("GADMIN...", "GPROVIDER...", 1_000, tx_opts)

# Claim vested rewards
pool.claim_vested_rewards("GPROVIDER...", tx_opts)

# Withdraw from pool
pool.withdraw("GPROVIDER...", 10_000, tx_opts)

# Set up reinsurance
pool.set_reinsurance_config(
    caller="GADMIN...",
    reinsurer="GREINSURER...",
    percentage=25,
    credit_score=850,
    opts=tx_opts,
)
```

### Governance

```python
# Initialize governance
gov.initialize(
    admin="GADMIN...",
    token_contract="CTOKEN_CONTRACT...",
    voting_period_days=7,
    min_voting_percentage=51,
    min_quorum_percentage=30,
    slashing_contract="CSLASHING_CONTRACT...",
    opts=tx_opts,
)

# Create a proposal
gov.create_proposal(
    title="Increase risk pool minimum stake",
    description="Raise the minimum provider stake from 1000 to 5000.",
    execution_data='{"action":"set_min_stake","value":5000}',
    threshold_percentage=60,
    opts=tx_opts,
)

# Vote on proposal
gov.vote(proposal_id=0, vote_weight=100, is_yes=True, opts=tx_opts)

# Finalize after voting period ends
gov.finalize_proposal(0, tx_opts)

# Execute if passed
gov.execute_proposal(0, tx_opts)

# Query proposals
active = gov.get_active_proposals(tx_opts)
stats = gov.get_proposal_stats(0, tx_opts)
```

### Emergency Controls

```python
# Pause any contract (admin only)
policy.set_pause_state(
    caller="GADMIN...",
    is_paused=True,
    reason="Maintenance window for upgrade",
    opts=tx_opts,
)

# Unpause
policy.set_pause_state(
    caller="GADMIN...",
    is_paused=False,
    reason="Maintenance complete",
    opts=tx_opts,
)
```

### Validation Errors

All payloads are validated before hitting the network. Invalid inputs raise `ValidationError`:

```python
from pydantic import ValidationError

try:
    policy.issue_policy(
        holder="",       # invalid: empty address
        policy_id=-1,    # invalid: negative
        coverage=0,      # invalid: must be positive
        premium=500,
        opts=tx_opts,
    )
except ValidationError as e:
    # Field-level error details:
    # - holder: "String should have at least 1 character"
    # - policy_id: "Input should be greater than or equal to 0"
    # - coverage: "Input should be greater than 0"
    print(e.errors())
```

## API Reference

| Contract | Class | Key Methods |
|----------|-------|-------------|
| Policy | `PolicyContract` | `issue_policy`, `cancel_policy`, `expire_policy`, `is_policy_active`, `calculate_dynamic_premium`, `request_endorsement`, `approve_endorsement` |
| Claims | `ClaimsContract` | `submit_claim`, `submit_evidence`, `verify_evidence`, `approve_claim`, `settle_claim`, `retry_payout`, `flag_claim_for_review`, `report_fraud` |
| Risk Pool | `RiskPoolContract` | `deposit`, `withdraw`, `set_vesting_parameters`, `allocate_rewards`, `claim_vested_rewards`, `appeal_slashing`, `resolve_appeal`, `set_reinsurance_config` |
| Governance | `GovernanceContract` | `create_proposal`, `vote`, `finalize_proposal`, `execute_proposal`, `create_slashing_proposal`, `get_active_proposals` |

## Architecture

```
stellar_insured_sdk/
├── __init__.py         # Public API
├── client.py           # SorobanClient — RPC, tx building, ScVal helpers
├── schemas.py          # Pydantic models for all payloads + result types
└── contracts/
    ├── __init__.py
    ├── policy.py       # PolicyContract wrapper
    ├── claims.py       # ClaimsContract wrapper
    ├── risk_pool.py    # RiskPoolContract wrapper
    └── governance.py   # GovernanceContract wrapper
```
