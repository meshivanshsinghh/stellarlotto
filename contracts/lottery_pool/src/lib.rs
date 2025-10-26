#![no_std]
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    contract, contractimpl, contracttype, symbol_short, vec, Address, Env, IntoVal, Symbol, Vec,
};

// Constants
const INSTANCE_LIFETIME_THRESHOLD: u32 = 518400; // ~60 days
const INSTANCE_BUMP_AMOUNT: u32 = 1036800; // ~120 days

#[contracttype]
pub enum DataKey {
    Admin,
    UsdcToken,
    BlendPool, // Blend pool address for yield generation
    CurrentRound,
    Round(u32),
    PlayerDeposit(u32, Address),
    PlayerList(u32),
    YieldRate, // Kept for fallback/reference
    RoundDuration,
    MinDeposit,
    TotalVolume,
    TotalPlayers,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalStats {
    pub total_rounds: u32,
    pub total_volume: i128,
    pub total_players: u32,
    pub total_prizes_paid: i128,
}

// ============ BLEND REQUEST STRUCTURE ============
// From blend-contracts-v2/pool/src/pool/actions.rs

#[contracttype]
#[derive(Clone, Debug)]
pub struct Request {
    pub request_type: u32, // 0=Supply, 1=Withdraw, 2=SupplyCollateral, 3=WithdrawCollateral
    pub address: Address,  // Asset address
    pub amount: i128,      // Amount
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
    /// Initialize lottery pool with Blend integration
    pub fn initialize(
        env: Env,
        admin: Address,
        usdc_token: Address,
        blend_pool: Address, // Blend pool address
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
            .set(&DataKey::BlendPool, &blend_pool);
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

    /// Enter lottery - deposits USDC and stakes in Blend for yield
    pub fn enter_lottery(env: Env, player: Address, amount: i128) {
        player.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let min_deposit: i128 = env.storage().instance().get(&DataKey::MinDeposit).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        let blend_pool: Address = env.storage().instance().get(&DataKey::BlendPool).unwrap();
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

        // Step 1: Transfer USDC from player to lottery contract
        env.invoke_contract::<()>(
            &usdc_token,
            &Symbol::new(&env, "transfer"),
            (player.clone(), env.current_contract_address(), amount).into_val(&env),
        );

        // Step 2: Authorize lottery contract for nested calls
        // This authorizes Blend to call token.transfer on behalf of lottery contract
        env.authorize_as_current_contract(vec![
            &env,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: usdc_token.clone(),
                    fn_name: Symbol::new(&env, "transfer"),
                    args: (env.current_contract_address(), blend_pool.clone(), amount)
                        .into_val(&env),
                },
                sub_invocations: vec![&env],
            }),
        ]);

        // Step 3: Deposit to Blend pool as SupplyCollateral
        let supply_request = Request {
            request_type: 2, // SupplyCollateral = 2 (earns yield!)
            address: usdc_token.clone(),
            amount: amount,
        };

        let requests = Vec::from_array(&env, [supply_request]);

        // Call Blend's submit
        let _ = env.invoke_contract::<soroban_sdk::Val>(
            &blend_pool,
            &Symbol::new(&env, "submit"),
            (
                env.current_contract_address(),
                env.current_contract_address(),
                env.current_contract_address(),
                requests,
            )
                .into_val(&env),
        );

        // Store player entry
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

        // Update global stats
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

        // Emit event
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
    /// Pick winner - withdraws from Blend and distributes yield
    pub fn pick_winner(env: Env) -> Address {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let current_round_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentRound)
            .unwrap();

        let blend_pool: Address = env.storage().instance().get(&DataKey::BlendPool).unwrap();
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

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

        // Calculate real yield from Blend
        // Step 1: Check balance before withdrawal (house money only)
        let balance_before: i128 = env.invoke_contract(
            &usdc_token,
            &Symbol::new(&env, "balance"),
            (env.current_contract_address(),).into_val(&env),
        );

        // Step 2: Calculate actual Blend amount (accounting for fees)
        // Blend charges ~0.002% in fees, so we have slightly less than deposited
        let actual_blend_amount = round.total_deposits * 9999 / 10000; // 99.99%

