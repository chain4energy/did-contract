# DID Contract Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying and operating the DID Contract on Chain4Energy networks. The contract implements decentralized identifier management compatible with W3C DID specifications.

## Prerequisites

### System Requirements
- **Rust**: 1.70+ with `wasm32-unknown-unknown` target
- **Docker**: For WASM optimization
- **c4ed CLI**: Chain4Energy command-line interface
- **Git**: For source code management

### Development Tools
```bash
# Install Rust and WASM target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Docker (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install docker.io
sudo usermod -aG docker $USER

# Install c4ed CLI
wget https://github.com/chain4energy/c4e-chain/releases/latest/download/c4ed-linux-amd64
chmod +x c4ed-linux-amd64
sudo mv c4ed-linux-amd64 /usr/local/bin/c4ed
```

## Network Configurations

### Mainnet
- **Chain ID**: `perun-1`
- **RPC Endpoint**: `https://rpc.c4e.io:443`
- **REST Endpoint**: `https://lcd.c4e.io:443`
- **Gas Prices**: `0.0025uc4e`

### Testnet (Babajaga)
- **Chain ID**: `babajaga-1`
- **RPC Endpoint**: `https://rpc.babajaga.c4e.io:443`
- **REST Endpoint**: `https://lcd.babajaga.c4e.io:443`
- **Gas Prices**: `0.0025uc4e`

### Local Development
- **Chain ID**: `c4e-dev`
- **RPC Endpoint**: `http://localhost:26657`
- **REST Endpoint**: `http://localhost:1317`
- **Gas Prices**: `0uc4e`

## Build Process

### 1. Clone Repository
```bash
git clone https://github.com/chain4energy/did-contract
cd did-contract
```

### 2. Development Build
```bash
# Build for development and testing
cargo build

# Run tests
cargo test

# Check linting
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### 3. Production Build
```bash
# Generate JSON schemas
cargo run --bin schema

# Build WASM binary
cargo wasm

# Optimize for deployment (requires Docker)
make optimize
```

### 4. Verify Build
```bash
# Check artifact size (should be < 2MB)
ls -lah artifacts/did_contract.wasm

# Verify checksums
cat artifacts/checksums.txt
```

## Deployment Steps

### Step 1: Configure Network

#### Mainnet Configuration
```bash
c4ed config chain-id perun-1
c4ed config node https://rpc.c4e.io:443
c4ed config broadcast-mode block
```

#### Testnet Configuration
```bash
c4ed config chain-id babajaga-1
c4ed config node https://rpc.babajaga.c4e.io:443
c4ed config broadcast-mode block
```

### Step 2: Prepare Wallet

#### Import Existing Wallet
```bash
c4ed keys add deployer --recover
# Enter your mnemonic phrase when prompted
```

#### Create New Wallet
```bash
c4ed keys add deployer
# Save the mnemonic phrase securely
```

#### Check Balance
```bash
c4ed query bank balances $(c4ed keys show deployer -a)
```

### Step 3: Store Contract Code

```bash
# Store the optimized WASM file
c4ed tx wasm store artifacts/did_contract.wasm \
  --from deployer \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices 0.0025uc4e \
  --broadcast-mode block

# Note the CODE_ID from the response
export CODE_ID=<code_id_from_response>
```

#### Verify Code Storage
```bash
c4ed query wasm code $CODE_ID
c4ed query wasm list-code
```

### Step 4: Instantiate Contract

```bash
# Instantiate with empty parameters
c4ed tx wasm instantiate $CODE_ID '{}' \
  --from deployer \
  --label "did-contract-v1.0.0" \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices 0.0025uc4e \
  --broadcast-mode block \
  --no-admin

# Save contract address from response
export CONTRACT_ADDRESS=<contract_address_from_response>
```

### Step 5: Verify Deployment

#### Check Contract Info
```bash
c4ed query wasm contract $CONTRACT_ADDRESS
c4ed query wasm contract-state all $CONTRACT_ADDRESS
```

#### Test Basic Functionality
```bash
# Test with a simple DID creation
c4ed tx wasm execute $CONTRACT_ADDRESS '{
  "create_did_document": {
    "did_doc": {
      "id": "did:c4e:test",
      "controller": ["'$(c4ed keys show deployer -a)'"],
      "service": []
    }
  }
}' \
  --from deployer \
  --gas auto \
  --gas-adjustment 1.3 \
  --gas-prices 0.0025uc4e

# Query the created DID
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "get_did_document": {
    "did": "did:c4e:test"
  }
}'
```

## Environment-Specific Deployments

### Local Development Network

#### 1. Start Local Chain
```bash
# Clone and start C4E chain
git clone https://github.com/chain4energy/c4e-chain
cd c4e-chain
make install
c4ed init test --chain-id c4e-dev
c4ed keys add alice
c4ed add-genesis-account alice 1000000000uc4e
c4ed gentx alice 1000000uc4e --chain-id c4e-dev
c4ed collect-gentxs
c4ed start
```

#### 2. Deploy Contract
```bash
# Store and instantiate on local chain
c4ed tx wasm store artifacts/did_contract.wasm \
  --from alice \
  --gas auto \
  --chain-id c4e-dev \
  --keyring-backend test

