# Architecture

This document describes the technical architecture of the PropChain smart contract system, including design patterns, data structures, and integration points.

## System Overview

PropChain is built on the Substrate blockchain framework using the ink! smart contract language. The system consists of multiple interconnected contracts that provide a comprehensive real estate tokenization platform.

## Core Components

### Smart Contract Layer

#### Property Registry Contract
- **Purpose**: Central registry for all tokenized properties
- **Storage**: Property metadata, ownership records, transfer history
- **Key Features**: Property registration, ownership verification, metadata management

#### Escrow Contract
- **Purpose**: Secure transfer of property ownership
- **Storage**: Escrow agreements, fund locks, release conditions
- **Key Features**: Multi-signature releases, time-locked transactions, dispute resolution

#### Token Contract
- **Purpose**: ERC-721 compatible NFT representation of properties
- **Storage**: Token ownership, transfer records, approval mechanisms
- **Key Features**: Fractional ownership, transfer restrictions, royalty enforcement

#### Oracle Contract
- **Purpose**: External data integration for property valuations
- **Storage**: Price feeds, valuation sources, confidence scores
- **Key Features**: Multiple oracle sources, price aggregation, outlier detection

### Data Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend UI   │───▶│  Gateway API    │───▶│  Smart Contract │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
│   Off-chain Storage │    │   Blockchain    │
│   (IPFS/Arweave)   │    │   (Substrate)   │
└─────────────────┘    └─────────────────┘
```

## Contract Architecture

### Property Registry

```rust
#[ink(storage)]
pub struct PropertyRegistry {
    /// Mapping from property ID to property information
    properties: Mapping<PropertyId, PropertyInfo>,
    
    /// Mapping from owner to their properties
    owner_properties: Mapping<AccountId, Vec<PropertyId>>,
    
    /// Registry configuration
    config: RegistryConfig,
    
    /// Access control mappings
    admins: Mapping<AccountId, bool>,
    agents: Mapping<AccountId, bool>,
}
```

#### Key Design Patterns

1. **Singleton Pattern**: Single registry instance
2. **Factory Pattern**: Property creation through standardized methods
3. **Observer Pattern**: Event emission for state changes
4. **Access Control**: Role-based permissions

### Escrow Contract

```rust
#[ink(storage)]
pub struct EscrowContract {
    /// Active escrow agreements
    escrows: Mapping<EscrowId, EscrowInfo>,
    
    /// User's active escrows
    user_escrows: Mapping<AccountId, Vec<EscrowId>>,
    
    /// Property escrow history
    property_escrows: Mapping<PropertyId, Vec<EscrowId>>,
    
    /// Contract configuration
    config: EscrowConfig,
}
```

#### Security Features

1. **Time Locks**: Prevent premature fund release
2. **Multi-signature**: Require multiple approvals
3. **Dispute Resolution**: Mechanism for handling conflicts
4. **Reentrancy Protection**: Prevent recursive calls

## Data Structures

### Property Metadata

```rust
#[derive(Debug, Clone, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct PropertyMetadata {
    /// Physical address
    pub location: Location,
    
    /// Property specifications
    pub specifications: PropertySpecs,
    
    /// Legal information
    pub legal_info: LegalInfo,
    
    /// Valuation details
    pub valuation: ValuationInfo,
    
    /// Document references
    pub documents: Vec<DocumentReference>,
}

#[derive(Debug, Clone, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Location {
    pub address: String,
    pub coordinates: Option<(f64, f64)>,
    pub country: String,
    pub region: String,
    pub postal_code: String,
}
```

### Ownership Structure

```rust
#[derive(Debug, Clone, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct OwnershipInfo {
    pub owner: AccountId,
    pub ownership_type: OwnershipType,
    pub shares: u64,  // For fractional ownership
    pub acquired_at: Timestamp,
    pub purchase_price: Balance,
}

