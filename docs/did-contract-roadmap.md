# DID Contract Development Roadmap

## 📋 Overview

This roadmap outlines the evolution of the DID Contract from its current basic implementation to a full W3C DID Core compliant system with key management capabilities. The focus is on enabling secure key rotation and cryptographic signature verification, which are critical for the DeTrack DePIN network.

**Current Status**: Phase 1 Complete ✅  
**Next Priority**: Phase 2a - Key Management 🔥 CRITICAL

---

## 🎯 Strategic Goals

1. **Security**: Enable key rotation to handle compromised keys without DID recreation
2. **Compliance**: Align with W3C DID Core 1.0 specification
3. **Interoperability**: Support multiple cryptographic key types
4. **Scalability**: Efficient signature verification queries for high-throughput systems
5. **Future-Proof**: Extensible architecture for advanced features (multisig, threshold schemes)

---

## 📈 Phase Breakdown

### Phase 1: Basic DID Foundation ✅ COMPLETED

**Status**: Deployed and operational  
**Timeline**: Completed Nov 2025

#### Features Implemented
- ✅ DID Document CRUD operations
- ✅ Controller management (add/remove controllers)
- ✅ Service registry (add/remove services)
- ✅ Multi-level controller hierarchy
- ✅ Recursive authorization resolution
- ✅ Query optimization with MultiSet indexing

#### Current DID Document Structure
```rust
pub struct DidDocument {
    pub id: String,              // DID identifier (did:c4e:...)
    pub controller: Vec<String>, // Controllers (addresses or DIDs)
    pub service: Vec<Service>,   // Service endpoints
}
```

#### Available Operations
- `CreateDidDocument` - Create new DID with controllers
- `UpdateDidDocument` - Update entire document
- `AddController` / `DeleteController` - Manage controllers
- `AddService` / `DeleteService` - Manage service endpoints
- `DeleteDidDocument` - Remove DID entirely

#### Limitations Identified
- ❌ No cryptographic key management
- ❌ No key rotation mechanism
- ❌ No signature verification
- ❌ No verification method relationships
- ❌ Not W3C DID Core compliant (missing verification methods)

---

### Phase 2a: DID Key Management 🔥 CRITICAL

**Priority**: HIGH - Blocks Multi-Gateway security  
**Timeline**: 3 weeks  
**Approach**: Minimal W3C implementation

#### Goals
1. Add VerificationMethod support to DID documents
2. Implement key rotation mechanism
3. Enable on-chain signature verification
4. Maintain backward compatibility with Phase 1

#### New Data Structures

```rust
// Extended DID Document
pub struct DidDocument {
    pub id: String,
    pub controller: Vec<String>,
    pub verification_method: Vec<VerificationMethod>,  // 🆕 NEW
    pub authentication: Vec<String>,                   // 🆕 NEW (Key IDs)
    pub assertion_method: Vec<String>,                 // 🆕 NEW (Key IDs)
    pub service: Vec<Service>,
}

// Verification Method Structure
pub struct VerificationMethod {
    pub id: String,              // "did:c4e:gateway:detrack1#key-1"
    pub vm_type: String,         // "EcdsaSecp256k1VerificationKey2019"
    pub controller: String,      // "did:c4e:gateway:detrack1"
    pub public_key_multibase: String,  // Public key (base58btc encoded)
}

// Verification Relationship (reference to keys)
pub type VerificationRelationship = String; // Key ID reference
```

#### New Execute Messages

