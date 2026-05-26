/**
 * Decay Progress Bar Component
 * 
 * Displays memory decay/health status visually.
 * Shows how much "life" remains in a memory based on decay score.
 */

'use client';

import React from 'react';
import { AlertTriangle } from 'lucide-react';

interface DecayProgressProps {
  /** Decay score (0.0 = fully decayed, 1.0 = fresh) */
  decayScore: number;
  /** Show percentage label */
  showLabel?: boolean;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
  /** Additional CSS classes */
  className?: string;
}

export function DecayProgress({
  decayScore,
  showLabel = true,
  size = 'md',
  className = '',
}: DecayProgressProps) {
  // Normalize to 0-100%
  const percentage = Math.max(0, Math.min(100, Math.round(decayScore * 100)));
  
  // Size mapping
  const heightClasses = {
    sm: 'h-1.5',
    md: 'h-2',
    lg: 'h-3',
  };
  
  const textSizes = {
    sm: 'text-xs',
    md: 'text-sm',
    lg: 'text-base',
  };

  // Color based on health
  const getColor = (pct: number) => {
    if (pct >= 70) return 'bg-green-500';
    if (pct >= 40) return 'bg-yellow-500';
    if (pct >= 20) return 'bg-orange-500';
    return 'bg-red-500';
  };

  // Health status
  const getStatus = (pct: number) => {
    if (pct >= 70) return 'Healthy';
    if (pct >= 40) return 'Aging';
    if (pct >= 20) return 'Fading';
    return 'Critical';
  };

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      {/* Progress Bar */}
      <div className={`flex-1 ${heightClasses[size]} w-full bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden`}>
        <div
          className={`${heightClasses[size]} ${getColor(percentage)} transition-all duration-300 rounded-full`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      
      {/* Label */}
      {showLabel && (
        <div className="flex items-center gap-1">
          {percentage < 20 && (
            <AlertTriangle className={`${textSizes[size]} text-red-500`} />
          )}
          <span className={`${textSizes[size]} font-medium text-gray-700 dark:text-gray-300 min-w-[50px]`}>
            {percentage}%
          </span>
        </div>
      )}
    </div>
  );
}

/**
 * Decay Status Badge
 * Shows a status badge with icon
 */
interface DecayStatusBadgeProps {
  /** Decay score (0.0 - 1.0) */
  decayScore: number;
  /** Size variant */
  size?: 'sm' | 'md';
}

export function DecayStatusBadge({ decayScore, size = 'md' }: DecayStatusBadgeProps) {
  const percentage = Math.max(0, Math.min(100, Math.round(decayScore * 100)));
  
  const getStatus = (pct: number) => {
    if (pct >= 70) {
      return { 
        label: 'Healthy', 
        color: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
        icon: '✓'
      };
    }
    if (pct >= 40) {
      return { 
        label: 'Aging', 
        color: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
        icon: '~'
      };
    }
    if (pct >= 20) {
      return { 
        label: 'Fading', 
        color: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
        icon: '!'
      };
    }
    return { 
      label: 'Critical', 
      color: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
      icon: '✗'
    };
  };
  
  const status = getStatus(percentage);
  const paddingClasses = size === 'sm' ? 'px-1.5 py-0.5 text-xs' : 'px-2 py-1 text-sm';
  
  return (
    <span className={`inline-flex items-center gap-1 rounded-full font-medium ${paddingClasses} ${status.color}`}>
      <span>{status.icon}</span>
      <span>{status.label}</span>
      <span className="opacity-70">({percentage}%)</span>
    </span>
  );
}
