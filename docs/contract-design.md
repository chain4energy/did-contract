# DID Contract Design

## Overview

The DID (Decentralized Identifier) Contract is a CosmWasm smart contract implementing a simplified W3C DID specification for the Chain4Energy blockchain. It provides a decentralized identity management system that allows creation, management, and verification of digital identities without relying on centralized authorities.

## Architecture

### Core Components

#### 1. DID Documents
DID Documents are the foundational data structures that contain identity information:
- **Unique Identifier**: Each DID has a unique identifier following the format `did:c4e:<identifier>`
- **Controllers**: Entities that can modify the DID document
- **Services**: Endpoint information for interacting with the DID subject

#### 2. Controller Management
The contract implements a sophisticated controller system:
- **Multi-level Control**: DIDs can be controlled by addresses or other DIDs
- **Recursive Resolution**: Controller authorization is resolved recursively through DID chains
- **Access Control**: Only authorized controllers can modify DID documents

#### 3. Service Registry
Services provide interaction endpoints:
- **Service Types**: Categorized service endpoints (MessagingService, AuthService, etc.)
- **Endpoint URLs**: Service access points
- **Service Lifecycle**: Add/remove services dynamically

### Storage Architecture

The contract uses a dual-storage pattern for optimal query performance:

#### Primary Storage
```rust
did_docs: Map<String, DidDocument>
```
- **Key**: DID identifier string
- **Value**: Complete DID document
- **Purpose**: Direct document retrieval

#### Controller Index
```rust
controllers: MultiSet
```
- **Structure**: Multi-value mapping from controllers to controlled DIDs
- **Purpose**: Efficient reverse lookups (find all DIDs controlled by an entity)
- **Implementation**: Uses custom MultiSet for one-to-many relationships

### Design Patterns

#### 1. Validation Pipeline
All operations follow a comprehensive validation pattern:
```
Input Validation → Business Logic Validation → Storage Validation → Execution
```

#### 2. Event-Driven Architecture
Each operation emits structured events for external monitoring:
- Document lifecycle events
- Controller change events
- Service modification events

#### 3. Recursive Authorization
Controller verification follows a recursive pattern to handle DID-controlled DIDs:
```
is_controlled_by(did, controller) →
  if controller is address: check direct control
  if controller is DID: recursively verify controller's authorization
```

## Technical Decisions

### 1. Framework Choice: Sylvia
**Decision**: Use Sylvia 1.3.5 framework over raw CosmWasm
**Rationale**:
- Cleaner code organization with attribute-based routing
- Automatic schema generation
- Better error handling patterns
- Reduced boilerplate code

### 2. Storage Strategy
**Decision**: Dual storage with primary documents and controller index
**Rationale**:
- Optimizes both direct lookups and reverse queries
- Supports complex authorization patterns
- Maintains data consistency

### 3. DID Format
**Decision**: Use `did:c4e:` prefix with configurable identifiers
**Rationale**:
- Follows W3C DID specification
- Chain-specific namespace prevents conflicts
- Future extensibility for different identifier types

### 4. Controller Model
**Decision**: Support both address and DID controllers
**Rationale**:
- Enables hierarchical identity structures
- Supports complex organizational models
- Maintains backward compatibility with simple address-based control

## Security Considerations

### 1. Input Validation
- **DID Format Validation**: Strict adherence to `did:c4e:` format
- **Address Validation**: Cosmos address format verification
- **Data Sanitization**: Prevention of malicious data injection

### 2. Authorization Checks
- **Controller Verification**: Recursive authorization for complex hierarchies
- **Circular Reference Prevention**: Detects and prevents self-controlled DIDs
- **Permission Boundaries**: Controllers can only modify documents they control

### 3. State Consistency
- **Atomic Operations**: All state changes are atomic
- **Index Synchronization**: Controller indices are always synchronized with documents
- **Duplicate Prevention**: Prevents duplicate controllers and services

### 4. DoS Protection
- **Recursion Limits**: Prevents infinite loops in controller resolution
- **Input Size Limits**: Reasonable limits on document size
- **Gas Efficiency**: Optimized storage access patterns

## Data Flow

### Document Creation
```
1. Validate DID format and uniqueness
2. Verify controller existence (if DID controller)
3. Check for duplicates (controllers/services)
4. Store document in primary storage
5. Update controller index
6. Emit creation event
```

### Document Updates
```
1. Verify caller authorization
2. Validate new document structure
3. Update primary storage
4. Rebuild controller index
5. Emit update event with changes
```

### Controller Operations
```
1. Verify caller authorization on target DID
2. Validate new controller format
3. Check for duplicates and circular references
4. Update document and index
5. Emit controller change event
```

## Query Optimization

### 1. Direct Lookups
- Primary key access for document retrieval: `O(1)`
- Optimized for most common query pattern

### 2. Controller Queries
- Indexed access for controlled DIDs: `O(log n)`
- Supports pagination for large result sets
- Efficient reverse relationship queries

### 3. Existence Checks
- Fast boolean operations for authorization
- Cached validation results where possible
- Minimal storage reads for common checks

## Scalability Features

### 1. Pagination Support
All list queries support pagination:
- `limit`: Maximum results per query
- `start_after`: Cursor-based pagination
- Prevents large response payloads

### 2. Efficient Indexing
- MultiSet storage for one-to-many relationships
- Optimized key structures for range queries
- Minimal storage overhead

### 3. Event Optimization
- Structured event data for external indexing
- Minimal event payload sizes
- Consistent event schemas

## Extension Points

### 1. Service Types
The contract supports arbitrary service types:
- Extensible type system
- Custom service endpoint formats
- Future service discovery mechanisms

### 2. DID Method Extensions
Framework for future DID method support:
- Pluggable identifier validation
- Custom resolution methods
- Cross-chain DID integration

### 3. Authorization Models
Flexible authorization framework:
- Custom controller validation
- Plugin-based permission systems
- Role-based access control extensions

## Testing Strategy

### 1. Unit Tests
- Function-level validation
- Error condition coverage
- Edge case handling

### 2. Integration Tests
- End-to-end workflows
- Multi-contract interactions
- State consistency verification

### 3. Property Testing
- Invariant verification
- Fuzzing for edge cases
- Performance benchmarking

## Performance Characteristics

### Operation Complexity
- **Document Creation**: `O(n)` where n = number of controllers
- **Document Retrieval**: `O(1)` for direct access
- **Controller Queries**: `O(log n + m)` where m = result size
- **Authorization Checks**: `O(d)` where d = controller chain depth

### Gas Costs
- **Storage Operations**: ~5,000 gas per KB stored
- **Index Updates**: ~1,000 gas per controller relationship
- **Query Operations**: ~100-500 gas depending on complexity

### Memory Usage
- **Document Storage**: ~1-5 KB per DID document
- **Index Overhead**: ~100 bytes per controller relationship
- **Query Buffers**: Configurable pagination limits

## Future Enhancements

### 1. Advanced Features
- DID document versioning
- Cryptographic key management
- Cross-chain DID resolution
- Advanced service discovery

### 2. Performance Optimizations
- Batch operations
- Compressed storage formats
- Advanced indexing strategies
- Cache layers for frequent queries

### 3. Ecosystem Integration
- DID resolver networks
- Identity verification protocols
- Credential issuance systems
- Privacy-preserving features