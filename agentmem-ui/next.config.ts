import type { NextConfig } from "next";

/**
 * Next.js 配置文件
 * 支持 Turbopack 和样式优化
 */
const nextConfig: NextConfig = {
  // Turbopack 配置（Next.js 15.5.2 推荐方式）
  turbopack: {
    root: process.cwd(),
    rules: {
      '*.svg': {
        loaders: ['@svgr/webpack'],
        as: '*.js',
      },
    },
  },
  // 编译器选项
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production',
  },
  // 图片优化
  images: {
    formats: ['image/webp', 'image/avif'],
  },
  // ESLint配置
  eslint: {
    // 即使有警告也继续构建
    ignoreDuringBuilds: true,
  },
};

export default nextConfig;
