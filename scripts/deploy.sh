#!/bin/bash
# StellarLotto Deployment with Blend Integration
# Deploys 3 Lottery Pools connected to Blend testnet

set -e
NETWORK="testnet"
ADMIN_KEY="admin"
PLAYER1_KEY="player1"
PLAYER2_KEY="player2"
PLAYER3_KEY="player3"
WHALE_KEY="whale"

# ============================================
# BLEND TESTNET ADDRESSES (from blend-utils/testnet.contracts.json)
# UPDATE THESE after running blend-utils mock script!
# ============================================
BLEND_USDC_ID="CCKBPIQHCIFGYR27Q3DAFPFCOI44JX2W6J5K7WFVLPLJB6SBJFUOYHYC"
BLEND_POOL_ID="CBQAII2SRK6EWTUTB5CKNBSN5JAR3MIBIOM2TCJK2PFKZ6KQQTKXCXIV"

# Pool minimums (in stroops: 1 USDC = 10000000)
SMALL_MIN=100000000      # 10 USDC
MEDIUM_MIN=1000000000    # 100 USDC  
WHALE_MIN=5000000000     # 500 USDC

# Mint amounts for players
PLAYER1_MINT=5000000000   # 500 USDC
PLAYER2_MINT=15000000000  # 1500 USDC
PLAYER3_MINT=10000000000  # 1000 USDC

# House money for each pool (for demo yield boosts)
SMALL_HOUSE=500000000     # 50 USDC
MEDIUM_HOUSE=2000000000   # 200 USDC
WHALE_HOUSE=5000000000    # 500 USDC

# Contract settings
YIELD_RATE=1000          # 10% APY (for reference, actual yield from Blend)
ROUND_DURATION=120       # 2 minutes

export STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
export STELLAR_RPC_URL="https://soroban-testnet.stellar.org"

echo "=========================================="
echo "StellarLotto - Blend Integration Deployment"
echo "=========================================="
echo ""
echo "Using Blend Testnet:"
echo "  USDC: $BLEND_USDC_ID"
echo "  Pool: $BLEND_POOL_ID"
echo ""

# ============================================
# PHASE 1: MINT USDC TO PLAYERS
# ============================================
echo "Minting USDC to test players..."
echo ""

echo "Minting 500 USDC to player1..."
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $PLAYER1_KEY) \
  --amount $PLAYER1_MINT

echo "Minting 1500 USDC to player2..."
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $PLAYER2_KEY) \
  --amount $PLAYER2_MINT

echo "Minting 1000 USDC to player3..."
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- mint \
  --to $(stellar keys address $PLAYER3_KEY) \
  --amount $PLAYER3_MINT

echo "✓ All players funded with USDC"
echo ""

# ============================================
# PHASE 2: VERIFY BALANCES
# ============================================
echo "Verifying player balances..."
echo ""

echo "Player1 balance:"
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $PLAYER1_KEY \
  --network $NETWORK \
  -- balance \
  --id $(stellar keys address $PLAYER1_KEY)

echo ""
echo "Player2 balance:"
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $PLAYER2_KEY \
  --network $NETWORK \
  -- balance \
  --id $(stellar keys address $PLAYER2_KEY)

echo ""
echo "Player3 balance:"
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $PLAYER3_KEY \
  --network $NETWORK \
  -- balance \
  --id $(stellar keys address $PLAYER3_KEY)

echo ""
echo "✓ Balances verified"
echo ""

# ============================================
# PHASE 3: BUILD LOTTERY POOL CONTRACT
# ============================================
echo "Building LotteryPool contract..."
cd ..
cd contracts/lottery_pool
stellar contract build
cd ../..
echo "✓ LotteryPool built"
echo ""

# ============================================
# PHASE 4: DEPLOY LOTTERY POOLS
# ============================================
echo "Deploying Small Pool (10+ USDC)..."
SMALL_POOL_ID=$(stellar contract deploy \
  --wasm contracts/lottery_pool/target/wasm32v1-none/release/lottery_pool.wasm \
  --source $ADMIN_KEY \
  --network $NETWORK)
echo "✓ Small Pool: $SMALL_POOL_ID"

echo "Deploying Medium Pool (100+ USDC)..."
MEDIUM_POOL_ID=$(stellar contract deploy \
  --wasm contracts/lottery_pool/target/wasm32v1-none/release/lottery_pool.wasm \
  --source $ADMIN_KEY \
  --network $NETWORK)
echo "✓ Medium Pool: $MEDIUM_POOL_ID"

echo "Deploying Whale Pool (500+ USDC)..."
WHALE_POOL_ID=$(stellar contract deploy \
  --wasm contracts/lottery_pool/target/wasm32v1-none/release/lottery_pool.wasm \
  --source $ADMIN_KEY \
  --network $NETWORK)
