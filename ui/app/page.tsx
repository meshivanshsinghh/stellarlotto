'use client';

import { useState } from 'react';
import { WalletConnect } from '@/components/WalletConnect';
import { PoolCard } from '@/components/PoolCard';
import { POOLS } from '@/lib/types';
import { Ghost } from 'lucide-react';

export default function Home() {
  const [walletAddress, setWalletAddress] = useState<string | null>(null);

  return (
    <div className="min-h-screen bg-linear-to-b from-halloween-black via-halloween-dark-gray to-halloween-black">
      {/* Header */}
      <header className="border-b border-halloween-orange/20 backdrop-blur-sm sticky top-0 z-50 bg-halloween-black/80">
        <div className="container mx-auto px-4 py-4">
          <div className="flex justify-between items-center">
            <div className="flex items-center gap-3">
              <Ghost className="w-8 h-8 text-halloween-orange float-animation" />
              <h1 className="text-3xl font-bold text-halloween-orange" style={{ fontFamily: 'var(--font-creepster)' }}>
                StellarLotto
              </h1>
            </div>
            <WalletConnect />
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <section className="py-16 text-center">
        <div className="container mx-auto px-4">
          <h2 className="text-5xl md:text-6xl font-bold text-halloween-orange mb-4" style={{ fontFamily: 'var(--font-creepster)' }}>
            No-Loss Lottery 🎃
          </h2>
          <p className="text-xl text-text-light max-w-2xl mx-auto mb-8">
            Win crypto prizes with <span className="text-halloween-orange font-bold">ZERO risk</span>! 
            Losers get full refunds. Built on Stellar with Blend Protocol yield.
          </p>
          
          <div className="flex justify-center gap-6 text-sm text-text-muted">
            <div>✅ No-loss guarantee</div>
            <div>✅ Real DeFi yield</div>
            <div>✅ Provably fair</div>
          </div>
        </div>
      </section>

      {/* Pools Grid */}
      <section className="py-12">
        <div className="container mx-auto px-4">
          <h3 className="text-3xl font-bold text-text-light mb-8 text-center">
            Choose Your Pool
          </h3>
          
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-7xl mx-auto">
            {POOLS.map((pool) => (
              <PoolCard 
                key={pool.id} 
                pool={pool} 
                walletAddress={walletAddress}
              />
            ))}
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-8 border-t border-halloween-orange/20 mt-16">
        <div className="container mx-auto px-4 text-center text-text-muted">
          <p>Built for EasyA x Stellar Harvard Hack-o-Ween 2025 🎃</p>
        </div>
      </footer>
    </div>
  );
}