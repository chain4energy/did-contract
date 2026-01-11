#!/bin/bash

# DID Contract Deployment Script for Local C4E Blockchain
# Usage: ./scripts/deploy-local.sh

set -e  # Exit on error

# Load configuration from detrack-worker-node .env
ENV_FILE="../detrack-worker-node/config/.env"
if [ -f "$ENV_FILE" ]; then
    source "$ENV_FILE"
else
    echo "❌ Environment file not found: $ENV_FILE"
    exit 1
fi

# Configuration
CHAIN_ID="${C4E_CHAIN_ID}"
NODE="${C4E_RPC_ENDPOINT}"
KEYRING_BACKEND="${KEYRING_BACKEND}"
HOME_DIR="${HOME_DIR}"
GAS_PRICES="${C4E_GAS_PRICE}"

# Deployer (Alice - admin)
DEPLOYER_NAME="${ALICE_NAME}"
DEPLOYER_ADDR="${ALICE}"

# Contract file
CONTRACT_WASM="./artifacts/did_contract.wasm"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 DID Contract Deployment - Local C4E Chain"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Configuration:"
echo "   Chain ID: $CHAIN_ID"
echo "   Node: $NODE"
echo "   Deployer: $DEPLOYER_NAME ($DEPLOYER_ADDR)"
echo "   Contract: $CONTRACT_WASM"
echo ""

# Check if contract file exists
if [ ! -f "$CONTRACT_WASM" ]; then
    echo "❌ Contract file not found: $CONTRACT_WASM"
    echo "   Run 'make optimize' first to build the contract"
    exit 1
fi

# Check contract size
CONTRACT_SIZE=$(stat -c%s "$CONTRACT_WASM")
echo "📦 Contract size: $(numfmt --to=iec-i --suffix=B $CONTRACT_SIZE)"
if [ $CONTRACT_SIZE -gt 2097152 ]; then
    echo "⚠️  Warning: Contract size exceeds 2MB, may fail on-chain"
fi
echo ""

# Check deployer balance
echo "💰 Checking deployer balance..."
BALANCE=$(c4ed query bank balances $DEPLOYER_ADDR \
    --node "$NODE" \
    --chain-id "$CHAIN_ID" \
    --output json 2>/dev/null | jq -r '.balances[] | select(.denom=="uc4e") | .amount // "0"')

if [ "$BALANCE" = "0" ] || [ -z "$BALANCE" ]; then
    echo "❌ Deployer has insufficient balance"
    exit 1
fi

echo "   Balance: $BALANCE uc4e"
echo ""

# Step 1: Store contract code
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📤 Step 1: Storing contract code on-chain..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

STORE_TX=$(c4ed tx wasm store "$CONTRACT_WASM" \
    --from "$DEPLOYER_NAME" \
    --node "$NODE" \
    --chain-id "$CHAIN_ID" \
    --gas-prices "$GAS_PRICES" \
    --gas auto \
    --gas-adjustment 1.3 \
    --keyring-backend "$KEYRING_BACKEND" \
    --home "$HOME_DIR" \
    --yes \
    --output json)

# Extract transaction hash
TX_HASH=$(echo "$STORE_TX" | jq -r '.txhash')
echo "   TX Hash: $TX_HASH"

# Wait for transaction confirmation
echo "   Waiting for confirmation..."
sleep 6

# Get code ID from transaction
CODE_ID=$(c4ed query tx "$TX_HASH" \
    --node "$NODE" \
    --chain-id "$CHAIN_ID" \
    --output json | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')

if [ -z "$CODE_ID" ] || [ "$CODE_ID" = "null" ]; then
    echo "❌ Failed to get Code ID from transaction"
    exit 1
fi

echo "✅ Contract code stored successfully"
echo "   Code ID: $CODE_ID"
echo ""

# Step 2: Instantiate contract
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 Step 2: Instantiating DID contract..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Instantiate message (empty for DID contract - no required parameters)
INIT_MSG='{}'

INSTANTIATE_TX=$(c4ed tx wasm instantiate "$CODE_ID" "$INIT_MSG" \
    --from "$DEPLOYER_NAME" \
    --node "$NODE" \
    --chain-id "$CHAIN_ID" \
    --gas-prices "$GAS_PRICES" \
    --gas auto \
    --gas-adjustment 1.3 \
    --keyring-backend "$KEYRING_BACKEND" \
    --home "$HOME_DIR" \
    --label "DID Contract v1.0" \
    --admin "$DEPLOYER_ADDR" \
    --yes \
    --output json)

# Extract transaction hash
INST_TX_HASH=$(echo "$INSTANTIATE_TX" | jq -r '.txhash')
echo "   TX Hash: $INST_TX_HASH"

# Wait for transaction confirmation
echo "   Waiting for confirmation..."
sleep 6

# Get contract address from transaction
CONTRACT_ADDRESS=$(c4ed query tx "$INST_TX_HASH" \
    --node "$NODE" \
    --chain-id "$CHAIN_ID" \
    --output json | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')

if [ -z "$CONTRACT_ADDRESS" ] || [ "$CONTRACT_ADDRESS" = "null" ]; then
    echo "❌ Failed to get contract address from transaction"
    exit 1
fi

echo "✅ DID contract instantiated successfully"
echo "   Contract Address: $CONTRACT_ADDRESS"
echo ""

# Step 3: Verify deployment
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Step 3: Verifying deployment..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Query contract info
CONTRACT_INFO=$(c4ed query wasm contract "$CONTRACT_ADDRESS" \
    --node "$NODE" \
    --chain-id "$CHAIN_ID" \
    --output json)

echo "   Label: $(echo "$CONTRACT_INFO" | jq -r '.contract_info.label')"
echo "   Admin: $(echo "$CONTRACT_INFO" | jq -r '.contract_info.admin')"
echo "   Code ID: $(echo "$CONTRACT_INFO" | jq -r '.contract_info.code_id')"
echo ""

# Test query (get stats)
echo "   Testing contract query (stats)..."
STATS=$(c4ed query wasm contract-state smart "$CONTRACT_ADDRESS" '{"stats":{}}' \
    --node "$NODE" \
    --chain-id "$CHAIN_ID" \
    --output json 2>/dev/null)

if [ $? -eq 0 ]; then
    echo "   ✅ Query successful: $(echo "$STATS" | jq -c '.data')"
else
    echo "   ⚠️  Query failed - contract may not be fully initialized"
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ DEPLOYMENT COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 Deployment Summary:"
echo "   Code ID: $CODE_ID"
echo "   Contract Address: $CONTRACT_ADDRESS"
echo ""
echo "📋 Next Steps:"
echo "   1. Update .env file with new DID_CONTRACT_ADDRESS"
echo "   2. Test DID registration: ./scripts/test-did-registration.sh"
echo "   3. Update Worker Node configuration"
echo ""
echo "   To update .env automatically, run:"
echo "   sed -i 's/DID_CONTRACT_ADDRESS=.*/DID_CONTRACT_ADDRESS=$CONTRACT_ADDRESS/' ../detrack-worker-node/config/.env"
echo ""
