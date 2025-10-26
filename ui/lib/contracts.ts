'use client';

import { Contract, Address, nativeToScVal, xdr, TransactionBuilder } from '@stellar/stellar-sdk';
import { buildAndSubmitTransaction, server, parseContractResult, USDC_CONTRACT_ID } from './stellar';
import type { Round, PlayerEntry, GlobalStats } from './types';

/**
 * Get current round data
 */
export async function getCurrentRound(poolId: string): Promise<Round> {
  const contract = new Contract(poolId);
  const account = await server.getAccount('GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF');
  
  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase: process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE!,
  })
    .addOperation(contract.call('get_current_round'))
    .setTimeout(0)
    .build();

  const simulated = await server.simulateTransaction(tx);
  
  if ('result' in simulated && simulated.result) {
    return parseContractResult(simulated.result.retval.toXDR('base64'));
  }

  throw new Error('Failed to get current round');
}

/**
 * Get players in a round
 */
export async function getPlayers(poolId: string, roundId: number): Promise<string[]> {
  const contract = new Contract(poolId);
  const account = await server.getAccount('GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF');
  
  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase: process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE!,
  })
    .addOperation(contract.call('get_players', nativeToScVal(roundId, { type: 'u32' })))
    .setTimeout(0)
    .build();

  const simulated = await server.simulateTransaction(tx);
  
  if ('result' in simulated && simulated.result) {
    return parseContractResult(simulated.result.retval.toXDR('base64'));
  }

  return [];
}

/**
 * Get global stats
 */
export async function getStats(poolId: string): Promise<GlobalStats> {
  const contract = new Contract(poolId);
  const account = await server.getAccount('GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF');
  
  const tx = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase: process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE!,
  })
    .addOperation(contract.call('get_stats'))
    .setTimeout(0)
    .build();

  const simulated = await server.simulateTransaction(tx);
  
  if ('result' in simulated && simulated.result) {
    return parseContractResult(simulated.result.retval.toXDR('base64'));
  }

  throw new Error('Failed to get stats');
}

// ==================== WRITE OPERATIONS ====================

/**
 * Approve USDC spending
 */
export async function approveUSDC(
  userPublicKey: string,
  spender: string,
  amount: string
): Promise<void> {
  const contract = new Contract(USDC_CONTRACT_ID);
  
  const operation = contract.call(
    'approve',
    new Address(userPublicKey).toScVal(),
    new Address(spender).toScVal(),
    nativeToScVal(BigInt(amount), { type: 'i128' }),
    nativeToScVal(2000000, { type: 'u32' }) // expiration ledger
  );

  await buildAndSubmitTransaction(userPublicKey, operation);
}

/**
 * Enter lottery
 */
export async function enterLottery(
  poolId: string,
  userPublicKey: string,
  amount: string
): Promise<void> {
  const contract = new Contract(poolId);
  
  const operation = contract.call(
    'enter_lottery',
    new Address(userPublicKey).toScVal(),
    nativeToScVal(BigInt(amount), { type: 'i128' })
  );

  await buildAndSubmitTransaction(userPublicKey, operation);
}

/**
 * Claim refund
 */
export async function claimRefund(
  poolId: string,
  userPublicKey: string,
  roundId: number
): Promise<void> {
  const contract = new Contract(poolId);
  
  const operation = contract.call(
    'claim_refund',
    new Address(userPublicKey).toScVal(),
    nativeToScVal(roundId, { type: 'u32' })
  );

  await buildAndSubmitTransaction(userPublicKey, operation);
}