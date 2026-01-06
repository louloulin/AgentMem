"use client";

import { useState, useRef, useEffect } from "react";
import { cn } from "@/lib/utils";
import { Loader2, ImageIcon } from "lucide-react";

/**
 * 响应式图片组件属性
 */
interface ResponsiveImageProps {
  src: string;
  alt: string;
  className?: string;
  width?: number;
  height?: number;
  priority?: boolean;
  lazy?: boolean;
  placeholder?: 'blur' | 'empty' | React.ReactNode;
  blurDataURL?: string;
  sizes?: string;
  quality?: number;
  onLoad?: () => void;
  onError?: () => void;
}

/**
 * 生成响应式图片 URL
 */
const generateResponsiveUrl = (src: string, width: number, quality: number = 75) => {
  // 如果是外部 URL 或已经包含参数，直接返回
  if (src.startsWith('http') || src.includes('?')) {
    return src;
  }
  
  // 对于内部图片，可以添加优化参数
  return `${src}?w=${width}&q=${quality}`;
};

/**
 * 响应式图片组件
 */
export function ResponsiveImage({
  src,
  alt,
  className,
  width,
  height,
  priority = false,
  lazy = true,
  placeholder = 'blur',
  blurDataURL,
  sizes = '(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw',
  quality = 75,
  onLoad,
  onError
}: ResponsiveImageProps) {
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const [isInView, setIsInView] = useState(!lazy || priority);
  const imgRef = useRef<HTMLImageElement>(null);
  const observerRef = useRef<IntersectionObserver | null>(null);

  // 懒加载逻辑
  useEffect(() => {
    if (!lazy || priority || isInView) return;

    observerRef.current = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setIsInView(true);
            observerRef.current?.disconnect();
          }
        });
      },
      {
        rootMargin: '50px'
      }
    );

    if (imgRef.current) {
      observerRef.current.observe(imgRef.current);
    }

    return () => {
      observerRef.current?.disconnect();
    };
  }, [lazy, priority, isInView]);

  const handleLoad = () => {
    setIsLoading(false);
    onLoad?.();
  };

  const handleError = () => {
    setIsLoading(false);
    setHasError(true);
    onError?.();
  };

  // 生成 srcSet
  const generateSrcSet = () => {
    if (!width) return undefined;
    
    const breakpoints = [480, 768, 1024, 1280, 1920];
    return breakpoints
      .filter(bp => bp <= width * 2) // 只生成不超过原图2倍的尺寸
      .map(bp => `${generateResponsiveUrl(src, bp, quality)} ${bp}w`)
      .join(', ');
  };

  return (
    <div 
      ref={imgRef}
      className={cn(
        "relative overflow-hidden",
        className
      )}
      style={{ width, height }}
    >
      {/* 占位符 */}
      {(isLoading || !isInView) && (
        <div className="absolute inset-0 flex items-center justify-center bg-slate-800">
          {placeholder === 'blur' && blurDataURL ? (
            <img
              src={blurDataURL}
              alt=""
              className="absolute inset-0 w-full h-full object-cover filter blur-sm scale-110"
            />
          ) : placeholder === 'empty' ? (
            <div className="w-full h-full bg-slate-700" />
          ) : typeof placeholder === 'string' ? (
            <div className="flex flex-col items-center justify-center text-slate-400">
              <ImageIcon className="h-8 w-8 mb-2" />
              <span className="text-sm">{placeholder}</span>
            </div>
          ) : (
            placeholder
          )}
          
          {isLoading && isInView && (
            <div className="absolute inset-0 flex items-center justify-center bg-slate-800/50">
              <Loader2 className="h-6 w-6 animate-spin text-white" />
            </div>
          )}
        </div>
      )}

      {/* 错误状态 */}
      {hasError && (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-slate-800 text-slate-400">
          <ImageIcon className="h-12 w-12 mb-2" />
          <span className="text-sm">图片加载失败</span>
        </div>
      )}

      {/* 实际图片 */}
      {isInView && !hasError && (
        <img
          src={generateResponsiveUrl(src, width || 1200, quality)}
          srcSet={generateSrcSet()}
          sizes={sizes}
          alt={alt}
          width={width}
          height={height}
          loading={priority ? 'eager' : 'lazy'}
          onLoad={handleLoad}
          onError={handleError}
          className={cn(
            "w-full h-full object-cover transition-opacity duration-300",
            isLoading ? "opacity-0" : "opacity-100"
          )}
        />
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
  gap?: number;
}

export function ImageGallery({ 
  images, 
  className, 
  columns = 3,
  gap = 4 
}: ImageGalleryProps) {
  return (
    <div 
      className={cn(
        "grid gap-4",
        columns === 2 && "grid-cols-1 md:grid-cols-2",
        columns === 3 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
        columns === 4 && "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
        className
      )}
      style={{ gap: `${gap * 0.25}rem` }}
    >
      {images.map((image, index) => (
        <div key={index} className="group">
          <ResponsiveImage
            src={image.src}
            alt={image.alt}
            className="rounded-lg overflow-hidden group-hover:scale-105 transition-transform duration-300"
            lazy={index > 2} // 前3张图片不懒加载
          />
          {image.caption && (
            <p className="mt-2 text-sm text-slate-400 text-center">
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
  className?: string;
  fallback?: string;
}

export function Avatar({ 
  src, 
  alt, 
  size = 'md', 
  className,
  fallback 
}: AvatarProps) {
  const [hasError, setHasError] = useState(false);
  
  const sizeClasses = {
    sm: 'w-8 h-8 text-xs',
    md: 'w-12 h-12 text-sm',
    lg: 'w-16 h-16 text-base',
    xl: 'w-24 h-24 text-lg'
  };

  const getFallbackText = () => {
    if (fallback) return fallback;
    return alt.split(' ').map(word => word[0]).join('').toUpperCase().slice(0, 2);
  };

  return (
    <div className={cn(
      "relative rounded-full overflow-hidden bg-slate-700 flex items-center justify-center",
      sizeClasses[size],
      className
    )}>
      {src && !hasError ? (
        <ResponsiveImage
          src={src}
          alt={alt}
          className="w-full h-full"
          onError={() => setHasError(true)}
          lazy={false}
          priority
        />
      ) : (
        <span className="font-semibold text-slate-300">
          {getFallbackText()}
        </span>
      )}
    </div>
  );
}

/**
 * 背景图片组件
 */
interface BackgroundImageProps {
  src: string;
  alt: string;
  children: React.ReactNode;
  className?: string;
  overlay?: boolean;
  overlayOpacity?: number;
}

export function BackgroundImage({ 
  src, 
  alt, 
  children, 
  className,
  overlay = true,
  overlayOpacity = 0.5 
}: BackgroundImageProps) {
  const [isLoaded, setIsLoaded] = useState(false);

  return (
    <div className={cn("relative overflow-hidden", className)}>
      <ResponsiveImage
        src={src}
        alt={alt}
        className="absolute inset-0 w-full h-full object-cover"
        onLoad={() => setIsLoaded(true)}
        priority
      />
      
      {overlay && (
        <div 
          className="absolute inset-0 bg-black transition-opacity duration-300"
          style={{ 
            opacity: isLoaded ? overlayOpacity : 0 
          }}
        />
      )}
      
      <div className="relative z-10">
        {children}
      </div>
    </div>
  );
}