```rust
// Add new verification method (public key)
ExecuteMsg::AddVerificationMethod {
    did: String,
    verification_method: VerificationMethod,
}

// Remove verification method
ExecuteMsg::RemoveVerificationMethod {
    did: String,
    key_id: String,  // "did:c4e:gateway:detrack1#key-1"
}

// Update verification method (key rotation)
ExecuteMsg::UpdateVerificationMethod {
    did: String,
    key_id: String,
    new_public_key: String,
}

// Atomic key rotation (recommended method)
ExecuteMsg::RotateKey {
    did: String,
    old_key_id: String,
    new_verification_method: VerificationMethod,
}

// Add key to authentication relationship
ExecuteMsg::AddAuthenticationMethod {
    did: String,
    key_id: String,  // Reference to existing verification_method
}

// Remove key from authentication
ExecuteMsg::RemoveAuthenticationMethod {
    did: String,
    key_id: String,
}

// Add key to assertion_method relationship
ExecuteMsg::AddAssertionMethod {
    did: String,
    key_id: String,
}

// Remove key from assertion_method
ExecuteMsg::RemoveAssertionMethod {
    did: String,
    key_id: String,
}
```

#### New Query Messages

```rust
// Get all verification methods for a DID
QueryMsg::GetVerificationMethods {
    did: String,
}

// Get specific verification method
QueryMsg::GetVerificationMethod {
    did: String,
    key_id: String,
}

// Verify signature (on-chain verification)
QueryMsg::VerifySignature {
    did: String,
    key_id: String,
    message: Vec<u8>,      // Message bytes
    signature: Vec<u8>,    // Signature bytes
    algorithm: String,     // "secp256k1" | "ed25519"
}

// Check if key is authorized for authentication
QueryMsg::IsAuthenticationKey {
    did: String,
    key_id: String,
}

// Check if key is authorized for assertion
QueryMsg::IsAssertionKey {
    did: String,
    key_id: String,
}
```

#### Implementation Details

**Week 1: Data Structures & Storage**
- Extend `DidDocument` struct with new fields
- Implement `VerificationMethod` struct
- Add storage for verification relationships
- Migration strategy for existing DIDs (empty verification_method arrays)
- Update validation logic

**Week 2: Execute Operations**
- Implement `AddVerificationMethod` with validation
  - Validate key_id format (must start with DID + "#")
  - Validate public key encoding (multibase)
  - Check for duplicates
  - Verify controller authorization
- Implement `RemoveVerificationMethod`
  - Prevent removal of last authentication key
  - Clean up relationships (authentication, assertion_method)
- Implement `UpdateVerificationMethod` (key rotation)
- Implement relationship management (Add/Remove Authentication/Assertion)
- Atomic `RotateKey` operation with validation

**Week 3: Query Operations & Signature Verification**
- Implement verification method queries
- Add on-chain signature verification
  - Support secp256k1 (ECDSA)
  - Support ed25519 (future)
  - Decode multibase public keys
  - Verify signature against message
- Add relationship validation queries
- Comprehensive testing
- Documentation updates

#### Security Considerations

1. **Authorization**: Only controllers can add/remove verification methods
2. **Key ID Format**: Must follow `<did>#<key-name>` pattern
3. **Minimum Keys**: At least one authentication key must exist
4. **Relationship Integrity**: Keys must exist in verification_method before adding to relationships
5. **Signature Verification**: Use battle-tested crypto libraries (cosmwasm-crypto)

#### Success Criteria

- [ ] DID documents can store multiple verification methods
- [ ] Keys can be added/removed without recreating DID
- [ ] Signature verification works on-chain (secp256k1)
- [ ] Authentication/assertion relationships properly enforced
- [ ] Existing DIDs migrate seamlessly (empty arrays)
- [ ] All operations covered by unit tests
- [ ] Documentation updated with examples

#### Example Usage

