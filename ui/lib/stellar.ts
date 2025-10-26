'use client';

import {
  TransactionBuilder,
  Networks,
  BASE_FEE,
  xdr,
  scValToNative,
  Address,
  Contract,
} from '@stellar/stellar-sdk';

// Import RPC from the correct path
import { Server } from '@stellar/stellar-sdk/rpc';

// Constants
export const STROOPS_PER_USDC = 10_000_000;

// Network configuration
export const RPC_URL = process.env.NEXT_PUBLIC_STELLAR_RPC_URL || 'https://soroban-testnet.stellar.org';
export const NETWORK_PASSPHRASE = process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE || 'Test SDF Network ; September 2015';
export const USDC_CONTRACT_ID = process.env.NEXT_PUBLIC_BLEND_USDC_ID || '';

// Initialize RPC server
export const server = new Server(RPC_URL);

/**
 * Check if Freighter wallet is installed
 */
export function isFreighterInstalled(): boolean {
  return typeof window !== 'undefined' && 'freighter' in window;
}

/**
 * Connect to Freighter wallet
 */
export async function connectWallet(): Promise<string> {
  if (!isFreighterInstalled()) {
    throw new Error('Freighter wallet not installed. Please install from https://freighter.app');
  }

  try {
    const publicKey = await window.freighter.getPublicKey();
    return publicKey;
  } catch (error) {
    console.error('Failed to connect wallet:', error);
    throw new Error('User denied wallet connection');
  }
}

/**
 * Get account from public key
 */
export async function getAccount(publicKey: string) {
  try {
    return await server.getAccount(publicKey);
  } catch (error) {
    console.error('Failed to get account:', error);
    throw new Error('Account not found. Please fund your account first.');
  }
}

/**
 * Build and submit a transaction
 */
export async function buildAndSubmitTransaction(
  publicKey: string,
  operation: xdr.Operation,
): Promise<any> {
  const account = await getAccount(publicKey);

  const transaction = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(180)
    .build();

  // Simulate first
  const simulated = await server.simulateTransaction(transaction);

  if ('error' in simulated) {
    throw new Error(`Simulation failed: ${simulated.error}`);
  }

  // Prepare transaction
  const prepared = await server.prepareTransaction(transaction);

  // Sign with Freighter
  const signedXDR = await window.freighter.signTransaction(prepared.toXDR(), {
    networkPassphrase: NETWORK_PASSPHRASE,
  });

  const signedTx = TransactionBuilder.fromXDR(signedXDR, NETWORK_PASSPHRASE);

  // Submit
  const sendResponse = await server.sendTransaction(signedTx);

  if (sendResponse.status === 'PENDING') {
    let getResponse = await server.getTransaction(sendResponse.hash);

    // Poll for completion
    while (getResponse.status === 'NOT_FOUND') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      getResponse = await server.getTransaction(sendResponse.hash);
    }

    if (getResponse.status === 'SUCCESS') {
      return getResponse;
    } else {
      throw new Error(`Transaction failed: ${getResponse.status}`);
    }
  } else {
    throw new Error(`Transaction submission failed`);
  }
}

/**
 * Parse contract result to native JavaScript value
 */
export function parseContractResult(result: string): any {
  const scVal = xdr.ScVal.fromXDR(result, 'base64');
  return scValToNative(scVal);
}

/**
 * Format stroops to USDC
 */
export function stroopsToUsdc(stroops: string | number): number {
  const amount = typeof stroops === 'string' ? BigInt(stroops) : BigInt(stroops);
  return Number(amount) / STROOPS_PER_USDC;
}

/**
 * Format USDC to stroops
 */
export function usdcToStroops(usdc: number): string {
  return (BigInt(Math.floor(usdc * STROOPS_PER_USDC))).toString();
}

/**
 * Format address for display (truncate middle)
 */
export function formatAddress(address: string, chars: number = 4): string {
  if (!address || address.length < chars * 2) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}

/**
 * Generate ghost name from address
 */
export function getGhostName(address: string): string {
  const prefixes = ['Spooky', 'Haunted', 'Phantom', 'Cursed', 'Shadow', 'Dark'];
  const names = ['Vampire', 'Banshee', 'Wraith', 'Poltergeist', 'Ghoul', 'Specter'];
  
  const seed = parseInt(address.slice(-8), 16);
  return `${prefixes[seed % 6]} ${names[(seed >> 4) % 6]}`;
}

/**
 * Format time remaining
 */
export function formatTimeRemaining(endTime: number | bigint): string {
  const now = Math.floor(Date.now() / 1000);
  
  // Convert BigInt to number if needed (timestamps from contracts)
  const endTimeNum = typeof endTime === 'bigint' ? Number(endTime) : endTime;
  const remaining = endTimeNum - now;

  if (remaining <= 0) return 'Ended';

  const minutes = Math.floor(remaining / 60);
  const seconds = remaining % 60;

  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }
  return `${seconds}s`;
}

// TypeScript declarations for Freighter
declare global {
  interface Window {
    freighter: {
      getPublicKey: () => Promise<string>;
      signTransaction: (xdr: string, opts: { networkPassphrase: string }) => Promise<string>;
    };
  }
}