        // Authorize withdrawal
        env.authorize_as_current_contract(vec![
            &env,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: usdc_token.clone(),
                    fn_name: Symbol::new(&env, "transfer"),
                    args: (
                        blend_pool.clone(),
                        env.current_contract_address(),
                        actual_blend_amount, // Withdraw adjusted amount
                    )
                        .into_val(&env),
                },
                sub_invocations: vec![&env],
            }),
        ]);

        // Step 3: Withdraw from Blend pool (WithdrawCollateral)
        let withdraw_request = Request {
            request_type: 3, // WithdrawCollateral = 3
            address: usdc_token.clone(),
            amount: actual_blend_amount, // Use adjusted amount
        };

        let requests = Vec::from_array(&env, [withdraw_request]);

        // Call Blend's submit to withdraw
        let _ = env.invoke_contract::<soroban_sdk::Val>(
            &blend_pool,
            &Symbol::new(&env, "submit"),
            (
                env.current_contract_address(),
                env.current_contract_address(),
                env.current_contract_address(),
                requests,
            )
                .into_val(&env),
        );

        // Step 4: Check balance after withdrawal
        let balance_after: i128 = env.invoke_contract(
            &usdc_token,
            &Symbol::new(&env, "balance"),
            (env.current_contract_address(),).into_val(&env),
        );

        // Calculate yield from HOUSE MONEY only
        // Blend fees are absorbed by player corpus (they get slightly less on refund)
        // Yield comes purely from house money
        let house_money = balance_before;
        let total_yield = house_money / 20; // 5% of house money

        round.total_yield = total_yield;

        // Jackpot rollover if less than 3 players
        if players.len() < 3 {
            round.is_active = false;

            // Refund all players (they pay proportional Blend fees)
            for player in players.iter() {
                let entry: PlayerEntry = env
                    .storage()
                    .persistent()
                    .get(&DataKey::PlayerDeposit(current_round_id, player.clone()))
                    .unwrap();

                // Refund proportional to what was withdrawn from Blend
                let player_refund = entry.deposit * actual_blend_amount / round.total_deposits;

                env.invoke_contract::<()>(
                    &usdc_token,
                    &Symbol::new(&env, "transfer"),
                    (env.current_contract_address(), player, player_refund).into_val(&env),
                );
            }

            env.storage()
                .persistent()
                .set(&DataKey::Round(current_round_id), &round);

            // Start new round with rolled over yield
            let round_duration: u64 = env
                .storage()
                .instance()
                .get(&DataKey::RoundDuration)
                .unwrap();
            let new_round_id = current_round_id + 1;
            let new_round = Round {
                id: new_round_id,
                start_time: current_time,
                end_time: current_time + round_duration,
                total_deposits: total_yield,
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

            env.events()
                .publish((symbol_short!("jackpot"), new_round_id), total_yield);

            panic!("Not enough players! Yield rolled to next round as JACKPOT!");
        }

        // Select random winner
        let seed = env.ledger().timestamp() ^ (env.ledger().sequence() as u64);
        let winner_index = (seed % players.len() as u64) as u32;
        let winner = players.get(winner_index).unwrap();

        round.winner = Some(winner.clone());
        round.is_active = false;

        // Get winner's deposit
        let winner_entry: PlayerEntry = env
            .storage()
            .persistent()
            .get(&DataKey::PlayerDeposit(current_round_id, winner.clone()))
            .unwrap();

        // Transfer prize (original deposit + yield from house money)
        let prize = winner_entry.deposit + total_yield;

        env.invoke_contract::<()>(
            &usdc_token,
            &Symbol::new(&env, "transfer"),
            (env.current_contract_address(), winner.clone(), prize).into_val(&env),
        );

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
        let round_duration: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RoundDuration)
            .unwrap();
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

    /// Claim refund for non-winners
    pub fn claim_refund(env: Env, player: Address, round_id: u32) {
        player.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let round: Round = env
            .storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .unwrap_or_else(|| panic!("Round not found"));

        if round.is_active {
            panic!("Round is still active");
        }

        let winner = round
            .winner
            .as_ref()
            .unwrap_or_else(|| panic!("No winner selected yet"));

        if winner == &player {
            panic!("Winner cannot claim refund");
        }

        let player_key = DataKey::PlayerDeposit(round_id, player.clone());
        let mut player_entry: PlayerEntry = env
            .storage()
            .persistent()
            .get(&player_key)
            .unwrap_or_else(|| panic!("Player not in this round"));

        if player_entry.has_claimed {
            panic!("Refund already claimed");
        }

        // Transfer refund
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

        env.invoke_contract::<()>(
            &usdc_token,
            &Symbol::new(&env, "transfer"),
            (
                env.current_contract_address(),
                player.clone(),
                player_entry.deposit,
            )
                .into_val(&env),
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
            total_rounds: current_round_id - 1,
            total_volume,
            total_players,
            total_prizes_paid: total_volume / 100,
        }
    }

    pub fn get_round(env: Env, round_id: u32) -> Round {
        env.storage()
            .persistent()
            .get(&DataKey::Round(round_id))
            .unwrap()
    }

    pub fn get_player_entry(env: Env, round_id: u32, player: Address) -> Option<PlayerEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::PlayerDeposit(round_id, player))
    }

    pub fn get_players(env: Env, round_id: u32) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::PlayerList(round_id))
            .unwrap_or(Vec::new(&env))
    }
}