```rust
// 1. Create DID with initial key
CreateDidDocument {
    did_doc: {
        id: "did:c4e:gateway:detrack1",
        controller: ["c4e1xyz..."],
        verification_method: [{
            id: "did:c4e:gateway:detrack1#key-1",
            vm_type: "EcdsaSecp256k1VerificationKey2019",
            controller: "did:c4e:gateway:detrack1",
            public_key_multibase: "zH3C2AVvLMv6gmMNam3uVAjZpfkcJC..."
        }],
        authentication: ["did:c4e:gateway:detrack1#key-1"],
        assertion_method: [],
        service: []
    }
}

// 2. Rotate compromised key
RotateKey {
    did: "did:c4e:gateway:detrack1",
    old_key_id: "did:c4e:gateway:detrack1#key-1",
    new_verification_method: {
        id: "did:c4e:gateway:detrack1#key-2",
        vm_type: "EcdsaSecp256k1VerificationKey2019",
        controller: "did:c4e:gateway:detrack1",
        public_key_multibase: "zNew2PublicKey..."
    }
}
// Result: key-1 removed, key-2 added, authentication updated

// 3. Verify signature
QueryMsg::VerifySignature {
    did: "did:c4e:gateway:detrack1",
    key_id: "did:c4e:gateway:detrack1#key-2",
    message: [0x48, 0x65, 0x6c, 0x6c, 0x6f],  // "Hello"
    signature: [0xAB, 0xCD, ...],
    algorithm: "secp256k1"
}
// Returns: true/false
```

#### Backward Compatibility

- Existing DIDs without verification_method will have empty arrays
- Old queries still work (GetDidDocument returns extended structure)
- Controllers remain unchanged
- Services remain unchanged
- Migration is non-breaking (additive changes only)

---

### Phase 2b: DeTrack Contract DID Integration 🚨 URGENT

**Priority**: HIGH - Depends on Phase 2a  
**Timeline**: 2 weeks (parallel with Phase 2a Week 3)

#### Goals
1. Update DeTrack Worker Contract to use DID verification methods
2. Store gateway_key_id in proofs
3. Verify gateway signatures on-chain
4. Update Worker Node to track key IDs

#### DeTrack Contract Changes

```rust
// Enhanced Proof structure
pub struct Proof {
    pub id: u64,
    pub data_hash: String,
    pub gateway_did: String,
    pub gateway_key_id: String,     // 🆕 "did:c4e:gateway:detrack1#key-1"
    pub gateway_signature: Vec<u8>, // 🆕 Signature bytes
    pub worker_did: String,         // 🆕 Worker DID
    pub worker_key_id: String,      // 🆕 Worker key ID
    pub batch_ids: Vec<String>,
    // ... existing fields
}

// Config update
pub struct Config {
    pub admin: Addr,
    pub did_contract: Addr,         // 🆕 Reference to DID Contract
    // ... existing fields
}

// Enhanced execute message
ExecuteMsg::StoreProof {
    data_hash: String,
    gateway_did: String,
    gateway_key_id: String,         // 🆕
    gateway_signature: Vec<u8>,     // 🆕
    batch_metadata: Vec<BatchMetadata>,
    worker_signature: Vec<u8>,      // 🆕
    // ... existing fields
}
```

#### Verification Flow

```rust
fn store_proof(
    deps: DepsMut,
    info: MessageInfo,
    msg: StoreProofMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // 1. Verify gateway signature
    let gateway_sig_valid: bool = deps.querier.query_wasm_smart(
        config.did_contract.clone(),
        &DidQueryMsg::VerifySignature {
            did: msg.gateway_did.clone(),
            key_id: msg.gateway_key_id.clone(),
            message: compute_batch_hash(&msg.batch_metadata),
            signature: msg.gateway_signature.clone(),
            algorithm: "secp256k1".to_string(),
        }
    )?;
    
    if !gateway_sig_valid {
        return Err(ContractError::InvalidGatewaySignature);
    }
    
    // 2. Verify worker signature
    let worker_sig_valid: bool = deps.querier.query_wasm_smart(
        config.did_contract.clone(),
        &DidQueryMsg::VerifySignature {
            did: extract_worker_did(&info.sender)?,
            key_id: msg.worker_key_id.clone(),
            message: msg.data_hash.as_bytes().to_vec(),
            signature: msg.worker_signature.clone(),
            algorithm: "secp256k1".to_string(),
        }
    )?;
    
    if !worker_sig_valid {
        return Err(ContractError::InvalidWorkerSignature);
    }
    
    // 3. Store proof with key IDs
    let proof = Proof {
        gateway_key_id: msg.gateway_key_id,
        gateway_signature: msg.gateway_signature,
        worker_key_id: msg.worker_key_id,
        // ... store all data
    };
    
    // ... proceed with storage
}
```

