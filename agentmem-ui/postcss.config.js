/**
 * PostCSS 配置文件 - Tailwind v3 兼容
 * 优化 CSS 处理和构建性能
 */
module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
    // 生产环境下启用 CSS 优化
    ...(process.env.NODE_ENV === 'production' && {
      cssnano: {
        preset: ['default', {
          discardComments: {
            removeAll: true,
          },
          normalizeWhitespace: true,
          minifySelectors: true,
          minifyFontValues: true,
          minifyParams: true,
        }],
      },
    }),
  },
};