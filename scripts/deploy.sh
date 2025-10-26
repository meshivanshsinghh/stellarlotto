#!/bin/bash
# StellarLotto Complete Deployment Script
# Deploys MockUSDC + 3 Lottery Pools (Small, Medium, Whale)

set -e
NETWORK="testnet"
ADMIN_KEY="admin"
PLAYER1_KEY="player1"
PLAYER2_KEY="player2"
PLAYER3_KEY="player3"
WHALE_KEY="whale"

# Pool minimums (in stroops: 1 USDC = 10000000)
SMALL_MIN=100000000      # 10 USDC
MEDIUM_MIN=1000000000    # 100 USDC  
WHALE_MIN=5000000000     # 500 USDC

# Contract settings
YIELD_RATE=1000          # 10% APY (basis points)
ROUND_DURATION=120       # 2 minutes

# Mint amounts for testing (in stroops)
PLAYER1_MINT=500000000   # 50 USDC
PLAYER2_MINT=2000000000  # 200 USDC
PLAYER3_MINT=1500000000  # 150 USDC
WHALE_MINT=20000000000   # 2000 USDC


# Initial yield reserves for pools
SMALL_RESERVE=100000000   # 10 USDC
MEDIUM_RESERVE=500000000  # 50 USDC
WHALE_RESERVE=1000000000  # 100 USDC

export STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export STELLAR_RPC_URL="https://soroban-testnet.stellar.org"


echo "Building MockUSDC..."
cd ..
cd contracts/mock_usdc
stellar contract build
cd ../..

echo "Building LotteryPool..."
cd contracts/lottery_pool
stellar contract build
cd ../..


echo "Deploying MockUSDC..."
MOCK_USDC_ID=$(stellar contract deploy \
  --wasm contracts/mock_usdc/target/wasm32v1-none/release/mock_usdc.wasm \
  --source $ADMIN_KEY \
  --network $NETWORK)

echo "MockUSDC deployed: $MOCK_USDC_ID"


echo "Initializing MockUSDC..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- initialize \
  --admin $(stellar keys address $ADMIN_KEY) > /dev/null

echo "MockUSDC initialized"

echo "Deploying Small Pool (10+ USDC)..."
SMALL_POOL_ID=$(stellar contract deploy \
  --wasm contracts/lottery_pool/target/wasm32v1-none/release/lottery_pool.wasm \
  --source $ADMIN_KEY \
  --network $NETWORK)
echo "Small Pool: $SMALL_POOL_ID"

echo "Deploying Medium Pool (100+ USDC)..."
MEDIUM_POOL_ID=$(stellar contract deploy \
  --wasm contracts/lottery_pool/target/wasm32v1-none/release/lottery_pool.wasm \
  --source $ADMIN_KEY \
  --network $NETWORK)
echo "Medium Pool: $MEDIUM_POOL_ID"

echo "Deploying Whale Pool (500+ USDC)..."
WHALE_POOL_ID=$(stellar contract deploy \
  --wasm contracts/lottery_pool/target/wasm32v1-none/release/lottery_pool.wasm \
  --source $ADMIN_KEY \
  --network $NETWORK)
echo "Whale Pool: $WHALE_POOL_ID"
echo ""

echo "Initializing Pools..."
echo "Initializing Small Pool..."
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- initialize \
  --admin $(stellar keys address $ADMIN_KEY) \
  --usdc_token $MOCK_USDC_ID \
  --yield_rate $YIELD_RATE \
  --round_duration $ROUND_DURATION \
  --min_deposit $SMALL_MIN > /dev/null
echo "Small Pool initialized"


echo "Initializing Medium Pool..."
stellar contract invoke \
  --id $MEDIUM_POOL_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- initialize \
  --admin $(stellar keys address $ADMIN_KEY) \
  --usdc_token $MOCK_USDC_ID \
  --yield_rate $YIELD_RATE \
  --round_duration $ROUND_DURATION \
  --min_deposit $MEDIUM_MIN > /dev/null
echo "Medium Pool initialized"

echo "Initializing Whale Pool..."
stellar contract invoke \
  --id $WHALE_POOL_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- initialize \
  --admin $(stellar keys address $ADMIN_KEY) \
  --usdc_token $MOCK_USDC_ID \
  --yield_rate $YIELD_RATE \
  --round_duration $ROUND_DURATION \
  --min_deposit $WHALE_MIN > /dev/null
echo "Whale Pool initialized"
echo ""



echo "Minting test USDC to players..."
echo "Minting 50 USDC to player1..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $PLAYER1_KEY) \
  --amount $PLAYER1_MINT > /dev/null

echo "Minting 200 USDC to player2..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $PLAYER2_KEY) \
  --amount $PLAYER2_MINT > /dev/null

echo "Minting 150 USDC to player3..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $PLAYER3_KEY) \
  --amount $PLAYER3_MINT > /dev/null

echo "Minting 2000 USDC to whale..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $WHALE_KEY) \
  --amount $WHALE_MINT > /dev/null

echo "All players funded"



echo "Depositing initial yield reserves to pools..."
TOTAL_RESERVE=$((SMALL_RESERVE + MEDIUM_RESERVE + WHALE_RESERVE))
echo "Minting $TOTAL_RESERVE stroops to admin for reserves..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $ADMIN_KEY) \
  --amount $TOTAL_RESERVE > /dev/null

echo "Transferring 10 USDC to Small Pool..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- transfer \
  --from $(stellar keys address $ADMIN_KEY) \
  --to $SMALL_POOL_ID \
  --amount $SMALL_RESERVE > /dev/null

echo "Transferring 50 USDC to Medium Pool..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- transfer \
  --from $(stellar keys address $ADMIN_KEY) \
  --to $MEDIUM_POOL_ID \
  --amount $MEDIUM_RESERVE > /dev/null

echo "Transferring 100 USDC to Whale Pool..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- transfer \
  --from $(stellar keys address $ADMIN_KEY) \
  --to $WHALE_POOL_ID \
  --amount $WHALE_RESERVE > /dev/null


echo "All pools funded with yield reserves"
echo ""

echo "Generating .env file..."
echo ""

cat > .env << EOF
# StellarLotto Environment Variables
# Generated: $(date)

# Network Configuration
NETWORK=$NETWORK
HORIZON_URL=https://horizon-testnet.stellar.org
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org

# Admin and Player Addresses
ADMIN_ADDRESS=$(stellar keys address $ADMIN_KEY)
PLAYER1_ADDRESS=$(stellar keys address $PLAYER1_KEY)
PLAYER2_ADDRESS=$(stellar keys address $PLAYER2_KEY)
PLAYER3_ADDRESS=$(stellar keys address $PLAYER3_KEY)
WHALE_ADDRESS=$(stellar keys address $WHALE_KEY)

# Contract IDs
MOCK_USDC_ID=$MOCK_USDC_ID
SMALL_POOL_ID=$SMALL_POOL_ID
MEDIUM_POOL_ID=$MEDIUM_POOL_ID
WHALE_POOL_ID=$WHALE_POOL_ID

# Pool Configurations (in stroops: 1 USDC = 10000000)
SMALL_POOL_MIN=$SMALL_MIN
MEDIUM_POOL_MIN=$MEDIUM_MIN
WHALE_POOL_MIN=$WHALE_MIN

# Contract Settings
YIELD_RATE=$YIELD_RATE
ROUND_DURATION=$ROUND_DURATION

# Derived Values
STROOPS_PER_USDC=10000000
EOF

echo ".env file created!"
echo ""