#### Worker Node Changes

```typescript
// Track active key ID
class WorkerNode {
  private workerDid: string;
  private workerKeyId: string;  // 🆕 "did:c4e:worker:detrack1#key-1"
  
  async submitProof(aggregation: Aggregation) {
    // Sign with current key
    const workerSignature = await this.signPayload(
      aggregation.root,
      this.workerKeyId
    );
    
    // Submit with key tracking
    await cosmosClient.storeProof({
      data_hash: aggregation.root,
      gateway_did: aggregation.gateway_did,
      gateway_key_id: aggregation.gateway_key_id,  // 🆕
      gateway_signature: aggregation.gateway_signature,  // 🆕
      batch_metadata: aggregation.batches,
      worker_signature: workerSignature,  // 🆕
    });
  }
}
```

#### Gateway Changes

```typescript
// Track active key ID in batch
interface SignedBatch {
  batch_id: string;
  merkle_root: string;
  measurements: Measurement[];
  gateway_did: string;
  gateway_key_id: string;    // 🆕 Which key signed this
  signature: string;
}

async function signBatch(batch: Batch): Promise<SignedBatch> {
  const activeKeyId = await getActiveKeyId(GATEWAY_DID);
  const signature = await signData(batch.merkle_root, activeKeyId);
  
  return {
    ...batch,
    gateway_key_id: activeKeyId,
    signature,
  };
}
```

#### Success Criteria

- [ ] DeTrack contract verifies gateway signatures on-chain
- [ ] Worker Node tracks and submits key IDs
- [ ] Gateway signs batches with tracked key ID
- [ ] Proof storage includes cryptographic audit trail
- [ ] Key rotation doesn't break existing proofs

---

### Phase 3: Multi-Gateway Aggregation 🔥 HIGH PRIORITY

**Timeline**: 4-5 weeks (after Phase 2b)  
**Depends on**: Phase 2a, Phase 2b

#### Goals
See [PHASE-IMPLEMENTATION-ANALYSIS.md](../../detrack-worker-node/docs/PHASE-IMPLEMENTATION-ANALYSIS.md) for detailed specification.

**DID Contract Impact**: Minimal - already supports multiple DIDs
- Each gateway has own DID with verification methods
- Worker Node verifies each gateway's signature independently
- Proof structure supports multiple gateway_dids

---

### Phase 4: Device NFT Ownership 📦

**Timeline**: 3-4 weeks (after Phase 3)

#### DID Contract Enhancements (Optional)

```rust
// Potential extension: Link NFT to DID
pub struct DidDocument {
    // ... existing fields
    pub linked_nft: Option<NftReference>,  // 🆕 Optional
}

pub struct NftReference {
    pub contract: Addr,
    pub token_id: String,
}

// Query if DID owns specific NFT
QueryMsg::VerifyNftOwnership {
    did: String,
    nft_contract: String,
    token_id: String,
}
```

**Note**: This may not be needed if Linkage Contract handles NFT ↔ DID mapping.

---

### Phase 5: Linkage Contract 🔗

**Timeline**: 4-5 weeks (after Phase 4)

#### DID Contract Impact
- Linkage Contract queries DID Contract to verify gateway/device DIDs
- No changes to DID Contract needed
- DID Contract provides verification queries

---

### Phase 6: Advanced Key Management (Future)

**Timeline**: TBD  
**Priority**: Low (after Phase 5)

#### Advanced Features