c4ed tx wasm instantiate $CODE_ID '{}' \
  --from alice \
  --label "did-contract-local" \
  --chain-id c4e-dev \
  --keyring-backend test \
  --no-admin
```

### Testnet (Babajaga) Deployment

#### 1. Get Testnet Tokens
```bash
# Request tokens from faucet
curl -X POST "https://faucet.babajaga.c4e.io" \
  -H "Content-Type: application/json" \
  -d '{"address": "'$(c4ed keys show deployer -a)'"}'
```

#### 2. Deploy with Testnet Configuration
```bash
c4ed config chain-id babajaga-1
c4ed config node https://rpc.babajaga.c4e.io:443

# Store contract
c4ed tx wasm store artifacts/did_contract.wasm \
  --from deployer \
  --gas 2000000 \
  --gas-prices 0.0025uc4e

# Instantiate
c4ed tx wasm instantiate $CODE_ID '{}' \
  --from deployer \
  --label "did-contract-testnet-$(date +%s)" \
  --gas 1000000 \
  --gas-prices 0.0025uc4e \
  --no-admin
```

### Mainnet Production Deployment

#### 1. Security Checklist
- [ ] Code audited by security professionals
- [ ] All tests passing with 100% success rate
- [ ] Gas usage optimized and tested
- [ ] Deployment wallet secured with hardware device
- [ ] Contract admin permissions reviewed
- [ ] Emergency procedures documented

#### 2. Deployment Steps
```bash
c4ed config chain-id perun-1
c4ed config node https://rpc.c4e.io:443

# Store contract (higher gas limit for mainnet)
c4ed tx wasm store artifacts/did_contract.wasm \
  --from deployer \
  --gas 3000000 \
  --gas-prices 0.005uc4e \
  --broadcast-mode block

# Instantiate with production label
c4ed tx wasm instantiate $CODE_ID '{}' \
  --from deployer \
  --label "did-contract-mainnet-v1.0.0" \
  --gas 1500000 \
  --gas-prices 0.005uc4e \
  --broadcast-mode block \
  --no-admin
```

## Post-Deployment Configuration

### 1. Document Contract Details

Create a deployment record:
```json
{
  "deployment_info": {
    "network": "babajaga-1",
    "code_id": 123,
    "contract_address": "c4e1...",
    "deployer": "c4e1...",
    "deployment_time": "2025-11-10T10:00:00Z",
    "contract_version": "1.0.0",
    "git_commit": "abc123...",
    "optimization_version": "0.14.0"
  }
}
```

### 2. Set Up Monitoring

#### Contract Events Monitoring
```bash
# Monitor contract events
c4ed query tx --type=hash <deployment_tx_hash>

# Set up event streaming
c4ed subscribe-events "wasm._contract_address='$CONTRACT_ADDRESS'"
```

#### Health Check Script
```bash
#!/bin/bash
# health_check.sh

CONTRACT_ADDRESS="c4e1..."

# Test basic query
RESPONSE=$(c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "do_controllers_exist": {
    "controllers": ["c4e1test"]
  }
}' --output json 2>/dev/null)

if [ $? -eq 0 ]; then
  echo "✅ Contract is responding"
else
  echo "❌ Contract health check failed"
  exit 1
fi
```

### 3. Integration Testing

#### End-to-End Test Suite
```bash
#!/bin/bash
# e2e_test.sh

set -e

CONTRACT_ADDRESS="c4e1..."
TEST_ACCOUNT="c4e1..."

echo "Running end-to-end tests..."

# Test 1: Create DID
echo "Test 1: Creating DID document..."
c4ed tx wasm execute $CONTRACT_ADDRESS '{
  "create_did_document": {
    "did_doc": {
      "id": "did:c4e:e2e-test-'$(date +%s)'",
      "controller": ["'$TEST_ACCOUNT'"],
      "service": []
    }
  }
}' --from deployer --gas auto --yes

# Test 2: Query DID
echo "Test 2: Querying DID document..."
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "get_did_document": {
    "did": "did:c4e:e2e-test-'$(date +%s)'"
  }
}'

# Test 3: Controller check
echo "Test 3: Checking controller authorization..."
c4ed query wasm contract-state smart $CONTRACT_ADDRESS '{
  "is_did_controller": {
    "did": "did:c4e:e2e-test-'$(date +%s)'",
    "controller": "'$TEST_ACCOUNT'"
  }
}'

