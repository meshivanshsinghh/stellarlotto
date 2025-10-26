'use client';

import { useState, useEffect } from 'react';
import { connectWallet, isFreighterInstalled, formatAddress } from '@/lib/stellar';
import { Button } from '@/components/ui/button';
import { Wallet, Ghost } from 'lucide-react';

export function WalletConnect() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [hasFreighter, setHasFreighter] = useState(false);

  useEffect(() => {
    setHasFreighter(isFreighterInstalled());
  }, []);

  const handleConnect = async () => {
    setIsConnecting(true);
    try {
      const address = await connectWallet();
      setWalletAddress(address);
      localStorage.setItem('walletAddress', address);
    } catch (error: any) {
      alert(error.message);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnect = () => {
    setWalletAddress(null);
    localStorage.removeItem('walletAddress');
  };

  // Auto-reconnect on mount
  useEffect(() => {
    const savedAddress = localStorage.getItem('walletAddress');
    if (savedAddress && hasFreighter) {
      setWalletAddress(savedAddress);
    }
  }, [hasFreighter]);

  if (!hasFreighter) {
    return (
      <a
        href="https://freighter.app"
        target="_blank"
        rel="noopener noreferrer"
        className="px-6 py-3 bg-halloween-orange hover:bg-orange-600 text-white font-semibold rounded-lg glow-orange transition-all"
      >
        Install Freighter Wallet
      </a>
    );
  }

  if (walletAddress) {
    return (
      <div className="flex items-center gap-3 bg-halloween-dark-gray px-4 py-2 rounded-lg border border-halloween-orange/30">
        <Ghost className="w-5 h-5 text-halloween-orange" />
        <span className="text-text-light font-mono">{formatAddress(walletAddress)}</span>
        <Button
          onClick={handleDisconnect}
          variant="ghost"
          size="sm"
          className="text-text-muted hover:text-white"
        >
          Disconnect
        </Button>
      </div>
    );
  }

  return (
    <Button
      onClick={handleConnect}
      disabled={isConnecting}
      className="bg-halloween-orange hover:bg-orange-600 text-white font-semibold glow-orange"
    >
      <Wallet className="w-5 h-5 mr-2" />
      {isConnecting ? 'Connecting...' : 'Connect Wallet'}
    </Button>
  );
}