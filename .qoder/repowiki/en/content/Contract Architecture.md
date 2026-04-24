# Contract Architecture

<cite>
**Referenced Files in This Document**
- [Cargo.toml](file://stellar-insured-contracts/Cargo.toml)
- [architecture.md](file://stellar-insured-contracts/docs/architecture.md)
- [contracts.md](file://stellar-insured-contracts/docs/contracts.md)
- [security_pipeline.md](file://stellar-insured-contracts/docs/security_pipeline.md)
- [best-practices.md](file://stellar-insured-contracts/docs/best-practices.md)
- [lib.rs](file://stellar-insured-contracts/contracts/lib/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/property-token/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/escrow/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/compliance_registry/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/insurance/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/bridge/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/oracle/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/proxy/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/traits/src/lib.rs)
- [lib.rs](file://stellar-insured-contracts/contracts/fractional/src/lib.rs)
</cite>

## Table of Contents
1. [Introduction](#introduction)
2. [Project Structure](#project-structure)
3. [Core Components](#core-components)
4. [Architecture Overview](#architecture-overview)
5. [Detailed Component Analysis](#detailed-component-analysis)
6. [Dependency Analysis](#dependency-analysis)
7. [Performance Considerations](#performance-considerations)
8. [Security Architecture](#security-architecture)
9. [Event Emission System](#event-emission-system)
10. [Upgrade and Proxy Mechanisms](#upgrade-and-proxy-mechanisms)
11. [Scalability and Monitoring](#scalability-and-monitoring)
12. [Integration and Extensibility](#integration-and-extensibility)
13. [Troubleshooting Guide](#troubleshooting-guide)
14. [Conclusion](#conclusion)

## Introduction
This document presents the architectural design of the PropChain smart contract system, a comprehensive real estate tokenization platform built on the Substrate blockchain using the ink! smart contract language. The system integrates multiple specialized contracts for property registration, tokenization, escrow, compliance, insurance, bridging, and oracle-driven valuations. It emphasizes modular design, transparent governance, robust security controls, and scalable operations across multiple chains.

## Project Structure
The workspace is organized as a Rust workspace with multiple member crates grouped by domain functionality:
- Core library and shared traits define common data structures, events, and interfaces used across contracts.
- Domain-specific contracts implement specialized capabilities: property token, escrow, compliance registry, insurance, bridge, oracle, fractional ownership, and proxy.
- Documentation covers architecture, contracts, best practices, security pipeline, and operational guidelines.
- Scripts and CI workflows automate building, testing, auditing, and deployment.

```mermaid
graph TB
subgraph "Workspace"
WS["Cargo.toml<br/>Workspace members"]
end
subgraph "Contracts"
LIB["contracts/lib<br/>Core library"]
TR["contracts/traits<br/>Shared traits"]
PT["contracts/property-token<br/>Token contract"]
ESC["contracts/escrow<br/>Escrow contract"]
CR["contracts/compliance_registry<br/>Compliance registry"]
INS["contracts/insurance<br/>Insurance platform"]
BR["contracts/bridge<br/>Cross-chain bridge"]
OR["contracts/oracle<br/>Property valuation oracle"]
FX["contracts/fractional<br/>Fractional ownership"]
PX["contracts/proxy<br/>Transparent proxy"]
end
subgraph "Docs"
DOC_ARCH["docs/architecture.md"]
DOC_CONTRACTS["docs/contracts.md"]
DOC_BP["docs/best-practices.md"]
DOC_SEC["docs/security_pipeline.md"]
end
WS --> LIB
WS --> TR
WS --> PT
WS --> ESC
WS --> CR
WS --> INS
WS --> BR
WS --> OR
WS --> FX
WS --> PX
LIB --- TR
PT --- TR
ESC --- TR
CR --- TR
INS --- TR
BR --- TR
OR --- TR
FX --- TR
PX --- TR
DOC_ARCH -.-> WS
DOC_CONTRACTS -.-> WS
DOC_BP -.-> WS
DOC_SEC -.-> WS
```

**Diagram sources**
- [Cargo.toml:1-45](file://stellar-insured-contracts/Cargo.toml#L1-L45)

**Section sources**
- [Cargo.toml:1-45](file://stellar-insured-contracts/Cargo.toml#L1-L45)

## Core Components
The system comprises several core contracts, each implementing a distinct aspect of the real estate ecosystem:

- Property Registry: Central registry for property metadata, ownership, approvals, badges, verifications, appeals, and pause/resume governance.
- Property Token: ERC-721/1155-compatible token contract with property metadata, compliance flags, legal documents, bridging, fractional shares, and governance events.
- Escrow: Advanced escrow with multi-signature approvals, time locks, conditions, dispute resolution, and audit trails.
- Compliance Registry: Jurisdiction-aware KYC/AML/sanctions compliance with GDPR consent, risk scoring, and audit logs.
- Insurance: Decentralized insurance platform with risk pools, claims, reinsurance, and dispute resolution.
- Bridge: Multi-signature cross-chain property token transfer with monitoring and recovery actions.
- Oracle: Property valuation oracle with multiple sources, confidence metrics, anomaly detection, and AI integration.
- Fractional: Lightweight fractional ownership aggregation and tax reporting helpers.
- Proxy: Transparent proxy for upgradable contract implementations.

**Section sources**
- [architecture.md:9-101](file://stellar-insured-contracts/docs/architecture.md#L9-L101)
- [lib.rs:50-97](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L50-L97)
- [lib.rs:47-102](file://stellar-insured-contracts/contracts/property-token/src/lib.rs#L47-L102)
- [lib.rs:11-162](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L11-L162)
- [lib.rs:213-241](file://stellar-insured-contracts/contracts/compliance_registry/lib.rs#L213-L241)
- [lib.rs:12-379](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L12-L379)
- [lib.rs:32-61](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L32-L61)
- [lib.rs:22-75](file://stellar-insured-contracts/contracts/oracle/src/lib.rs#L22-L75)
- [lib.rs:55-58](file://stellar-insured-contracts/contracts/fractional/src/lib.rs#L55-L58)

## Architecture Overview
The system follows a modular, layered architecture:
- Shared traits define interfaces and data models used across contracts.
- Core contracts encapsulate domain logic with explicit state management and event emission.
- Orchestration occurs through cross-contract calls and shared registries.
- Governance and security are enforced via role-based access control, pause mechanisms, and multi-signature workflows.

```mermaid
graph TB
subgraph "Governance & Security"
ADMIN["Admin"]
PAUSE["Pause/Resume"]
ROLES["Role-Based Access Control"]
end
subgraph "Core Contracts"
REG["Property Registry"]
TOK["Property Token"]
ESC["Escrow"]
COM["Compliance Registry"]
INS["Insurance"]
BRG["Bridge"]
ORA["Oracle"]
FRA["Fractional"]
PRX["Proxy"]
end
subgraph "External Systems"
IPFS["IPFS"]
ORACLES["External Oracles"]
CHAINS["Cross-chain Bridges"]
end
ADMIN --> PAUSE
ROLES --> REG
ROLES --> TOK
ROLES --> ESC
ROLES --> COM
ROLES --> INS
ROLES --> BRG
ROLES --> ORA
REG --> TOK
TOK --> ESC
TOK --> BRG
REG --> COM
INS --> ORA
ORA --> IPFS
BRG --> CHAINS
PRX --> REG
PRX --> TOK
PRX --> ESC
PRX --> COM
PRX --> INS
PRX --> BRG
PRX --> ORA
PRX --> FRA
```

**Diagram sources**
- [architecture.md:47-101](file://stellar-insured-contracts/docs/architecture.md#L47-L101)
- [lib.rs:50-97](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L50-L97)
- [lib.rs:19-25](file://stellar-insured-contracts/contracts/proxy/src/lib.rs#L19-L25)

## Detailed Component Analysis

### Property Registry
The Property Registry centralizes property lifecycle management with:
- Storage for properties, owners, approvals, escrows, badges, verifications, appeals, and pause state.
- Events for property registration, transfers, metadata updates, approvals, and governance actions.
- Compliance integration and optional oracle/fee manager linkage.

```mermaid
classDiagram
class PropertyRegistry {
+Mapping properties
+Mapping owner_properties
+Mapping property_owners
+Mapping approvals
+u64 property_count
+u32 version
+AccountId admin
+Mapping escrows
+u64 escrow_count
+GasTracker gas_tracker
+Option<AccountId> compliance_registry
+Mapping property_badges
+Mapping badge_verifiers
+Mapping verification_requests
+u64 verification_count
+Mapping appeals
+u64 appeal_count
+PauseInfo pause_info
+Mapping pause_guardians
+Option<AccountId> oracle
+Option<AccountId> fee_manager
+Mapping fractional
}
class GasTracker {
+u64 total_gas_used
+u64 operation_count
+u64 last_operation_gas
+u64 min_gas_used
+u64 max_gas_used
}
class PauseInfo {
+bool paused
+Option<u64> paused_at
+Option<AccountId> paused_by
+Option<String> reason
+Option<u64> auto_resume_at
+bool resume_request_active
+Option<AccountId> resume_requester
+Vec<AccountId> resume_approvals
+u32 required_approvals
}
PropertyRegistry --> GasTracker : "uses"
PropertyRegistry --> PauseInfo : "uses"
```

**Diagram sources**
- [lib.rs:50-97](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L50-L97)
- [lib.rs:180-204](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L180-L204)
- [lib.rs:312-329](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L312-L329)

**Section sources**
- [lib.rs:50-97](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L50-L97)
- [lib.rs:331-750](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L331-L750)

### Property Token
The Property Token implements ERC-721/1155 compatibility with real estate enhancements:
- Token ownership, approvals, and batch operations.
- Property metadata, compliance flags, legal documents.
- Cross-chain bridging with multi-signature requests and recovery actions.
- Fractional shares, dividends, voting, and tax reporting.

```mermaid
classDiagram
class PropertyToken {
+Mapping token_owner
+Mapping owner_token_count
+Mapping token_approvals
+Mapping operator_approvals
+Mapping balances
+Mapping operators
+Mapping token_properties
+Mapping property_tokens
+Mapping ownership_history_count
+Mapping ownership_history_items
+Mapping compliance_flags
+Mapping legal_documents_count
+Mapping legal_documents_items
+Mapping bridged_tokens
+Vec<AccountId> bridge_operators
+Mapping bridge_requests
+Mapping bridge_transactions
+BridgeConfig bridge_config
+Mapping verified_bridge_hashes
+u64 bridge_request_counter
+u64 total_supply
+u64 token_counter
+AccountId admin
+Mapping error_counts
+Mapping error_rates
+Mapping recent_errors
+u64 error_log_counter
+Mapping total_shares
+Mapping dividends_per_share
+Mapping dividend_credit
+Mapping dividend_balance
+Mapping proposal_counter
+Mapping proposals
+Mapping votes_cast
+Mapping asks
+Mapping escrowed_shares
+Mapping last_trade_price
+Option<AccountId> compliance_registry
+Mapping tax_records
}
```

**Diagram sources**
- [lib.rs:47-102](file://stellar-insured-contracts/contracts/property-token/src/lib.rs#L47-L102)

**Section sources**
- [lib.rs:47-102](file://stellar-insured-contracts/contracts/property-token/src/lib.rs#L47-L102)
- [lib.rs:260-476](file://stellar-insured-contracts/contracts/property-token/src/lib.rs#L260-L476)

### Escrow
The Advanced Escrow supports multi-signature approvals, time locks, conditions, disputes, and audit trails:
- EscrowData with status tracking and participant management.
- MultiSigConfig for required signatures and signers.
- Document hashes and conditions with verification and met tracking.
- Disputes and emergency overrides.

```mermaid
classDiagram
class AdvancedEscrow {
+Mapping escrows
+u64 escrow_count
+Mapping multi_sig_configs
+Mapping signatures
+Mapping signature_counts
+Mapping documents
+Mapping conditions
+Mapping condition_counters
+Mapping disputes
+Mapping audit_logs
+AccountId admin
+u128 min_high_value_threshold
}
class EscrowData {
+u64 id
+u64 property_id
+AccountId buyer
+AccountId seller
+u128 amount
+u128 deposited_amount
+EscrowStatus status
+u64 created_at
+Option<u64> release_time_lock
+Vec<AccountId> participants
}
class MultiSigConfig {
+u8 required_signatures
+Vec<AccountId> signers
}
AdvancedEscrow --> EscrowData : "stores"
AdvancedEscrow --> MultiSigConfig : "stores"
```

**Diagram sources**
- [lib.rs:135-162](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L135-L162)
- [lib.rs:58-73](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L58-L73)

**Section sources**
- [lib.rs:135-162](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L135-L162)
- [lib.rs:164-800](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L164-L800)

### Compliance Registry
The Compliance Registry manages KYC/AML/sanctions and GDPR consent:
- Jurisdiction-specific rules and risk factors.
- Verification status tracking and audit logs.
- Consent management and data retention enforcement.
- Integration with Property Token and Property Registry.

```mermaid
classDiagram
class ComplianceRegistry {
+AccountId owner
+Mapping verifiers
+Mapping compliance_data
+Mapping jurisdiction_rules
+Mapping audit_logs
+Mapping audit_log_count
+Mapping retention_policies
+Mapping encrypted_data_hashes
+Mapping verification_requests
+u64 request_counter
+Mapping service_providers
+Mapping account_requests
+Option<AccountId> zk_compliance_contract
}
```

**Diagram sources**
- [lib.rs:213-241](file://stellar-insured-contracts/contracts/compliance_registry/lib.rs#L213-L241)

**Section sources**
- [lib.rs:383-800](file://stellar-insured-contracts/contracts/compliance_registry/lib.rs#L383-L800)

### Insurance
The Insurance platform provides risk pooling, claims management, and reinsurance:
- RiskPool with capital, exposure limits, and liquidity provider tracking.
- InsurancePolicy and InsuranceClaim with status tracking and evidence metadata.
- ReinsuranceAgreement and ActuarialModel for risk modeling.
- Dispute resolution and governance controls.

```mermaid
classDiagram
class PropertyInsurance {
+AccountId admin
+Mapping policies
+u64 policy_count
+Mapping claims
+u64 claim_count
+Mapping pools
+u64 pool_count
+Mapping risk_assessments
+Mapping reinsurance_agreements
+u64 reinsurance_count
+Mapping insurance_tokens
+u64 token_count
+Vec<u64> token_listings
+Mapping actuarial_models
+u64 model_count
+Mapping underwriting_criteria
+Mapping liquidity_providers
+Mapping authorized_oracles
+Mapping authorized_assessors
+Mapping claim_cooldowns
+u32 platform_fee_rate
+u64 claim_cooldown_period
+u128 min_pool_capital
+u64 dispute_window_seconds
+Option<AccountId> arbiter
}
```

**Diagram sources**
- [lib.rs:322-379](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L322-L379)

**Section sources**
- [lib.rs:528-800](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L528-L800)

### Bridge
The Bridge enables multi-signature cross-chain property token transfers:
- MultisigBridgeRequest with required signatures and expiration.
- BridgeTransaction tracking and verification.
- Gas estimation and monitoring with recovery actions.

```mermaid
classDiagram
class PropertyBridge {
+BridgeConfig config
+Mapping bridge_requests
+Mapping bridge_history
+Mapping chain_info
+Mapping verified_transactions
+Vec<AccountId> bridge_operators
+u64 request_counter
+u64 transaction_counter
+AccountId admin
}
class MultisigBridgeRequest {
+u64 request_id
+u64 token_id
+u64 source_chain
+u64 destination_chain
+AccountId sender
+AccountId recipient
+u8 required_signatures
+Vec<AccountId> signatures
+u64 created_at
+Option<u64> expires_at
+BridgeOperationStatus status
+PropertyMetadata metadata
}
PropertyBridge --> MultisigBridgeRequest : "stores"
```

**Diagram sources**
- [lib.rs:32-61](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L32-L61)
- [lib.rs:612-631](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L612-L631)

**Section sources**
- [lib.rs:115-592](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L115-L592)

### Oracle
The Oracle aggregates property valuations from multiple sources with confidence metrics:
- PropertyValuation with confidence and volatility.
- OracleSource reputation and slashing.
- Price alerts and anomaly detection.

```mermaid
classDiagram
class PropertyValuationOracle {
+AccountId admin
+Mapping property_valuations
+Mapping historical_valuations
+Mapping oracle_sources
+Vec<String> active_sources
+Mapping price_alerts
+Mapping location_adjustments
+Mapping market_trends
+Mapping comparable_cache
+u64 max_price_staleness
+u32 min_sources_required
+u32 outlier_threshold
+Mapping source_reputations
+Mapping source_stakes
+Mapping pending_requests
+u64 request_id_counter
+Option<AccountId> ai_valuation_contract
}
```

**Diagram sources**
- [lib.rs:22-75](file://stellar-insured-contracts/contracts/oracle/src/lib.rs#L22-L75)

**Section sources**
- [lib.rs:105-785](file://stellar-insured-contracts/contracts/oracle/src/lib.rs#L105-L785)

### Fractional
The Fractional contract provides lightweight aggregation and reporting:
- PortfolioItem aggregation with last prices.
- TaxReport summarization for dividends and proceeds.

```mermaid
classDiagram
class Fractional {
+Mapping last_prices
}
```

**Diagram sources**
- [lib.rs:55-58](file://stellar-insured-contracts/contracts/fractional/src/lib.rs#L55-L58)

**Section sources**
- [lib.rs:60-118](file://stellar-insured-contracts/contracts/fractional/src/lib.rs#L60-L118)

### Proxy
The Transparent Proxy enables upgradability by delegating calls to an implementation contract:
- Admin-controlled upgrade and admin change.
- Event emission for upgrades and admin changes.

```mermaid
classDiagram
class TransparentProxy {
+Hash code_hash
+AccountId admin
}
```

**Diagram sources**
- [lib.rs:19-25](file://stellar-insured-contracts/contracts/proxy/src/lib.rs#L19-L25)

**Section sources**
- [lib.rs:39-80](file://stellar-insured-contracts/contracts/proxy/src/lib.rs#L39-L80)

## Dependency Analysis
Contracts depend on shared traits and each other through cross-contract calls and registries:
- Shared traits define common types, events, and interfaces used by all contracts.
- Property Token depends on Compliance Registry and Oracle for compliance and valuation.
- Property Registry coordinates with Compliance Registry and Oracle.
- Insurance integrates with Oracle for risk assessment.
- Bridge interacts with Property Token for locking/minting and with external chains.
- Proxy enables upgradable deployments of core contracts.

```mermaid
graph LR
TR["traits"] --> REG["property-registry"]
TR --> TOK["property-token"]
TR --> ESC["escrow"]
TR --> CR["compliance-registry"]
TR --> INS["insurance"]
TR --> BR["bridge"]
TR --> ORA["oracle"]
TR --> FRA["fractional"]
TR --> PRX["proxy"]
TOK --> CR
TOK --> ORA
REG --> CR
REG --> ORA
INS --> ORA
BR --> TOK
PRX --> REG
PRX --> TOK
PRX --> ESC
PRX --> CR
PRX --> INS
PRX --> BR
PRX --> ORA
PRX --> FRA
```

**Diagram sources**
- [lib.rs:23-722](file://stellar-insured-contracts/contracts/traits/src/lib.rs#L23-L722)
- [lib.rs:50-97](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L50-L97)
- [lib.rs:47-102](file://stellar-insured-contracts/contracts/property-token/src/lib.rs#L47-L102)
- [lib.rs:11-162](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L11-L162)
- [lib.rs:213-241](file://stellar-insured-contracts/contracts/compliance_registry/lib.rs#L213-L241)
- [lib.rs:322-379](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L322-L379)
- [lib.rs:32-61](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L32-L61)
- [lib.rs:22-75](file://stellar-insured-contracts/contracts/oracle/src/lib.rs#L22-L75)
- [lib.rs:55-58](file://stellar-insured-contracts/contracts/fractional/src/lib.rs#L55-L58)
- [lib.rs:19-25](file://stellar-insured-contracts/contracts/proxy/src/lib.rs#L19-L25)

**Section sources**
- [lib.rs:23-722](file://stellar-insured-contracts/contracts/traits/src/lib.rs#L23-L722)

## Performance Considerations
- Efficient storage: Mappings for O(1) lookups; compact encodings; lazy evaluation for expensive computations.
- Batch operations: Use batch transfer and batch bridge operations to reduce gas.
- Off-chain storage: Store large metadata on IPFS and keep only hashes on-chain.
- Caching: On-chain gas tracking and off-chain indexing for complex queries.
- Gas optimization: Minimize state writes, use minimal storage operations, and leverage event-based cache invalidation.

[No sources needed since this section provides general guidance]

## Security Architecture
- Role-based access control: Admin, agents, owners, and public roles with permission matrices.
- Multi-signature workflows: Required approvals for high-value operations and emergency overrides.
- Pause/resume governance: Controlled pausing with guardian approvals and auto-resume windows.
- Compliance enforcement: Mandatory checks before transfers and operations.
- Reentrancy protection: Guard patterns to prevent recursive calls.
- Slashing and reputation: Oracle source reputation and stake-based penalties.
- Formal verification: Kani proofs for critical properties.
- Automated security pipeline: Static analysis, dependency scanning, and vulnerability checks.

```mermaid
flowchart TD
Start(["Operation Request"]) --> CheckRole["Check Role & Permissions"]
CheckRole --> RoleOK{"Authorized?"}
RoleOK --> |No| Deny["Deny Operation"]
RoleOK --> |Yes| CheckCompliance["Check Compliance"]
CheckCompliance --> Compliant{"Compliant?"}
Compliant --> |No| Deny
Compliant --> |Yes| CheckPause["Check Contract Paused"]
CheckPause --> Paused{"Paused?"}
Paused --> |Yes| Deny
Paused --> |No| CheckMultiSig["Check Multi-Signature Threshold"]
CheckMultiSig --> SigOK{"Threshold Met?"}
SigOK --> |No| Hold["Hold Until Approval"]
SigOK --> |Yes| Execute["Execute Operation"]
Execute --> End(["Success"])
Deny --> End
Hold --> End
```

**Diagram sources**
- [architecture.md:203-266](file://stellar-insured-contracts/docs/architecture.md#L203-L266)
- [lib.rs:312-329](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L312-L329)

**Section sources**
- [architecture.md:203-266](file://stellar-insured-contracts/docs/architecture.md#L203-L266)
- [security_pipeline.md:1-58](file://stellar-insured-contracts/docs/security_pipeline.md#L1-L58)

## Event Emission System
All contracts emit structured events for transparency and off-chain indexing:
- Property Registry: Registration, transfer, metadata updates, approvals, badges, verifications, appeals, and governance events.
- Property Token: Transfers, approvals, minting, legal documents, compliance verification, bridging, and governance events.
- Escrow: Creation, funding, release/refund, document upload/verification, conditions, signatures, disputes, and emergency overrides.
- Compliance Registry: Verification updates, compliance checks, consent updates, retention expiration, and audit logs.
- Insurance: Policy creation/cancellation, claims submission/approval/rejection/payout, pool capitalization, reinsurance activation, token minting/transfers, risk assessment updates, and dispute resolution.
- Bridge: Request creation/signing, execution, failure, recovery, and monitoring events.
- Oracle: Valuation updates, price alerts, and source additions.
- Fractional: Last price updates, portfolio aggregation, and tax report summaries.

```mermaid
sequenceDiagram
participant Client as "Client"
participant Registry as "Property Registry"
participant Token as "Property Token"
participant Escrow as "Escrow"
participant Bridge as "Bridge"
Client->>Registry : register_property(metadata)
Registry-->>Client : PropertyRegistered event
Client->>Token : transfer_from(from,to,id)
Token-->>Client : Transfer event
Client->>Escrow : create_escrow_advanced(...)
Escrow-->>Client : EscrowCreated event
Client->>Bridge : initiate_bridge_multisig(...)
Bridge-->>Client : BridgeRequestCreated event
```

**Diagram sources**
- [lib.rs:331-575](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L331-L575)
- [lib.rs:260-476](file://stellar-insured-contracts/contracts/property-token/src/lib.rs#L260-L476)
- [lib.rs:164-260](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L164-L260)
- [lib.rs:63-113](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L63-L113)

**Section sources**
- [lib.rs:331-750](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L331-L750)
- [lib.rs:260-476](file://stellar-insured-contracts/contracts/property-token/src/lib.rs#L260-L476)
- [lib.rs:164-260](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L164-L260)
- [lib.rs:63-113](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L63-L113)
- [lib.rs:77-103](file://stellar-insured-contracts/contracts/oracle/src/lib.rs#L77-L103)
- [lib.rs:382-522](file://stellar-insured-contracts/contracts/insurance/src/lib.rs#L382-L522)

## Upgrade and Proxy Mechanisms
The system employs a transparent proxy pattern for upgradable implementations:
- TransparentProxy stores the current implementation code hash and admin.
- Admin-only upgrades replace the implementation and emit an Upgraded event.
- Admin changes are similarly restricted and emit an AdminChanged event.
- Contracts can be upgraded independently or as part of a coordinated upgrade strategy.

```mermaid
sequenceDiagram
participant Owner as "Owner/Admin"
participant Proxy as "TransparentProxy"
participant Impl as "Implementation"
Owner->>Proxy : upgrade_to(new_code_hash)
Proxy->>Proxy : ensure_admin()
Proxy->>Proxy : code_hash = new_code_hash
Proxy-->>Owner : Upgraded event
Owner->>Proxy : change_admin(new_admin)
Proxy->>Proxy : ensure_admin()
Proxy->>Proxy : admin = new_admin
Proxy-->>Owner : AdminChanged event
```

**Diagram sources**
- [lib.rs:39-80](file://stellar-insured-contracts/contracts/proxy/src/lib.rs#L39-L80)

**Section sources**
- [lib.rs:39-80](file://stellar-insured-contracts/contracts/proxy/src/lib.rs#L39-L80)
- [architecture.md:323-348](file://stellar-insured-contracts/docs/architecture.md#L323-L348)

## Scalability and Monitoring
- Scalability solutions: Layer 2 integration, rollups, sidechains, and cross-chain compatibility.
- Monitoring: On-chain gas tracking, performance metrics, health checks, and off-chain alerting.
- Gas optimization: Efficient data structures, batch operations, lazy evaluation, and minimal storage writes.
- Observability: Comprehensive event emission, error logging, and audit trails.

**Section sources**
- [architecture.md:365-431](file://stellar-insured-contracts/docs/architecture.md#L365-L431)
- [lib.rs:180-204](file://stellar-insured-contracts/contracts/lib/src/lib.rs#L180-L204)
- [best-practices.md:25-45](file://stellar-insured-contracts/docs/best-practices.md#L25-L45)

## Integration and Extensibility
- Shared traits enable new contracts to integrate seamlessly with existing components.
- Cross-contract calls facilitate orchestration between Property Token, Registry, Compliance, Insurance, and Bridge.
- Modular design allows incremental feature addition and independent upgrades.
- Best practices emphasize compliance checks, multi-signature workflows, and gas-efficient operations.

**Section sources**
- [lib.rs:23-722](file://stellar-insured-contracts/contracts/traits/src/lib.rs#L23-L722)
- [best-practices.md:47-56](file://stellar-insured-contracts/docs/best-practices.md#L47-L56)

## Troubleshooting Guide
Common issues and resolutions:
- Compliance failures: Ensure accounts meet jurisdictional requirements and GDPR consent is valid.
- Escrow disputes: Use dispute resolution workflows and emergency overrides when applicable.
- Bridge failures: Monitor request status, verify signatures, and apply recovery actions as admin.
- Oracle anomalies: Check source reputation, slashing thresholds, and confidence metrics.
- Gas optimization: Batch operations, minimize state changes, and use off-chain metadata.

**Section sources**
- [lib.rs:603-635](file://stellar-insured-contracts/contracts/compliance_registry/lib.rs#L603-L635)
- [lib.rs:760-800](file://stellar-insured-contracts/contracts/escrow/src/lib.rs#L760-L800)
- [lib.rs:349-404](file://stellar-insured-contracts/contracts/bridge/src/lib.rs#L349-L404)
- [lib.rs:311-327](file://stellar-insured-contracts/contracts/oracle/src/lib.rs#L311-L327)
- [best-practices.md:22-45](file://stellar-insured-contracts/docs/best-practices.md#L22-L45)

## Conclusion
The PropChain smart contract system demonstrates a mature, modular architecture designed for real estate tokenization. Its layered design, robust governance, comprehensive event emission, and upgrade mechanisms provide a solid foundation for scalable, secure, and interoperable property ecosystems. By adhering to best practices and leveraging the shared traits and proxy infrastructure, developers can extend the system while maintaining consistency and reliability.