echo "✅ All tests passed!"
```

## Upgrade Procedures

### Contract Migration (if admin enabled)

```bash
# Store new version
NEW_CODE_ID=$(c4ed tx wasm store artifacts/did_contract_v2.wasm \
  --from admin \
  --gas auto \
  --output json | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

# Migrate contract
c4ed tx wasm migrate $CONTRACT_ADDRESS $NEW_CODE_ID '{}' \
  --from admin \
  --gas auto
```

### New Deployment (if no admin)

```bash
# Deploy new version
c4ed tx wasm instantiate $NEW_CODE_ID '{}' \
  --from deployer \
  --label "did-contract-v2.0.0" \
  --gas auto \
  --no-admin

# Update frontend configurations
# Notify users of new contract address
# Maintain compatibility period
```

## Security Best Practices

### 1. Key Management
- Use hardware wallets for mainnet deployments
- Implement multi-signature for admin operations
- Rotate keys regularly
- Store mnemonic phrases securely offline

### 2. Access Control
- Deploy with `--no-admin` for immutable contracts
- Use least-privilege principle for admin functions
- Document all admin operations
- Implement time-locks for critical changes

### 3. Monitoring
- Set up automated health checks
- Monitor contract events and transactions
- Track gas usage patterns
- Alert on unusual activity

### 4. Backup and Recovery
- Backup all deployment artifacts
- Document recovery procedures
- Test backup restoration
- Maintain deployment history

## Troubleshooting

### Common Issues

#### Insufficient Gas
```bash
# Error: out of gas in location
# Solution: Increase gas limit
c4ed tx wasm store artifacts/did_contract.wasm \
  --from deployer \
  --gas 3000000  # Increased from default
```

#### Invalid Address Format
```bash
# Error: invalid address format
# Solution: Verify address encoding
c4ed debug addr $(c4ed keys show deployer -a)
```

#### Code ID Not Found
```bash
# Error: code id not found
# Solution: Verify code was stored successfully
c4ed query wasm list-code
c4ed query wasm code $CODE_ID
```

#### Contract Already Exists
```bash
# Error: contract already exists
# Solution: Use different label or check existing contracts
c4ed query wasm list-contract-by-code $CODE_ID
```

### Debugging Commands

#### Contract State
```bash
# Check all contract state
c4ed query wasm contract-state all $CONTRACT_ADDRESS

# Check specific state
c4ed query wasm contract-state raw $CONTRACT_ADDRESS <hex_key>
```

#### Transaction Analysis
```bash
# Get transaction details
c4ed query tx <tx_hash>

# Check events
c4ed query tx <tx_hash> --output json | jq '.logs[].events[]'
```

#### Gas Analysis
```bash
# Simulate transaction
c4ed tx wasm execute $CONTRACT_ADDRESS '{}' \
  --from deployer \
  --dry-run

# Get gas usage
c4ed tx wasm execute $CONTRACT_ADDRESS '{}' \
  --from deployer \
  --gas-adjustment 1.0 \
  --simulate
```

## Appendix

### A. Network Endpoints

| Network | Chain ID | RPC | REST | Faucet |
|---------|----------|-----|------|--------|
| Mainnet | perun-1 | https://rpc.c4e.io:443 | https://lcd.c4e.io:443 | N/A |
| Testnet | babajaga-1 | https://rpc.babajaga.c4e.io:443 | https://lcd.babajaga.c4e.io:443 | https://faucet.babajaga.c4e.io |
| Local | c4e-dev | http://localhost:26657 | http://localhost:1317 | Local |

### B. Gas Estimation Guide

| Operation | Estimated Gas | Notes |
|-----------|---------------|-------|
| Store Code | 2,000,000 - 3,000,000 | Depends on contract size |
| Instantiate | 500,000 - 1,000,000 | Simple instantiation |
| Create DID | 200,000 - 400,000 | Depends on document size |
| Update DID | 150,000 - 300,000 | Depends on changes |
| Query DID | 50,000 - 100,000 | Read-only operations |

### C. Useful Scripts

#### Deployment Script
```bash
#!/bin/bash
# deploy.sh
set -e

NETWORK=${1:-testnet}
DEPLOYER=${2:-deployer}

source scripts/networks/$NETWORK.env

echo "Deploying to $NETWORK..."
echo "Deployer: $(c4ed keys show $DEPLOYER -a)"
echo "Chain ID: $CHAIN_ID"

# Store code
CODE_ID=$(c4ed tx wasm store artifacts/did_contract.wasm \
  --from $DEPLOYER \
  --gas $STORE_GAS \
  --gas-prices $GAS_PRICES \
  --output json \
  --yes | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

echo "Code stored with ID: $CODE_ID"

# Instantiate
CONTRACT_ADDRESS=$(c4ed tx wasm instantiate $CODE_ID '{}' \
  --from $DEPLOYER \
  --label "did-contract-$NETWORK-$(date +%s)" \
  --gas $INSTANTIATE_GAS \
  --gas-prices $GAS_PRICES \
  --no-admin \
  --output json \
  --yes | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')

echo "Contract instantiated at: $CONTRACT_ADDRESS"

# Save deployment info
cat > deployments/$NETWORK.json << EOF
{
  "network": "$NETWORK",
  "chain_id": "$CHAIN_ID", 
  "code_id": "$CODE_ID",
  "contract_address": "$CONTRACT_ADDRESS",
  "deployer": "$(c4ed keys show $DEPLOYER -a)",
  "deployment_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "Deployment complete! Details saved to deployments/$NETWORK.json"
```