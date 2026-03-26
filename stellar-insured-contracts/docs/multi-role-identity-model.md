# Multi-Role Identity Model

## Overview

The Multi-Role Identity Model implements a comprehensive role-based access control (RBAC) system for the Property Token contract. This system reduces centralized key risk and supports the principle of least privilege by distributing administrative functions across specialized roles.

## Role Definitions

### 1. Admin (`Role::Admin`)

**Description**: Highest privilege level with full administrative control over the contract.

**Responsibilities**:
- Grant and revoke all roles
- Request admin transfer with timelock
- Set role timelock periods
- Log annual reviews for role holders
- Access to all contract functions
- Emergency override capabilities

**Security Considerations**:
- Admin transfer requires a timelock period (default: 7 days)
- All admin actions are logged on-chain
- Supports delayed transfer to prevent forced takeovers

### 2. Auditor (`Role::Auditor`)

**Description**: Responsible for compliance oversight and claim review operations.

**Responsibilities**:
- Flag tokens for compliance review
- Clear compliance flags after investigation
- Monitor suspicious activities
- Audit claim submissions
- Access compliance history

**Key Functions**:
```rust
// Flag a token for compliance review
pub fn flag_for_compliance_review(
    &mut self,
    token_id: TokenId,
    reason: String,
) -> Result<(), Error>

// Clear a compliance flag
pub fn clear_compliance_flag(
    &mut self,
    token_id: TokenId,
    notes: String,
) -> Result<(), Error>

// Get compliance flag status
pub fn get_compliance_flag_status(&self, token_id: TokenId) -> Option<ComplianceInfo>
```

**Use Case Example**:
When suspicious activity is detected in property transfers or valuations, the auditor can flag the token, preventing further transactions until the issue is resolved.

### 3. Liquidity Manager (`Role::LiquidityManager`)

**Description**: Manages pool parameters and liquidity-related configurations.

**Responsibilities**:
- Adjust dividend parameters
- Modify pool risk parameters
- Set liquidity pool fee rates
- Monitor pool health metrics
- Optimize liquidity provision

**Key Functions**:
```rust
// Update dividend parameters
pub fn update_dividend_parameters(
    &mut self,
    token_id: TokenId,
    new_dividend_rate: u128,
) -> Result<(), Error>

// Adjust pool risk parameters
pub fn adjust_pool_risk_parameters(
    &mut self,
    token_id: TokenId,
    risk_adjustment: i32,
) -> Result<(), Error>

// Set liquidity pool fee
pub fn set_liquidity_pool_fee(
    &mut self,
    token_id: TokenId,
    fee_rate: u128,
) -> Result<(), Error>
```

**Use Case Example**:
During market volatility, the liquidity manager can adjust risk parameters and fee rates to protect the pool and maintain stable operations.

### 4. Governance Operator (`Role::GovernanceOperator`)

**Description**: Executes governance proposals and participates in DAO operations.

**Responsibilities**:
- Execute approved governance proposals
- Veto malicious proposals (with admin)
- Pause/unpause bridge operations in emergencies
- Monitor governance activities
- Facilitate proposal lifecycle

**Key Functions**:
```rust
// Execute a governance proposal
pub fn execute_governance_proposal(
    &mut self,
    token_id: TokenId,
    proposal_id: u64,
) -> Result<(), Error>

// Veto a malicious proposal
pub fn veto_proposal(
    &mut self,
    token_id: TokenId,
    proposal_id: u64,
    reason: String,
) -> Result<(), Error>

// Emergency pause for bridge
pub fn set_emergency_pause(&mut self, paused: bool) -> Result<(), Error>
```

**Use Case Example**:
After a governance vote reaches quorum, the governance operator executes the approved proposal, implementing changes decided by token holders.

## Permission Matrix

| Function | Admin | Auditor | Liquidity Manager | Governance Operator |
|----------|-------|---------|-------------------|---------------------|
| Grant/Revoke Roles | ✓ | ✗ | ✗ | ✗ |
| Admin Transfer | ✓ | ✗ | ✗ | ✗ |
| Set Timelock | ✓ | ✗ | ✗ | ✗ |
| Annual Reviews | ✓ | ✗ | ✗ | ✗ |
| Flag Compliance | ✗ | ✓ | ✗ | ✗ |
| Clear Compliance | ✗ | ✓ | ✗ | ✗ |
| Update Dividend Params | ✗ | ✗ | ✓ | ✗ |
| Adjust Risk Params | ✗ | ✗ | ✓ | ✗ |
| Set Pool Fees | ✗ | ✗ | ✓ | ✗ |
| Execute Proposals | ✗ | ✗ | ✗ | ✓ |
| Veto Proposals | ✓ | ✗ | ✗ | ✓ |
| Emergency Pause | ✓ | ✗ | ✗ | ✓ |
| Bridge Config | ✓ | ✗ | ✗ | ✗ |

## Key Features

### 1. Safe Role Transfer with Timelock

**Purpose**: Prevents forced admin takeovers and provides time for community response.

**Mechanism**:
```rust
// Step 1: Current admin requests transfer
let request_id = contract.request_admin_transfer(new_admin)?;

// Step 2: Wait for timelock period (default: 7 days)
let executable_at = requested_at + timelock_seconds;

// Step 3: New admin executes transfer after timelock
contract.execute_admin_transfer(request_id)?;
```

**Security Benefits**:
- Time window allows stakeholders to react to unauthorized transfers
- Prevents coercion or key compromise attacks
- On-chain visibility of transfer intent

### 2. Annual Review System

