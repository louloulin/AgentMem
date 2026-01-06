"use client";

import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";

/**
 * 淡入动画组件
 */
export function FadeIn({ 
  children, 
  className, 
  delay = 0 
}: { 
  children: React.ReactNode; 
  className?: string; 
  delay?: number; 
}) {
  const [isVisible, setIsVisible] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setTimeout(() => setIsVisible(true), delay);
        }
      },
      { threshold: 0.1 }
    );

    if (ref.current) {
      observer.observe(ref.current);
    }

    return () => observer.disconnect();
  }, [delay]);

  return (
    <div
      ref={ref}
      className={cn(
        "transition-all duration-1000 ease-out",
        isVisible 
          ? "opacity-100 translate-y-0" 
          : "opacity-0 translate-y-8",
        className
      )}
    >
      {children}
    </div>
  );
}

/**
 * 滑入动画组件
 */
export function SlideIn({ 
  children, 
  direction = "left", 
  className, 
  delay = 0 
}: { 
  children: React.ReactNode; 
  direction?: "left" | "right" | "up" | "down"; 
  className?: string; 
  delay?: number; 
}) {
  const [isVisible, setIsVisible] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const getTransform = () => {
    if (!isVisible) {
      switch (direction) {
        case "left": return "-translate-x-8";
        case "right": return "translate-x-8";
        case "up": return "-translate-y-8";
        case "down": return "translate-y-8";
      }
    }
    return "translate-x-0 translate-y-0";
  };

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setTimeout(() => setIsVisible(true), delay);
        }
      },
      { threshold: 0.1 }
    );

    if (ref.current) {
      observer.observe(ref.current);
    }

    return () => observer.disconnect();
  }, [delay]);

  return (
    <div
      ref={ref}
      className={cn(
        "transition-all duration-700 ease-out",
        isVisible ? "opacity-100" : "opacity-0",
        getTransform(),
        className
      )}
    >
      {children}
    </div>
  );
}

/**
 * 缩放动画组件
 */
export function ScaleIn({ 
  children, 
  className, 
  delay = 0 
}: { 
  children: React.ReactNode; 
  className?: string; 
  delay?: number; 
}) {
  const [isVisible, setIsVisible] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setTimeout(() => setIsVisible(true), delay);
        }
      },
      { threshold: 0.1 }
    );

    if (ref.current) {
      observer.observe(ref.current);
    }

    return () => observer.disconnect();
  }, [delay]);

  return (
    <div
      ref={ref}
      className={cn(
        "transition-all duration-600 ease-out",
        isVisible 
          ? "opacity-100 scale-100" 
          : "opacity-0 scale-95",
        className
      )}
    >
      {children}
    </div>
  );
}

/**
 * 打字机效果组件
 */
export function TypeWriter({ 
  text, 
  speed = 50, 
  className 
}: { 
  text: string; 
  speed?: number; 
  className?: string; 
}) {
  const [displayText, setDisplayText] = useState("");
  const [currentIndex, setCurrentIndex] = useState(0);

  useEffect(() => {
    if (currentIndex < text.length) {
      const timeout = setTimeout(() => {
        setDisplayText(prev => prev + text[currentIndex]);
        setCurrentIndex(prev => prev + 1);
      }, speed);

      return () => clearTimeout(timeout);
    }
  }, [currentIndex, text, speed]);

  return (
    <span className={className}>
      {displayText}
      <span className="animate-pulse">|</span>
    </span>
  );
}

/**
 * 悬浮动画组件
 */
export function FloatingCard({ 
  children, 
  className 
}: { 
  children: React.ReactNode; 
  className?: string; 
}) {
  return (
    <div className={cn(
      "transform transition-all duration-300 ease-out hover:scale-105 hover:-translate-y-2 hover:shadow-2xl hover:shadow-purple-500/20",
      className
    )}>
      {children}
    </div>
  );
}

/**
 * 渐变文字动画
 */
export function GradientText({ 
  children, 
  className 
}: { 
  children: React.ReactNode; 
  className?: string; 
}) {
  return (
    <span className={cn(
      "bg-gradient-to-r from-purple-400 via-pink-400 to-purple-400 bg-clip-text text-transparent animate-gradient-x bg-size-300",
      className
    )}>
      {children}
    </span>
  );
}

/**
 * 脉冲动画组件
 */
export function PulseGlow({ 
  children, 
  className 
}: { 
  children: React.ReactNode; 
  className?: string; 
}) {
  return (
    <div className={cn(
      "animate-pulse-glow",
      className
    )}>
      {children}
    </div>
  );
}