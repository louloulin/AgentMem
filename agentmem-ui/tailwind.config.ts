import type { Config } from 'tailwindcss';
import tailwindcssAnimate from 'tailwindcss-animate';

/**
 * Tailwind CSS v3 配置文件 - 兼容 shadcn/ui
 * 支持深色/浅色主题切换和自定义动画
 */
const config: Config = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  darkMode: ['class'],
  theme: {
    container: {
      center: true,
      padding: '2rem',
      screens: {
        '2xl': '1400px',
      },
    },
    extend: {
      colors: {
        // Supabase Brand Colors
        'supabase-green': {
          DEFAULT: '#3ECF8E',
          light: '#4ADE95',
          dark: '#2CB574',
        },
        // Background Colors
        'bg-primary': '#1C1C1C',
        'bg-secondary': '#2A2A2A',
        'bg-tertiary': '#1A1A1A',
        // Original shadcn/ui colors (保持兼容)
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        primary: {
          DEFAULT: '#3ECF8E', // 改为 Supabase Green
          foreground: '#FFFFFF',
        },
        secondary: {
          DEFAULT: '#2A2A2A',
          foreground: '#FFFFFF',
        },
        destructive: {
          DEFAULT: 'hsl(var(--destructive))',
          foreground: 'hsl(var(--destructive-foreground))',
        },
        muted: {
          DEFAULT: '#2A2A2A',
          foreground: '#9CA3AF',
        },
        accent: {
          DEFAULT: '#3ECF8E',
          foreground: '#FFFFFF',
        },
        popover: {
          DEFAULT: '#1A1A1A',
          foreground: '#FFFFFF',
        },
        card: {
          DEFAULT: '#1A1A1A',
          foreground: '#FFFFFF',
        },
      },
      borderRadius: {
        lg: '1rem',     // 16px - Supabase style
        md: '0.75rem',  // 12px
        sm: '0.5rem',   // 8px
        xl: '1.5rem',   // 24px
        '2xl': '2rem',  // 32px
      },
      boxShadow: {
        'glow-green': '0 0 20px rgba(62, 207, 142, 0.3)',
        'glow-green-lg': '0 0 30px rgba(62, 207, 142, 0.4)',
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
        'gradient-primary': 'linear-gradient(135deg, #3ECF8E 0%, #2CB574 100%)',
        'gradient-hero': 'linear-gradient(180deg, #1C1C1C 0%, #0F0F0F 100%)',
        'gradient-card': 'linear-gradient(135deg, rgba(62, 207, 142, 0.1) 0%, rgba(44, 181, 116, 0.05) 100%)',
      },
      keyframes: {
        'fade-in': {
          '0%': { opacity: '0', transform: 'translateY(10px)' },
          '100%': { opacity: '1', transform: 'translateY(0)' },
        },
        'slide-in': {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(0)' },
        },
        'glow': {
          '0%, 100%': { boxShadow: '0 0 20px rgba(62, 207, 142, 0.2)' },
          '50%': { boxShadow: '0 0 30px rgba(62, 207, 142, 0.4)' },
        },
        'gradient-x': {
          '0%, 100%': {
            'background-position': '0% 50%',
          },
          '50%': {
            'background-position': '100% 50%',
          },
        },
        'pulse-glow': {
          '0%, 100%': {
            'box-shadow': '0 0 20px rgba(62, 207, 142, 0.3)',
          },
          '50%': {
            'box-shadow': '0 0 40px rgba(62, 207, 142, 0.6)',
          },
        },
        float: {
          '0%, 100%': {
            transform: 'translateY(0px)',
          },
          '50%': {
            transform: 'translateY(-10px)',
          },
        },
        shimmer: {
          '0%': {
            'background-position': '-200% 0',
          },
          '100%': {
            'background-position': '200% 0',
          },
        },
        'accordion-down': {
          from: { height: '0' },
          to: { height: 'var(--radix-accordion-content-height)' },
        },
        'accordion-up': {
          from: { height: 'var(--radix-accordion-content-height)' },
          to: { height: '0' },
        },
      },
      animation: {
        'fade-in': 'fade-in 0.5s ease-out',
        'slide-in': 'slide-in 0.3s ease-out',
        'glow': 'glow 2s ease-in-out infinite',
        'gradient-x': 'gradient-x 3s ease infinite',
        'pulse-glow': 'pulse-glow 2s ease-in-out infinite',
        'float': 'float 3s ease-in-out infinite',
        'shimmer': 'shimmer 2s infinite',
        'accordion-down': 'accordion-down 0.2s ease-out',
        'accordion-up': 'accordion-up 0.2s ease-out',
      },
      backgroundSize: {
        '300': '300% 300%',
      },
      fontFamily: {
        sans: ['Inter', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        mono: ['Fira Code', 'JetBrains Mono', 'Consolas', 'Courier New', 'monospace'],
      },
    },
  },
  plugins: [tailwindcssAnimate],
};

export default config;