pub enum OwnershipType {
    Full,
    Fractional(u64),  // Number of shares
    Leasehold(u64),   // Lease duration
}
```

## Integration Architecture

### External Services

#### IPFS Integration
- **Purpose**: Decentralized document storage
- **Protocol**: HTTP API calls to IPFS nodes
- **Content**: Property documents, images, legal papers

#### Oracle Integration
- **Purpose**: Real-time property valuations
- **Protocol**: Chainlink-compatible oracle feeds
- **Data**: Market prices, rental yields, appreciation rates

#### KYC/AML Services
- **Purpose**: Identity verification and compliance
- **Protocol**: REST API integration
- **Data**: User verification status, risk assessments

### Frontend Integration

#### React Components
```typescript
interface PropertyRegistryProps {
  contract: ContractPromise;
  account: AccountId;
}

const PropertyRegistry: React.FC<PropertyRegistryProps> = ({
  contract,
  account
}) => {
  // Component implementation
};
```

#### State Management
```typescript
interface AppState {
  properties: PropertyInfo[];
  userProperties: PropertyId[];
  loading: boolean;
  error: string | null;
}
```

## Security Architecture

### Access Control

#### Role-Based Access Control (RBAC)
```rust
pub enum Role {
  Admin,
  Agent,
  Owner,
  Public,
}

#[ink(storage)]
pub struct AccessControl {
    roles: Mapping<AccountId, Role>,
    permissions: Mapping<(Role, Operation), bool>,
}
```

#### Permission Matrix
| Role | Register | Transfer | Escrow | Admin |
|------|----------|----------|--------|-------|
| Admin | ✓ | ✓ | ✓ | ✓ |
| Agent | ✓ | ✓ | ✓ | ✗ |
| Owner | ✗ | ✓ | ✓ | ✗ |
| Public | ✗ | ✗ | ✗ | ✗ |

### Security Measures

#### Input Validation
- Type checking for all parameters
- Range validation for numeric inputs
- Format validation for strings
- Sanitization of user inputs

#### Reentrancy Protection
```rust
#[ink(storage)]
pub struct ReentrancyGuard {
    locked: bool,
}

impl ReentrancyGuard {
    fn begin_reentrancy_check(&mut self) -> Result<(), Error> {
        if self.locked {
            return Err(Error::ReentrantCall);
        }
        self.locked = true;
        Ok(())
    }
    
    fn end_reentrancy_check(&mut self) {
        self.locked = false;
    }
}
```

#### Gas Optimization
- Efficient data structures
- Minimal storage operations
- Batch processing for bulk operations
- Lazy loading of expensive computations

## Performance Architecture

### Storage Optimization

#### Efficient Data Structures
```rust
// Use Mapping instead of Vec for large datasets
properties: Mapping<PropertyId, PropertyInfo>  // O(1) access

// Use packed structs to reduce storage costs
#[derive(scale::Encode, scale::Decode)]
pub struct CompactPropertyInfo {
    pub owner: AccountId,           // 32 bytes
    pub value: Compact<Balance>,    // Variable bytes
    pub flags: u8,                  // 1 byte
}
```

#### Caching Strategy
- On-chain caching for frequently accessed data
- Off-chain indexing for complex queries
- Event-based cache invalidation

### Gas Optimization

#### Batch Operations
```rust
#[ink(message)]
pub fn batch_register_properties(
    &mut self,
    properties: Vec<PropertyMetadata>
) -> Result<Vec<PropertyId>, Error> {
    let mut results = Vec::new();
    for metadata in properties {
        let id = self.register_property_internal(metadata)?;
        results.push(id);
    }
    Ok(results)
}
```

#### Lazy Evaluation
```rust
#[ink(message)]
pub fn get_property_summary(&self, id: PropertyId) -> PropertySummary {
    let property = self.properties.get(&id).unwrap();
    PropertySummary {
        id,
        owner: property.owner,
        value: property.metadata.valuation.amount,
        // Only compute expensive fields when needed
        location: property.metadata.location.address.clone(),
    }
}
```

## Upgrade Architecture

### Proxy Pattern With Timelocked Admin Control

```rust
#[ink(storage)]
pub struct ProxyContract {
    implementation: Hash,
    admin: AccountId,
    pending_admin: Option<AccountId>,
    admin_transfer_requested_at: Option<u64>,
    renounce_requested_at: Option<u64>,
}

