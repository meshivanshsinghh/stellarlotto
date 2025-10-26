'use client';

import { useState, useEffect } from 'react';
import { formatTimeRemaining } from '@/lib/stellar';
import { Clock } from 'lucide-react';

interface CountdownTimerProps {
  endTime: number;
  onExpire?: () => void;
}

export function CountdownTimer({ endTime, onExpire }: CountdownTimerProps) {
  const [timeLeft, setTimeLeft] = useState<string>('');
  const [isExpired, setIsExpired] = useState(false);

  useEffect(() => {
    const updateTimer = () => {
      const remaining = formatTimeRemaining(endTime);
      setTimeLeft(remaining);
      
      if (remaining === 'Ended' && !isExpired) {
        setIsExpired(true);
        onExpire?.();
      }
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);

    return () => clearInterval(interval);
  }, [endTime, isExpired, onExpire]);

  return (
    <div className="flex items-center gap-2 text-halloween-orange">
      <Clock className={`w-5 h-5 ${!isExpired && 'pulse-glow'}`} />
      <span className="text-lg font-bold font-mono">
        {isExpired ? '🎃 Round Ended!' : timeLeft}
      </span>
    </div>
  );
}