'use client';

import { useState, useEffect } from 'react';
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { CountdownTimer } from './CountdownTimer';
import { PlayerList } from './PlayerList';
import { getCurrentRound, getPlayers, approveUSDC, enterLottery } from '@/lib/contracts';
import { stroopsToUsdc, usdcToStroops } from '@/lib/stellar';
import type { PoolConfig, Round } from '@/lib/types';
import { Coins, Users, TrendingUp } from 'lucide-react';

interface PoolCardProps {
  pool: PoolConfig;
  walletAddress: string | null;
}

export function PoolCard({ pool, walletAddress }: PoolCardProps) {
  const [round, setRound] = useState<Round | null>(null);
  const [players, setPlayers] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [entering, setEntering] = useState(false);
  const [amount, setAmount] = useState(pool.minDeposit.toString());

  // Fetch round data
  const fetchData = async () => {
    if (!pool.id) return;
    
    try {
      const [roundData, playersData] = await Promise.all([
        getCurrentRound(pool.id),
        getCurrentRound(pool.id).then(r => getPlayers(pool.id, r.id))
      ]);
      
      setRound(roundData);
      setPlayers(playersData);
    } catch (error) {
      console.error('Failed to fetch pool data:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
    const interval = setInterval(fetchData, 10000); // Refresh every 10s
    return () => clearInterval(interval);
  }, [pool.id]);

  const handleEnter = async () => {
    if (!walletAddress || !round) return;

    const entryAmount = parseFloat(amount);
    if (entryAmount < pool.minDeposit) {
      alert(`Minimum deposit is ${pool.minDeposit} USDC`);
      return;
    }

    setEntering(true);
    try {
      const stroops = usdcToStroops(entryAmount);
      
      // Step 1: Approve
      await approveUSDC(walletAddress, pool.id, stroops);
      
      // Step 2: Enter
      await enterLottery(pool.id, walletAddress, stroops);
      
      alert('Successfully entered lottery! 🎃');
      await fetchData();
    } catch (error: any) {
      alert(`Failed to enter: ${error.message}`);
    } finally {
      setEntering(false);
    }
  };

  if (loading) {
    return (
      <Card className="bg-halloween-dark-gray border-halloween-purple/30">
        <CardHeader>
          <Skeleton className="h-8 w-32" />
          <Skeleton className="h-4 w-48 mt-2" />
        </CardHeader>
        <CardContent>
          <Skeleton className="h-32 w-full" />
        </CardContent>
      </Card>
    );
  }

  if (!round) {
    return (
      <Card className="bg-halloween-dark-gray border-halloween-purple/30">
        <CardContent className="p-8 text-center text-text-muted">
          Failed to load pool data
        </CardContent>
      </Card>
    );
  }

  const totalDeposits = stroopsToUsdc(round.total_deposits);
  const totalYield = stroopsToUsdc(round.total_yield);

  return (
    <Card 
      className="bg-halloween-dark-gray border-2 transition-all hover:scale-[1.02]"
      style={{ borderColor: pool.color }}
    >
      <CardHeader>
        <div className="flex justify-between items-start">
          <div>
            <CardTitle className="text-2xl text-text-light flex items-center gap-2">
              {pool.name}
              <Badge 
                className="text-white"
                style={{ backgroundColor: pool.color }}
              >
                {pool.tier}
              </Badge>
            </CardTitle>
            <CardDescription className="text-text-muted mt-1">
              Min: {pool.minDeposit} USDC
            </CardDescription>
          </div>
          {round.is_active && (
            <CountdownTimer endTime={round.end_time} onExpire={fetchData} />
          )}
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Stats */}
        <div className="grid grid-cols-3 gap-3">
          <div className="bg-halloween-black/50 p-3 rounded-lg">
            <div className="flex items-center gap-2 text-halloween-purple mb-1">
              <Coins className="w-4 h-4" />
              <span className="text-xs">Pool</span>
            </div>
            <p className="text-lg font-bold text-white">{totalDeposits.toFixed(2)} USDC</p>
          </div>
          
          <div className="bg-halloween-black/50 p-3 rounded-lg">
            <div className="flex items-center gap-2 text-halloween-green mb-1">
              <TrendingUp className="w-4 h-4" />
              <span className="text-xs">Yield</span>
            </div>
            <p className="text-lg font-bold text-white">{totalYield.toFixed(2)} USDC</p>
          </div>

          <div className="bg-halloween-black/50 p-3 rounded-lg">
            <div className="flex items-center gap-2 text-halloween-orange mb-1">
              <Users className="w-4 h-4" />
              <span className="text-xs">Players</span>
            </div>
            <p className="text-lg font-bold text-white">{round.player_count}</p>
          </div>
        </div>

        {/* Players List */}
        <div>
          <h4 className="text-sm font-semibold text-text-light mb-2">Entered Players</h4>
          <PlayerList 
            players={players} 
            winner={round.winner} 
            currentUser={walletAddress}
          />
        </div>

        {/* Entry Form */}
        {round.is_active && walletAddress && (
          <div className="pt-4 border-t border-halloween-dark-gray">
            <div className="flex gap-2">
              <Input
                type="number"
                min={pool.minDeposit}
                step="1"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder={`Min ${pool.minDeposit} USDC`}
                className="bg-halloween-black border-halloween-purple/30 text-white"
              />
              <Button
                onClick={handleEnter}
                disabled={entering || !walletAddress}
                className="text-white font-semibold"
                style={{ backgroundColor: pool.color }}
              >
                {entering ? 'Entering...' : 'Enter Lottery'}
              </Button>
            </div>
          </div>
        )}

        {!walletAddress && round.is_active && (
          <p className="text-center text-text-muted text-sm py-4">
            Connect wallet to enter
          </p>
        )}
      </CardContent>
    </Card>
  );
}