1. **Multi-Key Support**
   - Multiple active keys per DID
   - Key purposes (signing, encryption, agreement)
   - Key priorities and fallbacks

2. **Threshold Signatures**
   - M-of-N multisig support
   - Threshold verification logic
   - Partial signature collection

3. **Key Expiration**
   - Time-based key validity
   - Automatic key rotation schedules
   - Grace periods for key overlap

4. **Advanced Cryptography**
   - Ed25519 support
   - BLS signatures
   - Post-quantum algorithms

5. **Key Recovery**
   - Social recovery mechanisms
   - Guardian-based key recovery
   - Time-locked recovery

---

## 🔄 Migration Strategy

### Phase 1 → Phase 2a Migration

**Existing DIDs**: Automatically extended with empty arrays
```rust
// Before (Phase 1)
DidDocument {
    id: "did:c4e:gateway:detrack1",
    controller: ["c4e1xyz..."],
    service: []
}

// After (Phase 2a) - automatic migration
DidDocument {
    id: "did:c4e:gateway:detrack1",
    controller: ["c4e1xyz..."],
    verification_method: [],  // 🆕 Empty initially
    authentication: [],        // 🆕 Empty initially
    assertion_method: [],      // 🆕 Empty initially
    service: []
}
```

**Action Required**: Gateways/Workers must add verification methods after upgrade
```bash
# Add first key after migration
c4ed tx wasm execute <did_contract> '{
  "add_verification_method": {
    "did": "did:c4e:gateway:detrack1",
    "verification_method": {
      "id": "did:c4e:gateway:detrack1#key-1",
      "vm_type": "EcdsaSecp256k1VerificationKey2019",
      "controller": "did:c4e:gateway:detrack1",
      "public_key_multibase": "<public_key_here>"
    }
  }
}' --from gateway
```

---

## 📊 Success Metrics

### Phase 2a Metrics
- [ ] 100% of gateways have at least one verification method
- [ ] Key rotation completes in <5 blocks
- [ ] Signature verification query: <50ms
- [ ] Zero downtime during key rotation
- [ ] 100% test coverage for key operations

### Phase 2b Metrics
- [ ] All proof submissions include key IDs
- [ ] 100% gateway signature verification success rate
- [ ] <100ms proof submission latency increase
- [ ] Zero invalid signatures accepted
- [ ] Audit trail complete for all proofs

---

## 🔐 Security Considerations

### Key Management Security
1. **Private Key Storage**: Never store private keys on-chain
2. **Public Key Validation**: Verify multibase encoding and format
3. **Authorization**: Only controllers can modify verification methods
4. **Atomic Operations**: Key rotation must be atomic (remove old, add new)
5. **Minimum Keys**: Always maintain at least one authentication key

### Signature Verification Security
1. **Replay Protection**: Include nonces or timestamps in signed messages
2. **Algorithm Whitelisting**: Only allow approved signature algorithms
3. **Key Purpose**: Verify key is authorized for specific purpose (auth vs assertion)
4. **Signature Malleability**: Use non-malleable signature schemes

---

## 📚 References

- [W3C DID Core 1.0 Specification](https://www.w3.org/TR/did-core/)
- [DID Method Rubric](https://www.w3.org/TR/did-rubric/)
- [Multibase Specification](https://github.com/multiformats/multibase)
- [CosmWasm Crypto Library](https://github.com/CosmWasm/cosmwasm/tree/main/packages/crypto)
- [DeTrack Network HLD](../../detrack-worker-node/docs/DeTrack-Network-HLD.md)

---

## 🤝 Contributing

For questions or contributions to DID Contract development:
- Open issues in [did-contract repository](https://github.com/chain4energy/did-contract)
- Discuss on Chain4Energy Discord #detrack-development
- Review ADRs before making architectural changes

---

**Document Version**: 1.0  
**Last Updated**: 2025-12-30  
**Next Review**: After Phase 2a completion
