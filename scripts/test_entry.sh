#!/bin/bash
# Test Lottery Entry Script

set -e

cd "$(dirname "$0")/.."

# Load environment variables
source .env

echo "Testing Lottery Entry..."
echo ""

ENTRY_AMOUNT=200000000  # 20 USDC
echo "Player1 entering Small Pool with 20 USDC..."

# Step 1: Approve
echo "Step 1: Approving spend..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player1 \
  --network $NETWORK \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- approve \
  --from $PLAYER1_ADDRESS \
  --spender $SMALL_POOL_ID \
  --amount $ENTRY_AMOUNT \
  --expiration_ledger 999999

echo "Approved!"

# Step 2: Enter lottery
echo "Step 2: Entering lottery..."
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player1 \
  --network $NETWORK \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- enter_lottery \
  --player $PLAYER1_ADDRESS \
  --amount $ENTRY_AMOUNT

echo ""
echo "SUCCESS! Player1 entered the lottery!"
echo ""

# Get current round info
echo "Current Round Info:"
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player1 \
  --network $NETWORK \
  --network-passphrase "Test SDF Network ; September 2015" \
  -- get_current_round

echo ""
echo "Test complete!"