**Purpose**: Ensures regular evaluation of role holders and maintains accountability.

**Features**:
- Performance scoring (0-100)
- Renewal tracking
- Detailed notes and feedback
- Automatic expiration extension upon renewal

**Usage**:
```rust
// Log annual review for a role holder
contract.log_annual_review(
    account,
    role,
    performance_score,  // 0-100
    notes,
    is_renewed,
)?;
```

### 3. Role Expiration

**Purpose**: Limits role duration and requires periodic re-authorization.

**Implementation**:
```rust
// Grant role with expiration (1 year)
let expires_at = current_time + 31536000; // 1 year in seconds
contract.grant_role(account, role, Some(expires_at))?;
```

**Benefits**:
- Reduces risk of long-term key compromise
- Ensures regular re-verification of role holders
- Supports automatic renewal through annual review

### 4. Comprehensive Event Logging

All role operations emit events for transparency and auditability:

```rust
// Role granted
RoleGranted {
    account,
    role,
    granted_by,
    granted_at,
    expires_at,
}

// Role revoked
RoleRevoked {
    account,
    role,
    revoked_by,
    revoked_at,
}

// Admin transfer requested
AdminTransferRequested {
    from,
    to,
    request_id,
    requested_at,
    executable_at,
}

// Annual review logged
AnnualReviewLogged {
    account,
    role,
    log_id,
    performance_score,
    is_renewed,
    reviewed_at,
}
```

## Security Best Practices

### 1. Least Privilege Principle

- Grant only necessary permissions for each role
- Avoid role accumulation unless required
- Regular review of role assignments

### 2. Separation of Duties

- Critical operations require different roles
- Prevents single point of failure
- Enables checks and balances

### 3. Regular Audits

- Use annual review system for formal evaluations
- Monitor role usage through event logs
- Track compliance flags and resolutions

### 4. Timelock Configuration

**Recommended Settings**:
- Development/Staging: 3 days (259,200 seconds)
- Production: 7 days (604,800 seconds)
- High-security: 14 days (1,209,600 seconds)

### 5. Role Expiration Strategy

**Recommended Durations**:
- Admin: No expiration (manually revoked)
- Auditor: 1 year with annual renewal
- Liquidity Manager: 1 year with annual renewal
- Governance Operator: 1 year with annual renewal

## Implementation Examples

### Example 1: Setting Up Multi-Sig Style Governance

```rust
// Deploy contract - Alice becomes admin
let mut contract = PropertyToken::new();

// Alice grants specialized roles
contract.grant_role(Bob, Role::Auditor, None)?;
contract.grant_role(Charlie, Role::LiquidityManager, None)?;
contract.grant_role(David, Role::GovernanceOperator, None)?;

// Now operations are distributed:
// - Bob handles compliance
// - Charlie manages liquidity
// - David executes governance
// - Alice retains ultimate control
```

### Example 2: Emergency Response

```rust
// Suspicious activity detected
// Auditor flags the token
contract.flag_for_compliance_review(token_id, "Unusual transfer pattern")?;

// Governance operator pauses bridge if needed
contract.set_emergency_pause(true)?;

// After investigation, auditor clears flag
contract.clear_compliance_flag(token_id, "Issue resolved")?;

// Bridge operations resume
contract.set_emergency_pause(false)?;
```

### Example 3: Admin Succession Planning

```rust
// Current admin plans succession
// Step 1: Request transfer (Month 1)
contract.request_admin_transfer(successor)?;

// Step 2: Wait timelock period (7 days minimum)

// Step 3: Successor executes transfer
contract.execute_admin_transfer(request_id)?;

// Step 4: New admin sets up their own role structure
contract.grant_role(trusted_party, Role::Auditor, None)?;
```

## Monitoring and Compliance

### Key Metrics to Track

1. **Role Distribution**
   - Number of accounts per role
   - Role concentration risk
   - Geographic distribution

2. **Activity Metrics**
   - Compliance flags raised/cleared
   - Pool parameter adjustments
   - Governance proposals executed
   - Annual reviews completed

3. **Security Metrics**
   - Failed authorization attempts
   - Timelock transfer requests
   - Role expirations approaching
   - Emergency pause events

### Audit Trail

All role operations are permanently recorded on-chain:
- Who performed the action
- What action was taken
- When it occurred
- Which role was used

## Integration with Other Contracts

The role system can be extended to other contracts in the ecosystem:

```rust
// In insurance contract
if !property_token.has_role(caller, Role::Auditor) {
    return Err(Error::Unauthorized);
}

// In oracle contract
if !property_token.has_role(caller, Role::LiquidityManager) {
    return Err(Error::Unauthorized);
}
```

## Future Enhancements

Potential improvements for the role system:

1. **Hierarchical Roles**: Sub-roles with limited scope
2. **Time-Limited Roles**: Auto-expiring after specific actions
3. **Multi-Sig Requirements**: Multiple roles for critical operations
4. **Delegated Authority**: Temporary role delegation
5. **Reputation Scoring**: On-chain reputation based on performance

## Conclusion

The Multi-Role Identity Model provides a robust framework for decentralized governance and operations. By distributing authority across specialized roles and implementing safety mechanisms like timelocks and annual reviews, the system achieves:

- ✅ Reduced centralized key risk
- ✅ Least-privilege security model
- ✅ Clear separation of duties
- ✅ Comprehensive audit trail
- ✅ Flexible governance structures
- ✅ Emergency response capabilities

This foundation supports the growth of a truly decentralized property token ecosystem while maintaining security and compliance standards.
