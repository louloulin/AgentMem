/**
 * Time Range Selector Component
 * 
 * Provides time-based filtering for memory search.
 * Supports: Last 7 days, Last 30 days, Last 90 days, All time.
 */

'use client';

import React from 'react';
import { Calendar, Clock } from 'lucide-react';

export type TimeRange = '7d' | '30d' | '90d' | 'all';

interface TimeRangeSelectorProps {
  /** Current selected range */
  value: TimeRange;
  /** Callback when range changes */
  onChange: (range: TimeRange) => void;
  /** Show label */
  showLabel?: boolean;
  /** Size variant */
  size?: 'sm' | 'md';
  /** Additional CSS classes */
  className?: string;
}

const TIME_RANGES: { value: TimeRange; label: string; description: string }[] = [
  { value: '7d', label: '7 Days', description: 'Last week' },
  { value: '30d', label: '30 Days', description: 'Last month' },
  { value: '90d', label: '90 Days', description: 'Last quarter' },
  { value: 'all', label: 'All Time', description: 'No filter' },
];

export function TimeRangeSelector({
  value,
  onChange,
  showLabel = true,
  size = 'md',
  className = '',
}: TimeRangeSelectorProps) {
  const paddingClasses = size === 'sm' ? 'px-2 py-1 text-xs gap-1' : 'px-3 py-1.5 text-sm gap-2';
  
  return (
    <div className={`flex items-center gap-2 ${className}`}>
      {showLabel && (
        <span className="text-sm text-gray-600 dark:text-gray-400 flex items-center gap-1">
          <Clock className="w-4 h-4" />
          Time:
        </span>
      )}
      
      <div className="flex rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
        {TIME_RANGES.map((range) => (
          <button
            key={range.value}
            onClick={() => onChange(range.value)}
            className={`
              ${paddingClasses}
              flex items-center gap-1
              font-medium transition-colors
              ${
                value === range.value
                  ? 'bg-blue-500 text-white'
                  : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700'
              }
              ${range.value !== 'all' ? 'border-r border-gray-200 dark:border-gray-700' : ''}
            `}
            title={range.description}
          >
            {range.label}
          </button>
        ))}
      </div>
    </div>
  );
}

/**
 * Time Range Helper
 * Converts range to timestamp for API calls
 */
export function timeRangeToTimestamp(range: TimeRange): number | null {
  const now = Date.now();
  const dayMs = 24 * 60 * 60 * 1000;
  
  switch (range) {
    case '7d':
      return now - (7 * dayMs);
    case '30d':
      return now - (30 * dayMs);
    case '90d':
      return now - (90 * dayMs);
    case 'all':
    default:
      return null;
  }
}

/**
 * Format timestamp for display
 */
export function formatTimestamp(timestamp: number | null): string {
  if (!timestamp) return 'All time';
  
  const date = new Date(timestamp);
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}
