# Claims Contract

<cite>
**Referenced Files in This Document**
- [lib.rs](file://stellar-insured-contracts/contracts/insurance/src/lib.rs)
- [README.md](file://README.md)
- [contracts.md](file://stellar-insured-contracts/docs/contracts.md)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Project Structure](#project-structure)
3. [Core Components](#core-components)
4. [Architecture Overview](#architecture-overview)
5. [Detailed Component Analysis](#detailed-component-analysis)
6. [Dependency Analysis](#dependency-analysis)
7. [Performance Considerations](#performance-considerations)
8. [Troubleshooting Guide](#troubleshooting-guide)
9. [Conclusion](#conclusion)

## Introduction
This document provides comprehensive technical documentation for the Claims contract within the Stellar Insured platform. The Claims contract manages the full lifecycle of insurance claims, from initial submission through investigation, approval/rejection, and final payout execution. It integrates with external oracles for incident verification, enforces compliance checks, and coordinates with Risk Pools for fund allocation.

The Claims contract implements a multi-stage workflow with strict state transitions, evidence validation, and dispute resolution mechanisms. It supports various claim types including property damage, theft, and natural disasters, while maintaining robust security controls against replay attacks and unauthorized access.

## Project Structure
The Claims contract is part of the Property Insurance system, which includes interconnected contracts for policy management, risk assessment, and compliance verification.

```mermaid
graph TB
subgraph "Insurance System"
PI[PropertyInsurance Contract]
RC[RiskPool Management]
OR[Oracle Integration]
CR[Compliance Registry]
PT[Property Token]
end
subgraph "External Systems"
EX[Chainlink Oracles]
IPFS[IPFS Storage]
REG[Regulatory Bodies]
end
PI --> RC
PI --> OR
PI --> CR
PI --> PT
OR --> EX
PI --> IPFS
CR --> REG
subgraph "Claims Workflow"
SUB[Claim Submission]
INV[Investigation]
APP[Approval/Rejection]
PAY[Payout Execution]
DIS[Dispute Resolution]
end
SUB --> INV --> APP --> PAY
APP --> DIS
```

**Diagram sources**
- [lib.rs:1-1873](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1-1873)
- [README.md:28-35](file://README.md#L28-L35)

**Section sources**
- [README.md:1-35](file://README.md#L1-L35)
- [contracts.md:87-105](file://stellar-insured-contracts/docs/contracts.md#L87-L105)

## Core Components

### Claim Data Model
The Claims contract defines a comprehensive claim structure with essential fields for tracking and processing:

```mermaid
classDiagram
class InsuranceClaim {
+u64 claim_id
+u64 policy_id
+AccountId claimant
+u128 claim_amount
+String description
+EvidenceMetadata evidence
+String oracle_report_url
+ClaimStatus status
+u64 submitted_at
+Option~u64~ under_review_at
+Option~u64~ dispute_deadline
+Option~u64~ processed_at
+u128 payout_amount
+Option~AccountId~ assessor
+String rejection_reason
}
class EvidenceMetadata {
+String evidence_type
+String uri
+Vec~u8~ hash
+String nonce
+String description
}
class ClaimStatus {
<<enumeration>>
Pending
UnderReview
OracleVerifying
Approved
Rejected
Paid
Disputed
DisputeResolved
}
InsuranceClaim --> EvidenceMetadata : "contains"
InsuranceClaim --> ClaimStatus : "uses"
```

**Diagram sources**
- [lib.rs:181-197](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L181-L197)
- [lib.rs:124-135](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L124-L135)
- [lib.rs:108-117](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L108-L117)

### Claim Status Transitions
The Claims contract implements a strict finite state machine for claim processing:

```mermaid
stateDiagram-v2
[*] --> Pending
Pending --> UnderReview : "Investigation Started"
UnderReview --> OracleVerifying : "External Verification"
OracleVerifying --> Approved : "Verification Passed"
OracleVerifying --> Rejected : "Verification Failed"
UnderReview --> Approved : "Manual Review Passed"
UnderReview --> Rejected : "Manual Review Failed"
Approved --> Paid : "Payout Executed"
Rejected --> [*]
Paid --> [*]
state UnderReview {
[*] --> OracleVerifying
OracleVerifying --> Approved
OracleVerifying --> Rejected
}
state Disputed {
[*] --> DisputeResolved
DisputeResolved --> Paid
DisputeResolved --> Rejected
}
```

**Diagram sources**
- [lib.rs:108-117](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L108-L117)

**Section sources**
- [lib.rs:181-197](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L181-L197)
- [lib.rs:108-117](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L108-L117)

## Architecture Overview

### Claims Processing Pipeline
The Claims contract orchestrates a sophisticated multi-layered processing pipeline:

```mermaid
sequenceDiagram
participant Policyholder as "Policyholder"
participant Claims as "Claims Contract"
participant Oracle as "External Oracle"
participant RiskPool as "Risk Pool"
participant Compliance as "Compliance Registry"
Policyholder->>Claims : submit_claim()
Claims->>Claims : Validate Evidence Metadata
Claims->>Claims : Check Policy Validity
Claims->>Claims : Verify Nonce Uniqueness
Claims->>Claims : Apply Cooldown Check
Claims->>Claims : Create Claim Record
Claims->>Claims : Emit ClaimSubmitted Event
Claims->>Oracle : Request Incident Verification
Oracle-->>Claims : Return Oracle Report
Claims->>Claims : Validate Oracle Response
Claims->>Compliance : Verify Claimant Compliance
Compliance-->>Claims : Compliance Status
alt Claim Approved
Claims->>Claims : Apply Deductible Logic
Claims->>RiskPool : Execute Payout
RiskPool-->>Claims : Fund Transfer Confirmation
Claims->>Claims : Update Claim Status
Claims->>Claims : Emit ClaimApproved Event
else Claim Rejected
Claims->>Claims : Record Rejection Reason
Claims->>Claims : Emit ClaimRejected Event
end
```

**Diagram sources**
- [lib.rs:973-1162](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L973-L1162)
- [lib.rs:1766-1827](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1766-L1827)

### Integration Architecture
The Claims contract integrates with multiple external systems for comprehensive verification:

```mermaid
graph TB
subgraph "Claims Contract"
CC[Claims Controller]
EV[Evidence Validator]
ST[State Tracker]
DP[Dispute Processor]
end
subgraph "External Integrations"
OR[Oracle Network]
IP[IPFS Storage]
CR[Compliance Registry]
RP[Risk Pools]
ZK[ZK Compliance]
end
subgraph "Regulatory Layer"
REG[Regulatory Bodies]
AUD[Audit Trail]
end
CC --> EV
CC --> ST
CC --> DP
EV --> IP
ST --> OR
ST --> CR
DP --> ZK
OR --> REG
CR --> AUD
ZK --> AUD
```

**Diagram sources**
- [lib.rs:1-1873](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1-1873)

**Section sources**
- [lib.rs:973-1162](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L973-L1162)
- [lib.rs:1766-1827](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1766-L1827)

## Detailed Component Analysis

### Claim Submission Process
The claim submission process implements comprehensive validation and security measures:

```mermaid
flowchart TD
Start([Claim Submission Request]) --> ValidateCaller["Validate Policyholder Identity"]
ValidateCaller --> CheckPolicy["Verify Policy Active Status"]
CheckPolicy --> CheckExpiry["Check Policy Expiration"]
CheckExpiry --> ValidateEvidence["Validate Evidence Metadata"]
ValidateEvidence --> CheckURI["Verify URI Format (ipfs/https)"]
CheckURI --> CheckHash["Validate Hash Length (32 bytes)"]
CheckHash --> CheckNonce["Validate Nonce Presence"]
CheckNonce --> CheckReplay["Prevent Replay Attack"]
CheckReplay --> CheckCoverage["Verify Coverage Amount"]
CheckCoverage --> CheckCooldown["Apply Cooldown Period"]
CheckCooldown --> CreateClaim["Create Claim Record"]
CreateClaim --> UpdateCounters["Update Counters"]
UpdateCounters --> EmitEvent["Emit ClaimSubmitted Event"]
EmitEvent --> End([Submission Complete])
CheckPolicy --> Error1["Policy Inactive/Error"]
CheckExpiry --> Error2["Policy Expired/Error"]
ValidateEvidence --> Error3["Invalid Evidence/Error"]
CheckReplay --> Error4["Nonce Already Used/Error"]
CheckCoverage --> Error5["Exceeds Coverage/Error"]
CheckCooldown --> Error6["Cooldown Active/Error"]
Error1 --> End
Error2 --> End
Error3 --> End
Error4 --> End
Error5 --> End
Error6 --> End
```

**Diagram sources**
- [lib.rs:973-1079](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L973-L1079)

### Investigation and Approval Workflow
The investigation process combines automated verification with manual oversight:

```mermaid
sequenceDiagram
participant Assessor as "Authorized Assessor"
participant Claims as "Claims Contract"
participant Oracle as "Oracle System"
participant Pool as "Risk Pool"
Assessor->>Claims : process_claim(approved=true, report_url, reason="")
Claims->>Claims : Validate Assessor Authorization
Claims->>Claims : Check Claim Status (Pending/UnderReview)
Claims->>Claims : Record Assessor Information
Claims->>Claims : Update Oracle Report URL
Claims->>Claims : Set Processed Timestamp
alt First Processing
Claims->>Claims : Set UnderReview Timestamp
Claims->>Claims : Calculate Dispute Deadline
end
alt Claim Approved
Claims->>Claims : Fetch Policy Details
Claims->>Claims : Apply Deductible Calculation
Claims->>Claims : Calculate Payout Amount
Claims->>Claims : Update Claim Status to Approved
Claims->>Pool : Execute Payout
Pool-->>Claims : Confirm Fund Transfer
Claims->>Claims : Update Claim Status to Paid
Claims->>Claims : Emit ClaimApproved Event
else Claim Rejected
Claims->>Claims : Record Rejection Reason
Claims->>Claims : Update Claim Status to Rejected
Claims->>Claims : Emit ClaimRejected Event
end
```

**Diagram sources**
- [lib.rs:1082-1162](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1082-L1162)
- [lib.rs:1766-1827](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1766-L1827)

### Payout Execution Engine
The payout system implements sophisticated fund management with risk controls:

```mermaid
flowchart TD
PayoutStart([Payout Request]) --> CheckAmount["Check Payout Amount"]
CheckAmount --> ZeroAmount{"Amount = 0?"}
ZeroAmount --> |Yes| SkipPayout["Skip Payout"]
ZeroAmount --> |No| FetchPolicy["Fetch Policy Details"]
FetchPolicy --> FetchPool["Fetch Risk Pool Details"]
FetchPool --> CheckReinsurance["Check Reinsurance Threshold"]
CheckReinsurance --> NeedReinsurance{"Amount > Threshold?"}
NeedReinsurance --> |Yes| TryReinsurance["Attempt Reinsurance Recovery"]
NeedReinsurance --> |No| CheckFunds["Check Pool Funds"]
TryReinsurance --> CheckFunds
CheckFunds --> SufficientFunds{"Available Capital >= Payout?"}
SufficientFunds --> |No| InsufficientError["Insufficient Pool Funds"]
SufficientFunds --> |Yes| UpdatePool["Update Pool Available Capital"]
UpdatePool --> UpdatePolicy["Update Policy Total Claimed"]
UpdatePolicy --> UpdateCooldown["Update Property Cooldown"]
UpdateCooldown --> UpdateClaim["Update Claim Status to Paid"]
UpdateClaim --> EmitPayout["Emit PayoutExecuted Event"]
EmitPayout --> PayoutComplete([Payout Complete])
InsufficientError --> PayoutComplete
SkipPayout --> PayoutComplete
```

**Diagram sources**
- [lib.rs:1766-1827](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1766-L1827)

**Section sources**
- [lib.rs:973-1162](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L973-L1162)
- [lib.rs:1766-1827](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1766-L1827)

### Dispute Resolution Mechanism
The Claims contract implements a comprehensive dispute resolution system:

```mermaid
stateDiagram-v2
[*] --> Active : Claim Submitted
Active --> UnderReview : Investigation Started
UnderReview --> Disputed : Dispute Raised
Disputed --> DisputeResolved : Arbiter Decision
DisputeResolved --> Paid : Approved
DisputeResolved --> Rejected : Denied
note right of Disputed : Dispute Window : 7 days<br/>Arbiter Authority<br/>Policyholder Appeal
note right of DisputeResolved : Final Resolution<br/>Automated Payout or Rejection
```

**Diagram sources**
- [lib.rs:1535-1572](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1535-L1572)

**Section sources**
- [lib.rs:1535-1572](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1535-L1572)

## Dependency Analysis

### Claims Contract Dependencies
The Claims contract interacts with multiple system components through well-defined interfaces:

```mermaid
graph TB
subgraph "Internal Dependencies"
PC[Policy Contract]
RP[Risk Pool Manager]
RA[Risk Assessment]
LI[Liquidity Providers]
AC[Actuarial Models]
end
subgraph "External Dependencies"
OR[Oracle Network]
IP[IPFS Storage]
CR[Compliance Registry]
ZK[ZK Compliance]
end
subgraph "Claims Contract"
CS[Claims Storage]
ES[Evidence Storage]
SS[Status Tracking]
DS[Dispute System]
end
CS --> PC
CS --> RP
CS --> RA
ES --> IP
SS --> OR
SS --> CR
DS --> ZK
PC --> LI
RP --> AC
```

**Diagram sources**
- [lib.rs:1-1873](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1-1873)

### Data Flow Patterns
The Claims contract follows established patterns for data persistence and retrieval:

```mermaid
erDiagram
INSURANCE_CLAIM {
u64 claim_id PK
u64 policy_id FK
AccountId claimant
u128 claim_amount
String description
String oracle_report_url
ClaimStatus status
u64 submitted_at
u64 under_review_at
u64 dispute_deadline
u64 processed_at
u128 payout_amount
AccountId assessor
String rejection_reason
}
EVIDENCE_METADATA {
String evidence_type
String uri
bytes hash
String nonce
String description
}
INSURANCE_POLICY {
u64 policy_id PK
u64 property_id
AccountId policyholder
CoverageType coverage_type
u128 coverage_amount
u128 premium_amount
u128 deductible
u64 start_time
u64 end_time
PolicyStatus status
u64 pool_id
u32 claims_count
u128 total_claimed
}
RISK_POOL {
u64 pool_id PK
String name
CoverageType coverage_type
u128 total_capital
u128 available_capital
u128 total_premiums_collected
u128 total_claims_paid
u64 active_policies
u32 max_coverage_ratio
u128 reinsurance_threshold
u64 created_at
boolean is_active
}
INSURANCE_CLAIM ||--|| INSURANCE_POLICY : "belongs_to"
INSURANCE_CLAIM ||--|| EVIDENCE_METADATA : "contains"
INSURANCE_POLICY ||--|| RISK_POOL : "managed_by"
```

**Diagram sources**
- [lib.rs:181-197](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L181-L197)
- [lib.rs:159-175](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L159-L175)
- [lib.rs:203-216](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L203-L216)

**Section sources**
- [lib.rs:1-1873](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L1-1873)

## Performance Considerations

### Gas Optimization Strategies
The Claims contract implements several optimization techniques for cost-effective operation:

1. **Efficient Storage Layout**: Uses compact storage structures with minimal memory footprint
2. **Batch Operations**: Supports batch processing for multiple claims and policies
3. **Lazy Evaluation**: Defers expensive computations until necessary
4. **Cache Management**: Maintains hot data in frequently accessed storage slots

### Scalability Features
- **Modular Design**: Separate concerns for claims, policies, and risk management
- **Event-Driven Architecture**: Reduces computational overhead through event emission
- **Index Management**: Optimized lookup patterns for claim and policy queries
- **Resource Limits**: Built-in caps on pool exposure and claim processing

### Security Optimizations
- **Nonce Validation**: Prevents replay attacks through unique nonce tracking
- **Authorization Checks**: Multi-level permission system for claim processing
- **Input Validation**: Comprehensive validation for all external inputs
- **State Consistency**: Atomic operations for claim state transitions

## Troubleshooting Guide

### Common Claim Processing Issues

#### Evidence Validation Failures
**Symptoms**: Claims consistently rejected during submission
**Causes**: 
- Invalid URI format (not starting with ipfs:// or https://)
- Hash length not equal to 32 bytes
- Empty or missing nonce value
- Duplicate nonce detected

**Resolutions**:
1. Verify evidence metadata format compliance
2. Ensure unique nonce values per claim submission
3. Check IPFS content availability and accessibility
4. Validate hash computation accuracy

#### Funding Shortage Errors
**Symptoms**: Payout execution fails with insufficient funds
**Causes**:
- Risk pool insufficient capital
- Reinsurance threshold exceeded
- Pool exposure limits reached

**Resolutions**:
1. Increase pool liquidity through additional capital providers
2. Adjust coverage amounts to fit pool capacity
3. Implement reinsurance agreements for large claims
4. Monitor pool utilization ratios

#### Authorization Problems
**Symptoms**: Claims rejected with unauthorized access errors
**Causes**:
- Non-policyholder attempting claim submission
- Unauthenticated assessors processing claims
- Missing oracle authorization

**Resolutions**:
1. Verify policyholder identity and ownership
2. Register authorized assessors through admin functions
3. Configure oracle addresses for verification
4. Check compliance registry status

**Section sources**
- [lib.rs:23-54](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L23-L54)
- [lib.rs:989-1005](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L989-L1005)

## Conclusion

The Claims contract represents a sophisticated, enterprise-grade solution for insurance claims processing on the Stellar blockchain. Its comprehensive feature set includes advanced validation mechanisms, multi-layered verification processes, and robust security controls.

Key strengths of the implementation include:

- **Comprehensive Evidence Management**: Structured evidence validation prevents fraud while enabling efficient processing
- **Flexible Integration**: Seamless integration with external oracles and compliance systems
- **Robust Security**: Multi-layered authorization and replay attack prevention
- **Scalable Architecture**: Modular design supporting high-volume claim processing
- **Transparent Operations**: Complete audit trail through comprehensive event emission

The contract successfully balances security, efficiency, and regulatory compliance while maintaining flexibility for future enhancements. Its integration with the broader Property Insurance ecosystem creates a cohesive platform for comprehensive property risk management.

Future enhancements could include expanded claim types, enhanced AI-powered fraud detection, and additional integration points with emerging DeFi protocols.