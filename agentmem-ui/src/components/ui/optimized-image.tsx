"use client";

import { useState, useRef, useEffect } from 'react';
import { cn } from '@/lib/utils';

interface OptimizedImageProps {
  src: string;
  alt: string;
  width?: number;
  height?: number;
  className?: string;
  priority?: boolean;
  placeholder?: 'blur' | 'empty';
  blurDataURL?: string;
  onLoad?: () => void;
  onError?: () => void;
}

/**
 * 优化的图片组件，支持懒加载、占位符和错误处理
 */
export function OptimizedImage({
  src,
  alt,
  width,
  height,
  className,
  priority = false,
  placeholder = 'empty',
  blurDataURL,
  onLoad,
  onError,
}: OptimizedImageProps) {
  const [isLoaded, setIsLoaded] = useState(false);
  const [isError, setIsError] = useState(false);
  const [isInView, setIsInView] = useState(priority);
  const imgRef = useRef<HTMLImageElement>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);

  // 懒加载逻辑
  useEffect(() => {
    if (priority) return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsInView(true);
          observer.disconnect();
        }
      },
      {
        rootMargin: '50px',
        threshold: 0.1,
      }
    );

    observerRef.current = observer;

    if (imgRef.current) {
      observer.observe(imgRef.current);
    }

    return () => {
      observer.disconnect();
    };
  }, [priority]);

  const handleLoad = () => {
    setIsLoaded(true);
    onLoad?.();
  };

  const handleError = () => {
    setIsError(true);
    onError?.();
  };

  // 生成占位符
  const getPlaceholder = () => {
    if (placeholder === 'blur' && blurDataURL) {
      return blurDataURL;
    }
    
    // 生成简单的 SVG 占位符
    const svgPlaceholder = `data:image/svg+xml;base64,${btoa(
      `<svg width="${width || 400}" height="${
        height || 300
      }" xmlns="http://www.w3.org/2000/svg">
        <rect width="100%" height="100%" fill="#f1f5f9"/>
        <text x="50%" y="50%" font-family="Arial, sans-serif" font-size="14" fill="#64748b" text-anchor="middle" dy=".3em">
          ${alt || 'Loading...'}
        </text>
      </svg>`
    )}`;
    
    return svgPlaceholder;
  };

  return (
    <div
      ref={imgRef}
      className={cn(
        'relative overflow-hidden bg-slate-100 dark:bg-slate-800',
        className
      )}
      style={{ width, height }}
    >
      {/* 占位符 */}
      {!isLoaded && !isError && (
        <div className="absolute inset-0 flex items-center justify-center">
          {placeholder === 'blur' ? (
            <img
              src={getPlaceholder()}
              alt=""
              className="w-full h-full object-cover filter blur-sm scale-110"
            />
          ) : (
            <div className="w-full h-full bg-gradient-to-br from-slate-200 to-slate-300 dark:from-slate-700 dark:to-slate-800 animate-pulse" />
          )}
        </div>
      )}

      {/* 实际图片 */}
      {isInView && !isError && (
        <img
          src={src}
          alt={alt}
          width={width}
          height={height}
          className={cn(
            'w-full h-full object-cover transition-opacity duration-300',
            isLoaded ? 'opacity-100' : 'opacity-0'
          )}
          onLoad={handleLoad}
          onError={handleError}
          loading={priority ? 'eager' : 'lazy'}
        />
      )}

      {/* 错误状态 */}
      {isError && (
        <div className="absolute inset-0 flex items-center justify-center bg-slate-100 dark:bg-slate-800">
          <div className="text-center text-slate-500 dark:text-slate-400">
            <svg
              className="w-12 h-12 mx-auto mb-2"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
              />
            </svg>
            <p className="text-sm">图片加载失败</p>
          </div>
        </div>
      )}

      {/* 加载指示器 */}
      {isInView && !isLoaded && !isError && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full animate-spin" />
        </div>
      )}
    </div>
  );
}

/**
 * 图片画廊组件
 */
interface ImageGalleryProps {
  images: Array<{
    src: string;
    alt: string;
    caption?: string;
  }>;
  className?: string;
  columns?: number;
}

export function ImageGallery({ images, className, columns = 3 }: ImageGalleryProps) {
  return (
    <div
      className={cn(
        'grid gap-4',
        {
          'grid-cols-1': columns === 1,
          'grid-cols-2': columns === 2,
          'grid-cols-3': columns === 3,
          'grid-cols-4': columns === 4,
        },
        className
      )}
    >
      {images.map((image, index) => (
        <div key={index} className="group">
          <OptimizedImage
            src={image.src}
            alt={image.alt}
            className="w-full h-48 rounded-lg group-hover:scale-105 transition-transform duration-300"
          />
          {image.caption && (
            <p className="mt-2 text-sm text-slate-600 dark:text-slate-400 text-center">
              {image.caption}
            </p>
          )}
        </div>
      ))}
    </div>
  );
}

/**
 * 头像组件
 */
interface AvatarProps {
  src?: string;
  alt: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  fallback?: string;
  className?: string;
}

export function Avatar({ src, alt, size = 'md', fallback, className }: AvatarProps) {
  const [isError, setIsError] = useState(false);

  const sizeClasses = {
    sm: 'w-8 h-8 text-xs',
    md: 'w-12 h-12 text-sm',
    lg: 'w-16 h-16 text-base',
    xl: 'w-24 h-24 text-lg',
  };

  const getFallbackText = () => {
    if (fallback) return fallback;
    return alt
      .split(' ')
      .map(word => word[0])
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  return (
    <div
      className={cn(
        'relative rounded-full overflow-hidden bg-gradient-to-br from-purple-500 to-pink-500 flex items-center justify-center text-white font-semibold',
        sizeClasses[size],
        className
      )}
    >
      {src && !isError ? (
        <img
          src={src}
          alt={alt}
          className="w-full h-full object-cover"
          onError={() => setIsError(true)}
        />
      ) : (
        <span>{getFallbackText()}</span>
      )}
    </div>
  );
}