# Multi-Role Identity Model - Quick Reference

## Role Permissions at a Glance

| Action | Admin | Auditor | Liquidity Manager | Governance Operator |
|--------|-------|---------|-------------------|---------------------|
| Grant/revoke roles | ✅ | ❌ | ❌ | ❌ |
| Transfer admin | ✅ | ❌ | ❌ | ❌ |
| Flag compliance | ❌ | ✅ | ❌ | ❌ |
| Clear compliance | ❌ | ✅ | ❌ | ❌ |
| Update dividends | ❌ | ❌ | ✅ | ❌ |
| Adjust risk params | ❌ | ❌ | ✅ | ❌ |
| Set pool fees | ❌ | ❌ | ✅ | ❌ |
| Execute proposals | ❌ | ❌ | ❌ | ✅ |
| Veto proposals | ✅ | ❌ | ❌ | ✅ |
| Emergency pause | ✅ | ❌ | ❌ | ✅ |

## Function Signatures

### Admin Functions

```rust
// Grant role to account
grant_role(account: AccountId, role: Role, expires_at: Option<u64>) -> Result<(), Error>

// Revoke role from account  
revoke_role(account: AccountId, role: Role) -> Result<(), Error>

// Check if account has role
has_role(account: AccountId, role: Role) -> bool

// Request admin transfer (with timelock)
request_admin_transfer(new_admin: AccountId) -> Result<u64, Error>

// Execute admin transfer after timelock
execute_admin_transfer(request_id: u64) -> Result<(), Error>

// Cancel pending transfer
cancel_admin_transfer(request_id: u64) -> Result<(), Error>

// Set timelock period
set_role_timelock_seconds(seconds: u64) -> Result<(), Error>

// Log annual review
log_annual_review(account: AccountId, role: Role, score: u32, notes: String, renewed: bool) -> Result<u64, Error>
```

### Auditor Functions

```rust
// Flag token for compliance review
flag_for_compliance_review(token_id: TokenId, reason: String) -> Result<(), Error>

// Clear compliance flag
clear_compliance_flag(token_id: TokenId, notes: String) -> Result<(), Error>

// Get compliance status
get_compliance_flag_status(token_id: TokenId) -> Option<ComplianceInfo>
```

### Liquidity Manager Functions

```rust
// Update dividend parameters
update_dividend_parameters(token_id: TokenId, rate: u128) -> Result<(), Error>

// Adjust pool risk (-100 to 100)
adjust_pool_risk_parameters(token_id: TokenId, adjustment: i32) -> Result<(), Error>

// Set pool fee (0-10000 basis points)
set_liquidity_pool_fee(token_id: TokenId, fee_rate: u128) -> Result<(), Error>
```

### Governance Operator Functions

```rust
// Execute approved proposal
execute_governance_proposal(token_id: TokenId, proposal_id: u64) -> Result<(), Error>

// Veto malicious proposal
veto_proposal(token_id: TokenId, proposal_id: u64, reason: String) -> Result<(), Error>

// Emergency pause bridge
set_emergency_pause(paused: bool) -> Result<(), Error>
```

## Common Workflows

### Workflow 1: Setup Multi-Role System

```rust
// 1. Deploy contract (Alice becomes admin)
let mut contract = PropertyToken::new();

// 2. Grant specialized roles
contract.grant_role(Bob, Role::Auditor, None)?;
contract.grant_role(Charlie, Role::LiquidityManager, None)?;
contract.grant_role(David, Role::GovernanceOperator, None)?;

// 3. Verify roles
assert!(contract.has_role(Bob, Role::Auditor));
assert!(contract.has_role(Charlie, Role::LiquidityManager));
assert!(contract.has_role(David, Role::GovernanceOperator));
```

### Workflow 2: Handle Suspicious Activity

