#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, token, Address, Env, Vec};

// Constants
const INSTANCE_LIFETIME_THRESHOLD: u32 = 518400; // ~60 days
const INSTANCE_BUMP_AMOUNT: u32 = 1036800; // ~120 days

#[contracttype]
pub enum DataKey {
    Admin,
    UsdcToken,
    CurrentRound,
    Round(u32),
    PlayerDeposit(u32, Address),
    PlayerList(u32),
    YieldRate,
    RoundDuration,
    MinDeposit,
    TotalVolume,  // NEW: Track total USDC volume
    TotalPlayers, // NEW: Track total unique players
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Round {
    pub id: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub total_deposits: i128,
    pub total_yield: i128,
    pub winner: Option<Address>,
    pub is_active: bool,
    pub player_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerEntry {
    pub player: Address,
    pub deposit: i128,
    pub round_id: u32,
    pub has_claimed: bool,
}

// NEW: Global statistics struct
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalStats {
    pub total_rounds: u32,
    pub total_volume: i128,
    pub total_players: u32,
    pub total_prizes_paid: i128,
}

// ============ HELPER FUNCTIONS ============

fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

// ============ CONTRACT ============

#[contract]
pub struct LotteryPool;

#[contractimpl]
impl LotteryPool {
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_token: Address,
        yield_rate: u32,
        round_duration: u64,
        min_deposit: i128,
    ) {
        if is_initialized(&env) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        if yield_rate > 10000 {
            panic!("Yield rate cannot exceed 100%");
        }
        if min_deposit <= 0 {
            panic!("Minimum deposit must be positive");
        }
        if round_duration < 60 {
            panic!("Round duration must be at least 60 seconds");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::UsdcToken, &usdc_token);
        env.storage()
            .instance()
            .set(&DataKey::YieldRate, &yield_rate);
        env.storage()
            .instance()
            .set(&DataKey::RoundDuration, &round_duration);
        env.storage()
            .instance()
            .set(&DataKey::MinDeposit, &min_deposit);

        // Initialize global stats
        env.storage()
            .persistent()
            .set(&DataKey::TotalVolume, &0i128);
        env.storage()
            .persistent()
            .set(&DataKey::TotalPlayers, &0u32);

        let current_time = env.ledger().timestamp();
        let round = Round {
            id: 1,
            start_time: current_time,
            end_time: current_time + round_duration,
            total_deposits: 0,
            total_yield: 0,
            winner: None,
            is_active: true,
            player_count: 0,
        };

        env.storage().instance().set(&DataKey::CurrentRound, &1u32);
        env.storage().persistent().set(&DataKey::Round(1), &round);
        env.storage()
            .persistent()
            .set(&DataKey::PlayerList(1), &Vec::<Address>::new(&env));

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    }

    pub fn enter_lottery(env: Env, player: Address, amount: i128) {
        player.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let min_deposit: i128 = env.storage().instance().get(&DataKey::MinDeposit).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let current_round_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRound)
            .unwrap();

        if amount < min_deposit {
            panic!("Deposit below minimum");
        }
        if amount < 0 {
            panic!("Amount must be positive");
        }

        let mut round: Round = env
            .storage()
            .persistent()
            .get(&DataKey::Round(current_round_id))
            .unwrap();

        if !round.is_active {
            panic!("Round is not active");
        }

        let current_time = env.ledger().timestamp();
        if current_time >= round.end_time {
            panic!("Round has ended, please wait for winner selection");
        }

        let player_key = DataKey::PlayerDeposit(current_round_id, player.clone());
        if env.storage().persistent().has(&player_key) {
            panic!("Player already entered this round");
        }

        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&player, &env.current_contract_address(), &amount);

        let player_entry = PlayerEntry {
            player: player.clone(),
            deposit: amount,
            round_id: current_round_id,
            has_claimed: false,
        };

        env.storage().persistent().set(&player_key, &player_entry);

