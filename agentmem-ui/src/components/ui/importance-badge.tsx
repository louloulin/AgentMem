/**
 * Importance Badge Component
 * 
 * Displays memory importance as a visual rating (1-5 stars).
 * Uses the existing importance_score field from the backend.
 */

'use client';

import React from 'react';
import { Star, StarHalf } from 'lucide-react';

interface ImportanceBadgeProps {
  /** Importance score (0.0 - 1.0) */
  score: number;
  /** Show numeric score */
  showScore?: boolean;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
  /** Additional CSS classes */
  className?: string;
}

export function ImportanceBadge({
  score,
  showScore = true,
  size = 'md',
  className = '',
}: ImportanceBadgeProps) {
  // Normalize score to 0-1 range
  const normalizedScore = Math.max(0, Math.min(1, score));
  
  // Convert to 5-star rating
  const starRating = Math.round(normalizedScore * 5);
  
  // Size mapping
  const sizeClasses = {
    sm: 'w-3 h-3',
    md: 'w-4 h-4',
    lg: 'w-5 h-5',
  };
  
  const textSizes = {
    sm: 'text-xs',
    md: 'text-sm',
    lg: 'text-base',
  };

  // Color based on rating
  const getColor = (rating: number) => {
    if (rating >= 4) return 'text-yellow-500 fill-yellow-500';
    if (rating >= 3) return 'text-blue-500 fill-blue-500';
    if (rating >= 2) return 'text-gray-500 fill-gray-400';
    return 'text-gray-300 fill-gray-300';
  };

  return (
    <div className={`flex items-center gap-1 ${className}`}>
      {/* Star Rating */}
      <div className="flex gap-0.5">
        {[1, 2, 3, 4, 5].map((star) => (
          <Star
            key={star}
            className={`${sizeClasses[size]} ${
              star <= starRating
                ? getColor(starRating)
                : 'text-gray-200 dark:text-gray-700'
            }`}
            fill={star <= starRating ? 'currentColor' : 'none'}
          />
        ))}
      </div>
      
      {/* Numeric Score */}
      {showScore && (
        <span className={`${textSizes[size]} text-gray-500 dark:text-gray-400 ml-1`}>
          {normalizedScore.toFixed(1)}
        </span>
      )}
    </div>
  );
}

/**
 * Importance Level Badge
 * Shows a colored label based on importance level
 */
interface ImportanceLevelProps {
  /** Importance score (0.0 - 1.0) */
  score: number;
  size?: 'sm' | 'md';
}

export function ImportanceLevel({ score, size = 'md' }: ImportanceLevelProps) {
  const normalizedScore = Math.max(0, Math.min(1, score));
  
  // Determine level
  const getLevel = (s: number) => {
    if (s >= 0.7) return { label: 'High', color: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' };
    if (s >= 0.4) return { label: 'Medium', color: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200' };
    return { label: 'Low', color: 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300' };
  };
  
  const level = getLevel(normalizedScore);
  const paddingClasses = size === 'sm' ? 'px-1.5 py-0.5 text-xs' : 'px-2 py-1 text-sm';
  
  return (
    <span className={`inline-flex items-center rounded-full font-medium ${paddingClasses} ${level.color}`}>
      {level.label}
    </span>
  );
}