impl ProxyContract {
    #[ink(message)]
    pub fn upgrade_to(&mut self, new_implementation: Hash) -> Result<(), Error> { /* admin only */ }

    #[ink(message)]
    pub fn set_admin(&mut self, new_admin: AccountId) -> Result<(), Error> {
        // Starts a timelocked handoff.
    }

    #[ink(message)]
    pub fn accept_admin(&mut self) -> Result<(), Error> {
        // Pending admin accepts after timelock expiry.
    }

    #[ink(message)]
    pub fn renounce_admin(&mut self) -> Result<(), Error> {
        // First call schedules renounce, second call executes after delay.
    }
}
```

Recommended control flow:

1. Current admin calls `set_admin(new_admin)`.
2. The contract records the pending admin and a future acceptance timestamp.
3. The pending admin calls `accept_admin()` only after the timelock has fully elapsed.
4. Upgrades remain callable only by the active admin.
5. `renounce_admin()` uses the same delayed pattern so loss of control is visible on-chain before it becomes final.

Operational guidance:

- Use a fixed on-chain delay of at least `3 days` for admin handoff and renounce flows in development and staging.
- Use a fixed on-chain delay of at least `7 days` for production governance-controlled deployments.
- Record the proposal id, requested timestamp, ETA, and final execution timestamp for every admin or upgrade transition.

### Migration Strategy

#### Data Migration
1. **Snapshot**: Create backup of current state
2. **Transform**: Convert data to new format
3. **Validate**: Verify data integrity
4. **Deploy**: Deploy new contract
5. **Migrate**: Transfer data to new contract
6. **Verify**: Final validation and cleanup

#### Backward Compatibility
- Versioned API endpoints
- Graceful degradation for old clients
- Migration windows and notifications

## Monitoring Architecture

### On-chain Metrics

#### Contract Events
```rust
#[ink(event)]
pub struct PerformanceMetrics {
    gas_used: Balance,
    execution_time: u64,
    operation: Operation,
    timestamp: Timestamp,
}
```

#### Health Checks
```rust
#[ink(message)]
pub fn health_check(&self) -> HealthStatus {
    HealthStatus {
        total_properties: self.property_count,
        active_escrows: self.escrow_count,
        last_operation: self.last_timestamp,
        gas_balance: self.env().balance(),
    }
}
```

### Off-chain Monitoring

#### Metrics Collection
- Gas usage patterns
- Transaction success rates
- Error frequency analysis
- Performance benchmarking

#### Alerting System
- Threshold-based alerts
- Anomaly detection
- Performance degradation warnings
- Security incident notifications

## Future Architecture

### Scalability Solutions

#### Layer 2 Integration
- State channels for frequent transfers
- Rollups for batch operations
- Sidechains for specialized functionality

#### Cross-chain Compatibility
- Bridge contracts for asset transfer
- Standardized interfaces across chains
- Unified governance framework

### Advanced Features

#### AI Integration
- Automated property valuation
- Predictive analytics
- Risk assessment models

#### DeFi Integration
- Property-backed lending
- Yield farming opportunities
- Liquidity pools for real estate tokens

## Related References

- [DAO Risk Architecture](C:\Users\hp\Desktop\wave\Stellar-soroban-contracts\stellar-insured-contracts\docs\dao-risk-architecture.md): formulae, operating thresholds, pseudocode, and dashboard metrics for exposure, coverage, slashing, and liquidity monitoring