        let mut players: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::PlayerList(current_round_id))
            .unwrap_or(Vec::new(&env));
        players.push_back(player.clone());
        env.storage()
            .persistent()
            .set(&DataKey::PlayerList(current_round_id), &players);

        round.total_deposits += amount;
        round.player_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Round(current_round_id), &round);

        // NEW: Update global stats
        let total_vol: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalVolume)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::TotalVolume, &(total_vol + amount));

        let total_players: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalPlayers)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::TotalPlayers, &(total_players + 1));

        // Emit event using v21 syntax
        env.events().publish(
            (symbol_short!("entered"), player.clone()),
            (current_round_id, amount),
        );
    }

    pub fn get_current_round(env: Env) -> Round {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let current_round_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRound)
            .unwrap_or(0);

        if current_round_id == 0 {
            panic!("Contract not initialized");
        }

        env.storage()
            .persistent()
            .get(&DataKey::Round(current_round_id))
            .unwrap()
    }

    pub fn pick_winner(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let current_round_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRound)
            .unwrap();

        let mut round: Round = env
            .storage()
            .persistent()
            .get(&DataKey::Round(current_round_id))
            .unwrap();

        // Check round has ended
        let current_time = env.ledger().timestamp();
        if current_time < round.end_time {
            panic!("Round has not ended yet");
        }

        if !round.is_active {
            panic!("Round already finished");
        }

        // Get player list
        let players: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::PlayerList(current_round_id))
            .unwrap();

        if players.is_empty() {
            panic!("No players in this round");
        }

        // Calculate yield (mock 10% APY for demo)
        let yield_rate: u32 = env.storage().instance().get(&DataKey::YieldRate).unwrap();
        let round_duration: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RoundDuration)
            .unwrap();

        // Mock yield: (total * rate * duration) / (year in seconds * basis points)
        let mock_yield = (round.total_deposits * yield_rate as i128 * round_duration as i128)
            / (365 * 24 * 60 * 60 * 10000);

        // NEW: JACKPOT ROLLOVER - If less than 3 players, roll yield to next round!
        if players.len() < 3 {
            round.is_active = false;
            round.total_yield = mock_yield;

            // Refund all players their deposits
            let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
            let token_client = token::Client::new(&env, &usdc_token);

            for player in players.iter() {
                let entry: PlayerEntry = env
                    .storage()
                    .persistent()
                    .get(&DataKey::PlayerDeposit(current_round_id, player.clone()))
                    .unwrap();

                token_client.transfer(&env.current_contract_address(), &player, &entry.deposit);
            }

            env.storage()
                .persistent()
                .set(&DataKey::Round(current_round_id), &round);

            // Start new round WITH ROLLED OVER YIELD AS JACKPOT
            let new_round_id = current_round_id + 1;
            let new_round = Round {
                id: new_round_id,
                start_time: current_time,
                end_time: current_time + round_duration,
                total_deposits: mock_yield, // JACKPOT STARTS WITH PREVIOUS YIELD!
                total_yield: 0,
                winner: None,
                is_active: true,
                player_count: 0,
            };

            env.storage()
                .instance()
                .set(&DataKey::CurrentRound, &new_round_id);
            env.storage()
                .persistent()
                .set(&DataKey::Round(new_round_id), &new_round);
            env.storage().persistent().set(
                &DataKey::PlayerList(new_round_id),
                &Vec::<Address>::new(&env),
            );

            // Emit jackpot event
            env.events()
                .publish((symbol_short!("jackpot"), new_round_id), mock_yield);

            panic!("Not enough players! Yield rolled to next round as JACKPOT!");
        }

        // Normal flow: Select random winner
        let seed = env.ledger().timestamp() ^ (env.ledger().sequence() as u64);
        let winner_index = (seed % players.len() as u64) as u32;
        let winner = players.get(winner_index).unwrap();

        round.total_yield = mock_yield;
        round.winner = Some(winner.clone());
        round.is_active = false;

        // Get winner's deposit
        let winner_entry: PlayerEntry = env
            .storage()
            .persistent()
            .get(&DataKey::PlayerDeposit(current_round_id, winner.clone()))
            .unwrap();

        // Transfer prize (original deposit + all yield)
        let prize = winner_entry.deposit + mock_yield;
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &winner, &prize);

        // Update round
        env.storage()
            .persistent()
            .set(&DataKey::Round(current_round_id), &round);

        // Emit event
        env.events().publish(
            (symbol_short!("winner"), winner.clone()),
            (current_round_id, prize),
        );

        // Start new round
        let new_round_id = current_round_id + 1;
        let new_round = Round {
            id: new_round_id,
            start_time: current_time,
            end_time: current_time + round_duration,
            total_deposits: 0,
            total_yield: 0,
            winner: None,
            is_active: true,
            player_count: 0,
        };

        env.storage()
            .instance()
            .set(&DataKey::CurrentRound, &new_round_id);
        env.storage()
            .persistent()
            .set(&DataKey::Round(new_round_id), &new_round);
        env.storage().persistent().set(
            &DataKey::PlayerList(new_round_id),
            &Vec::<Address>::new(&env),
        );

        winner
    }

    pub fn claim_refund(env: Env, player: Address, round_id: u32) {
        player.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Get the round
        let round: Round = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .unwrap_or_else(|| panic!("Round not found"));

        // Check round is finished
        if round.is_active {
            panic!("Round is still active");
        }

        // Check there is a winner
        let winner = round
            .winner
            .as_ref()
            .unwrap_or_else(|| panic!("No winner selected yet"));

        // Check player is not the winner
        if winner == &player {
            panic!("Winner cannot claim refund");
        }

        // Get player entry
        let player_key = DataKey::PlayerDeposit(round_id, player.clone());
        let mut player_entry: PlayerEntry = env
            .storage()
            .persistent()
            .get(&player_key)
            .unwrap_or_else(|| panic!("Player not in this round"));

        // Check not already claimed
        if player_entry.has_claimed {
            panic!("Refund already claimed");
        }

        // Transfer refund
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(
            &env.current_contract_address(),
            &player,
            &player_entry.deposit,
        );

        // Mark as claimed
        player_entry.has_claimed = true;
        env.storage().persistent().set(&player_key, &player_entry);

        // Emit event
        env.events().publish(
            (symbol_short!("refund"), player.clone()),
            (round_id, player_entry.deposit),
        );
    }

    // FIXED: trick_or_treat now actually pays out USDC!
    pub fn trick_or_treat(env: Env, player: Address) -> i128 {
        player.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Simple random prize between 1-10 USDC (in stroops: 7 decimals)
        let seed = env.ledger().timestamp() ^ (env.ledger().sequence() as u64);
        let prize = ((seed % 10) + 1) as i128 * 10_000_000; // 1-10 USDC

        // ACTUALLY PAY THE PRIZE
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let token_client = token::Client::new(&env, &usdc_token);
        token_client.transfer(&env.current_contract_address(), &player, &prize);

        // Emit event
        env.events()
            .publish((symbol_short!("treat"), player.clone()), prize);

        prize
    }

    // NEW: Get global statistics
    pub fn get_stats(env: Env) -> GlobalStats {
        let current_round_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRound)
            .unwrap_or(1);

        let total_volume: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalVolume)
            .unwrap_or(0);

        let total_players: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalPlayers)
            .unwrap_or(0);

        GlobalStats {
            total_rounds: current_round_id - 1, // Minus current active round
            total_volume,
            total_players,
            total_prizes_paid: total_volume / 100, // Mock: assume 1% yield paid out
        }
    }

    // Get specific round info
    pub fn get_round(env: Env, round_id: u32) -> Round {
        env.storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .unwrap()
    }

    // Get player entry for a round
    pub fn get_player_entry(env: Env, round_id: u32, player: Address) -> Option<PlayerEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::PlayerDeposit(round_id, player))
    }

    // Get all players in current round
    pub fn get_players(env: Env, round_id: u32) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::PlayerList(round_id))
            .unwrap_or(Vec::new(&env))
    }
}