echo "✓ Whale Pool: $WHALE_POOL_ID"
echo ""

# ============================================
# PHASE 5: INITIALIZE POOLS WITH BLEND
# ============================================
echo "Initializing pools with Blend integration..."
echo ""

echo "Initializing Small Pool..."
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- initialize \
  --admin $(stellar keys address $ADMIN_KEY) \
  --usdc_token $BLEND_USDC_ID \
  --blend_pool $BLEND_POOL_ID \
  --yield_rate $YIELD_RATE \
  --round_duration $ROUND_DURATION \
  --min_deposit $SMALL_MIN
echo "✓ Small Pool initialized"

echo "Initializing Medium Pool..."
stellar contract invoke \
  --id $MEDIUM_POOL_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- initialize \
  --admin $(stellar keys address $ADMIN_KEY) \
  --usdc_token $BLEND_USDC_ID \
  --blend_pool $BLEND_POOL_ID \
  --yield_rate $YIELD_RATE \
  --round_duration $ROUND_DURATION \
  --min_deposit $MEDIUM_MIN
echo "✓ Medium Pool initialized"

echo "Initializing Whale Pool..."
stellar contract invoke \
  --id $WHALE_POOL_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- initialize \
  --admin $(stellar keys address $ADMIN_KEY) \
  --usdc_token $BLEND_USDC_ID \
  --blend_pool $BLEND_POOL_ID \
  --yield_rate $YIELD_RATE \
  --round_duration $ROUND_DURATION \
  --min_deposit $WHALE_MIN
echo "✓ Whale Pool initialized"
echo ""

# ============================================
# PHASE 6: FUND POOLS WITH HOUSE MONEY
# ============================================
echo "Funding pools with house money (for demo yield)..."
echo ""

echo "Transferring 10 USDC to Small Pool..."
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- transfer \
  --from $(stellar keys address $ADMIN_KEY) \
  --to $SMALL_POOL_ID \
  --amount $SMALL_HOUSE
echo "✓ Small Pool funded with house money"

echo "Transferring 50 USDC to Medium Pool..."
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- transfer \
  --from $(stellar keys address $ADMIN_KEY) \
  --to $MEDIUM_POOL_ID \
  --amount $MEDIUM_HOUSE
echo "✓ Medium Pool funded with house money"

echo "Transferring 100 USDC to Whale Pool..."
stellar contract invoke \
  --id $BLEND_USDC_ID \
  --source $ADMIN_KEY \
  --network $NETWORK \
  -- transfer \
  --from $(stellar keys address $ADMIN_KEY) \
  --to $WHALE_POOL_ID \
  --amount $WHALE_HOUSE
echo "✓ Whale Pool funded with house money"
echo ""

# ============================================
# PHASE 7: GENERATE .ENV FILE
# ============================================
echo "Generating .env file..."
echo ""

cat > .env << EOF
# StellarLotto with Blend Integration
# Generated: $(date)

# Network Configuration
NETWORK=$NETWORK
HORIZON_URL=https://horizon-testnet.stellar.org
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org

# Blend Testnet Contracts
BLEND_USDC_ID=$BLEND_USDC_ID
BLEND_POOL_ID=$BLEND_POOL_ID

# USDC Issuer (your admin!)
USDC_ISSUER=$(stellar keys address $ADMIN_KEY)

# Admin and Player Addresses
ADMIN_ADDRESS=$(stellar keys address $ADMIN_KEY)
PLAYER1_ADDRESS=$(stellar keys address $PLAYER1_KEY)
PLAYER2_ADDRESS=$(stellar keys address $PLAYER2_KEY)
PLAYER3_ADDRESS=$(stellar keys address $PLAYER3_KEY)
WHALE_ADDRESS=$(stellar keys address $WHALE_KEY)

# Lottery Pool Contract IDs
SMALL_POOL_ID=$SMALL_POOL_ID
MEDIUM_POOL_ID=$MEDIUM_POOL_ID
WHALE_POOL_ID=$WHALE_POOL_ID

# Pool Configurations (in stroops: 1 USDC = 10000000)
SMALL_POOL_MIN=$SMALL_MIN
MEDIUM_POOL_MIN=$MEDIUM_MIN
WHALE_POOL_MIN=$WHALE_MIN

# House Money (pre-funded for demo yield)
SMALL_HOUSE=$SMALL_HOUSE
MEDIUM_HOUSE=$MEDIUM_HOUSE
WHALE_HOUSE=$WHALE_HOUSE

# Contract Settings
YIELD_RATE=$YIELD_RATE
ROUND_DURATION=$ROUND_DURATION

# Derived Values
STROOPS_PER_USDC=10000000
EOF

echo "✓ .env file created!"
echo ""