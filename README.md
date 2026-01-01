# DID Contract

A CosmWasm smart contract implementing W3C Decentralized Identifier (DID) specification for decentralized identity management on the Chain4Energy (C4E) blockchain.

## Overview

The DID Contract provides a complete decentralized identity management system that enables creation, management, and verification of digital identities without relying on centralized authorities. It implements core W3C DID specification features with optimizations for the Cosmos ecosystem.

## Features

- **🆔 DID Document Management**: Create, update, and delete W3C-compliant DID documents
- **👥 Multi-Controller Support**: Hierarchical control with address and DID-based controllers
- **🔗 Service Registry**: Manage service endpoints for identity interactions
- **🔍 Advanced Queries**: Efficient lookups by DID, controller, or relationship
- **🛡️ Recursive Authorization**: Secure permission resolution through controller chains
- **📊 Optimized Storage**: Dual-index storage pattern for high-performance queries
- **🔄 Real-time Events**: Comprehensive event system for external monitoring

## Quick Start

### Prerequisites

- Rust 1.70+
- `wasm32-unknown-unknown` target
- Docker (for optimization)

### Installation

```bash
# Clone the repository
git clone https://github.com/chain4energy/did-contract
cd did-contract

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build

```bash
# Development build
cargo wasm

# Production build (optimized)
make optimize
```

### Test

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test create_did_document

# With output
cargo test -- --nocapture
```

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[Contract Design](docs/contract-design.md)**: Architecture, design decisions, and technical details
- **[API Specification](docs/api-specification.md)**: Complete API reference with examples
- **[Deployment Guide](docs/deployment-guide.md)**: Step-by-step deployment instructions

## Usage Examples

### Create a DID Document

```bash
# Create personal identity
c4ed tx wasm execute <contract_address> '{
  "create_did_document": {
    "did_doc": {
      "id": "did:c4e:alice",
      "controller": ["c4e1alice...address"],
      "service": [
        {
          "id": "did:c4e:alice#messaging",
          "a_type": "MessagingService",
          "service_endpoint": "https://alice.example.com/messaging"
        }
      ]
    }
  }
}' --from alice
```

### Query a DID Document

```bash
c4ed query wasm contract-state smart <contract_address> '{
  "get_did_document": {
    "did": "did:c4e:alice"
  }
}'
```

### Check Controller Authorization

```bash
c4ed query wasm contract-state smart <contract_address> '{
  "is_did_controller": {
    "did": "did:c4e:alice",
    "controller": "c4e1alice...address"
  }
}'
```

### List Controlled DIDs

```bash
c4ed query wasm contract-state smart <contract_address> '{
  "get_controlled_dids": {
    "controller": "c4e1alice...address",
    "limit": 50
  }
}'
```

## Architecture

### Storage Structure

The contract uses a dual-index storage pattern for optimal performance:

- **Primary Storage**: Direct DID document lookup by identifier
- **Controller Index**: Efficient reverse lookups for controlled DIDs
- **Event System**: Real-time notifications for all state changes

### Authorization Model

Multi-level authorization system:
1. **Address Controllers**: Direct control by Cosmos addresses
2. **DID Controllers**: Hierarchical control through other DIDs
3. **Recursive Resolution**: Automatic authorization chain traversal

## Project Structure

```
did-contract/
├── docs/                    # Documentation
│   ├── contract-design.md   # Architecture details
│   ├── api-specification.md # Complete API reference
│   └── deployment-guide.md  # Deployment instructions
├── src/
│   ├── contract.rs         # Main contract implementation (Sylvia)
│   ├── state.rs           # State structures and validation
│   ├── error.rs           # Error definitions
│   ├── multiset.rs        # Controller indexing utilities
│   ├── lib.rs             # Library exports
│   └── test/              # Comprehensive test suite
├── schema/                # Generated JSON schemas
├── Cargo.toml            # Dependencies and metadata
├── Makefile              # Build and deployment tasks
└── README.md             # This file
```

## Technology Stack

- **Framework**: Sylvia 1.3.5 (CosmWasm framework)
- **CosmWasm**: 2.2.2
- **Storage**: cw-storage-plus 2.0.0
- **Schema Generation**: cosmwasm-schema 2.1.3
- **Testing**: cw-multi-test 2.1.1

## Development

### Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test create_did_document

# Integration tests
cargo test --features library

# With coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt

# Security audit
cargo audit
```

### Schema Generation

```bash
# Generate JSON schemas for messages
cargo run --bin schema

# Schemas will be created in schema/ directory
ls schema/
```

## Deployment

### Quick Deployment (Testnet)

```bash
# 1. Build optimized WASM
make optimize

# 2. Store code
c4ed tx wasm store artifacts/did_contract.wasm --from admin

# 3. Instantiate
c4ed tx wasm instantiate <code_id> '{}' \
  --from admin \
  --label "did-contract"
```

See [Deployment Guide](docs/deployment-guide.md) for detailed instructions.

## Use Cases

### 1. Personal Identity Management
- Individual identity creation and management
- Service endpoint registration
- Authorization delegation

### 2. Organizational Identity
- Corporate identity structures
- Employee credential management
- Hierarchical access control

### 3. Device Identity (IoT)
- Device registration and authentication
- Sensor data attribution
- Device lifecycle management

### 4. Cross-Chain Identity
- Multi-chain identity resolution
- Cross-chain authentication
- Interoperability protocols

## API Overview

### Execute Messages
- `create_did_document`: Create new DID document
- `update_did_document`: Update existing document
- `delete_did_document`: Remove DID document
- `add_controller`: Add new controller
- `delete_controller`: Remove controller
- `add_service`: Add service endpoint
- `delete_service`: Remove service endpoint

### Query Messages
- `get_did_document`: Retrieve DID document
- `is_did_controller`: Check controller authorization
- `get_controlled_dids`: List controlled DIDs
- `get_controlled_did_documents`: Get full documents
- `do_controllers_exist`: Batch controller existence check

See [API Specification](docs/api-specification.md) for complete reference.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## Security

### Reporting Vulnerabilities

Please report security vulnerabilities to: [security contact]

### Security Features

- ✅ Comprehensive input validation
- ✅ Recursive authorization with loop detection
- ✅ Address format validation
- ✅ DID format validation
- ✅ Duplicate prevention
- ✅ Atomic state updates
- ✅ Event-driven monitoring
- ✅ Gas optimization

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

Copyright 2025 Chain4Energy

## Contact

- **Project**: Chain4Energy DID Contract
- **Repository**: https://github.com/chain4energy/did-contract
- **Documentation**: [Link to online docs]

## Acknowledgments

Built with:
- [CosmWasm](https://cosmwasm.com/)
- [Sylvia Framework](https://github.com/CosmWasm/sylvia)
- [Chain4Energy](https://c4e.io/)
- [W3C DID Specification](https://www.w3.org/TR/did-core/)