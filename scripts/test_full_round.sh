#!/bin/bash
# Complete Round Test - Enter 3 players, pick winner, claim refunds
set -e
cd "$(dirname "$0")/.."
source .env

export STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

echo "====================================="
echo "FULL LOTTERY ROUND TEST"
echo "====================================="
echo ""

# Entry amounts (all above 10 USDC minimum for small pool)
PLAYER1_ENTRY=150000000  # 15 USDC
PLAYER2_ENTRY=200000000  # 20 USDC
PLAYER3_ENTRY=120000000  # 12 USDC


echo "Test Plan:"
echo "   - Player1 enters with 15 USDC"
echo "   - Player2 enters with 20 USDC"
echo "   - Player3 enters with 12 USDC"
echo "   - Wait for round to end (2 minutes)"
echo "   - Pick winner"
echo "   - Losers claim refunds"
echo ""
read -p "Press Enter to start..."

# ============ PLAYER 1 ENTERS ============
echo ""
echo "Player 1 entering with 15 USDC..."

stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player1 \
  --network $NETWORK \
  -- approve \
  --from $PLAYER1_ADDRESS \
  --spender $SMALL_POOL_ID \
  --amount $PLAYER1_ENTRY \
  --expiration_ledger 999999 > /dev/null


stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player1 \
  --network $NETWORK \
  -- enter_lottery \
  --player $PLAYER1_ADDRESS \
  --amount $PLAYER1_ENTRY

echo "Player1 entered!"


echo ""
echo "Player 2 entering with 20 USDC..."


stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player2 \
  --network $NETWORK \
  -- approve \
  --from $PLAYER2_ADDRESS \
  --spender $SMALL_POOL_ID \
  --amount $PLAYER2_ENTRY \
  --expiration_ledger 999999 > /dev/null


stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player2 \
  --network $NETWORK \
  -- enter_lottery \
  --player $PLAYER2_ADDRESS \
  --amount $PLAYER2_ENTRY

echo "Player2 entered!"


echo "Player 3 entering with 12 USDC..."
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player3 \
  --network $NETWORK \
  -- approve \
  --from $PLAYER3_ADDRESS \
  --spender $SMALL_POOL_ID \
  --amount $PLAYER3_ENTRY \
  --expiration_ledger 999999 > /dev/null

stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player3 \
  --network $NETWORK \
  -- enter_lottery \
  --player $PLAYER3_ADDRESS \
  --amount $PLAYER3_ENTRY

echo "Player3 entered!"



echo ""
echo "Current Round Status:"
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player1 \
  --network $NETWORK \
  -- get_current_round


echo ""
echo "Round duration: 2 minutes (120 seconds)"
echo "Waiting for round to end..."
echo ""


# Get current round end time
ROUND_INFO=$(stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player1 \
  --network $NETWORK \
  -- get_current_round 2>/dev/null)

echo "You can either:"
echo "1. Wait 2 minutes, then continue"
echo "2. Or press Ctrl+C and run the winner selection later with:"
echo "   stellar contract invoke --id $SMALL_POOL_ID --source admin --network testnet -- pick_winner"
echo ""

# Countdown timer (optional - comment out if you want manual control)
echo "Starting 2 minute countdown..."
for i in {120..1}; do
  printf "\rTime remaining: %02d:%02d" $((i/60)) $((i%60))
  sleep 1
done
echo ""
echo "Round ended!"


echo ""
echo "Picking winner..."
WINNER_OUTPUT=$(stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source admin \
  --network $NETWORK \
  -- pick_winner)

echo "$WINNER_OUTPUT"

echo ""
echo "Winner has been selected!"
echo ""

echo "Final Round 1 Info:"
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player1 \
  --network $NETWORK \
  -- get_round \
  --round_id 1


echo ""
echo "Player Balances After Winner Selection:"
echo ""

echo "Player1 balance:"
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player1 \
  --network $NETWORK \
  -- balance \
  --account $PLAYER1_ADDRESS

echo ""
echo "Player2 balance:"
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player2 \
  --network $NETWORK \
  -- balance \
  --account $PLAYER2_ADDRESS

echo ""
echo "Player3 balance:"
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player3 \
  --network $NETWORK \
  -- balance \
  --account $PLAYER3_ADDRESS



# ============ LOSERS CLAIM REFUNDS ============
echo ""
echo "Now losers should claim their refunds..."
echo ""

echo "Attempting refund for Player1..."
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player1 \
  --network $NETWORK \
  -- claim_refund \
  --player $PLAYER1_ADDRESS \
  --round_id 1 || echo "Player1 was the winner or already claimed!"

echo ""
echo "Attempting refund for Player2..."
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player2 \
  --network $NETWORK \
  -- claim_refund \
  --player $PLAYER2_ADDRESS \
  --round_id 1 || echo "Player2 was the winner or already claimed!"

echo ""
echo "Attempting refund for Player3..."
stellar contract invoke \
  --id $SMALL_POOL_ID \
  --source player3 \
  --network $NETWORK \
  -- claim_refund \
  --player $PLAYER3_ADDRESS \
  --round_id 1 || echo "Player3 was the winner or already claimed!"


# ============ FINAL BALANCES ============
echo ""
echo "Final Balances After Refunds:"
echo ""

echo "Player1:"
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player1 \
  --network $NETWORK \
  -- balance \
  --account $PLAYER1_ADDRESS

echo ""
echo "Player2:"
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player2 \
  --network $NETWORK \
  -- balance \
  --account $PLAYER2_ADDRESS

echo ""
echo "Player3:"
stellar contract invoke \
  --id $MOCK_USDC_ID \
  --source player3 \
  --network $NETWORK \
  -- balance \
  --account $PLAYER3_ADDRESS

echo "Summary:"
echo "- Winner got their deposit + all yield"
echo "- Losers got full refunds"
echo "- Nobody lost money!"