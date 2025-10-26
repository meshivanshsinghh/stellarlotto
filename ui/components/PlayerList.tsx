'use client';

import { getGhostName, formatAddress } from '@/lib/stellar';
import { Ghost, Trophy } from 'lucide-react';
import { Badge } from '@/components/ui/badge';

interface PlayerListProps {
  players: string[];
  winner?: string | null;
  currentUser?: string | null;
}

export function PlayerList({ players, winner, currentUser }: PlayerListProps) {
  if (players.length === 0) {
    return (
      <div className="text-center py-8 text-text-muted">
        <Ghost className="w-12 h-12 mx-auto mb-2 opacity-50" />
        <p>No players yet... be the first! 👻</p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {players.map((player, index) => {
        const isWinner = winner === player;
        const isCurrentUser = currentUser === player;

        return (
          <div
            key={player}
            className={`
              flex items-center justify-between p-3 rounded-lg border
              ${isWinner ? 'bg-halloween-orange/20 border-halloween-orange' : 'bg-halloween-dark-gray border-halloween-dark-gray'}
              ${isCurrentUser && !isWinner && 'border-halloween-purple/50'}
            `}
          >
            <div className="flex items-center gap-3">
              {isWinner ? (
                <Trophy className="w-5 h-5 text-halloween-orange" />
              ) : (
                <Ghost className="w-5 h-5 text-halloween-purple" />
              )}
              <div>
                <p className="font-semibold text-text-light">
                  {getGhostName(player)}
                </p>
                <p className="text-xs text-text-muted font-mono">
                  {formatAddress(player)}
                </p>
              </div>
            </div>

            <div className="flex items-center gap-2">
              {isWinner && (
                <Badge className="bg-halloween-orange text-white">
                  Winner! 🎃
                </Badge>
              )}
              {isCurrentUser && !isWinner && (
                <Badge variant="outline" className="border-halloween-purple text-halloween-purple">
                  You
                </Badge>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}