```rust
// 1. Auditor detects and flags issue
contract.flag_for_compliance_review(token_id, "Suspicious pattern")?;

// 2. Off-chain investigation occurs...

// 3. Auditor clears after resolution
contract.clear_compliance_flag(token_id, "Issue resolved")?;
```

### Workflow 3: Admin Succession

```rust
// 1. Current admin requests transfer
let request_id = contract.request_admin_transfer(successor)?;

// 2. Wait for timelock (default: 7 days)

// 3. Successor executes transfer
contract.execute_admin_transfer(request_id)?;
```

### Workflow 4: Annual Review Cycle

```rust
// Admin logs review for each role holder
contract.log_annual_review(
    Bob,                    // account
    Role::Auditor,         // role
    85,                    // performance score (0-100)
    "Excellent work".into(), // notes
    true                   // renew for another year
)?;
```

## Default Configuration Values

| Parameter | Default Value | Description |
|-----------|--------------|-------------|
| Timelock period | 604,800 seconds | 7 days for admin transfer |
| Role expiration | None | No automatic expiration unless set |
| Performance score range | 0-100 | Annual review scoring |
| Renewal extension | 31,536,000 seconds | 1 year extension |

## Error Codes

| Error | When It Occurs |
|-------|----------------|
| `Unauthorized` | Caller lacks required role |
| `PropertyNotFound` | Role not assigned to account |
| `ComplianceFailed` | Operation conflicts with existing state |
| `ProposalNotFound` | Invalid transfer request ID |
| `ProposalClosed` | Transfer already executed/cancelled |

## Event Monitoring

Key events to monitor:

```rust
// Role changes
RoleGranted { account, role, granted_by, ... }
RoleRevoked { account, role, revoked_by, ... }

// Admin transfer
AdminTransferRequested { from, to, request_id, executable_at }
AdminTransferExecuted { request_id, from, to, ... }
AdminTransferCancelled { request_id, ... }

// Operations
ComplianceFlagged { token_id, reason, ... }
ComplianceFlagCleared { token_id, ... }
GovernanceProposalExecuted { proposal_id, ... }
EmergencyPauseUpdated { paused, ... }

// Reviews
AnnualReviewLogged { account, role, score, is_renewed, ... }
```

## Security Checklist

- [ ] Set appropriate timelock for your security requirements
- [ ] Distribute roles across different individuals/entities
- [ ] Configure role expiration for periodic re-verification
- [ ] Monitor failed authorization attempts
- [ ] Log all annual reviews with meaningful scores
- [ ] Use multi-sig for admin role when possible
- [ ] Document off-chain procedures for each role
- [ ] Regular audit of role assignments

## Gas Optimization Tips

1. **Batch role grants** when onboarding multiple users
2. **Use role expiration** to clean up inactive assignments
3. **Monitor event logs** off-chain instead of on-chain queries
4. **Cache role info** in frontend applications

## Integration Patterns

### Pattern 1: Cross-Contract Role Checking

```rust
// In another contract
if !property_token.has_role(caller, Role::Auditor) {
    return Err(Error::Unauthorized);
}
// Proceed with auditor-only operation
```

### Pattern 2: Multi-Sig Admin

```rust
// Admin is a multi-sig wallet address
// All admin operations require multi-sig approval
contract.grant_role(...); // Requires M-of-N signatures
```

### Pattern 3: DAO Governance

```rust
// Admin is DAO treasury
// Roles granted based on community votes
contract.grant_role(elected_rep, Role::GovernanceOperator, None)?;
```

## Troubleshooting

**Problem**: User can't perform role-specific action  
**Solution**: Verify `has_role(user, required_role)` returns true

**Problem**: Admin transfer stuck  
**Solution**: Check timelock hasn't expired, verify caller is recipient

**Problem**: Role appears inactive  
**Solution**: Check expiration time and `is_active` flag in role info

**Problem**: Compliance flag not clearing  
**Solution**: Ensure only auditor calls the clear function

---

**Quick Help**: See full documentation at `docs/multi-role-identity-model.md`
