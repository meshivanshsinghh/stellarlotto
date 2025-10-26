// Stellar Lottery Types
export interface Round {
  id: number;
  start_time: number | bigint; // Unix timestamp (can be BigInt from contract)
  end_time: number | bigint; // Unix timestamp (can be BigInt from contract)
  total_deposits: string; // i128 as string
  total_yield: string; // i128 as string
  winner: string | null; // Address or null
  is_active: boolean;
  player_count: number;
}

export interface PlayerEntry {
  player: string; // Address
  deposit: string; // i128 as string
  round_id: number;
  has_claimed: boolean;
}

export interface GlobalStats {
  total_rounds: number;
  total_volume: string; // i128 as string
  total_players: number;
  total_prizes_paid: string;
}

export interface PoolConfig {
  id: string;
  name: string;
  tier: "small" | "medium" | "whale";
  minDeposit: number; // in USDC
  minDepositStroops: string; // in stroops
  color: string; // Halloween theme color
}

export const POOLS: PoolConfig[] = [
  {
    id: process.env.NEXT_PUBLIC_SMALL_POOL_ID || "",
    name: "Spooky Pool",
    tier: "small",
    minDeposit: 10,
    minDepositStroops: "100000000",
    color: "#ff6b35", // halloween-orange
  },
  {
    id: process.env.NEXT_PUBLIC_MEDIUM_POOL_ID || "",
    name: "Haunted Pool",
    tier: "medium",
    minDeposit: 100,
    minDepositStroops: "1000000000",
    color: "#8b5cf6", // halloween-purple
  },
  {
    id: process.env.NEXT_PUBLIC_WHALE_POOL_ID || "",
    name: "Cursed Pool",
    tier: "whale",
    minDeposit: 500,
    minDepositStroops: "5000000000",
    color: "#10b981", // halloween-green
  },
];

export const STROOPS_PER_USDC = 10_000_000;
