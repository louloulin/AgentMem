import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { LanguageProvider } from "@/contexts/language-context";
import { ThemeProvider } from "next-themes";
import { PageLoadingProgress } from "@/components/ui/loading-progress";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: {
    default: "AgentMem - 智能记忆管理平台",
    template: "%s | AgentMem"
  },
  description: "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎。为 AI 代理提供强大的记忆能力，支持语义搜索、智能推理和实时学习。",
  keywords: [
    "AgentMem",
    "智能记忆",
    "AI记忆管理",
    "Rust",
    "DeepSeek",
    "记忆管理",
    "向量数据库",
    "语义搜索",
    "智能推理",
    "AI代理",
    "机器学习",
    "人工智能",
    "开源",
    "API",
    "SDK"
  ],
  authors: [{ name: "AgentMem Team", url: "https://agentmem.ai" }],
  creator: "AgentMem Team",
  publisher: "AgentMem",
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  alternates: {
    canonical: "https://agentmem.ai",
    languages: {
      'zh-CN': 'https://agentmem.ai/zh',
      'en-US': 'https://agentmem.ai/en',
    },
  },
  openGraph: {
    type: "website",
    locale: "zh_CN",
    url: "https://agentmem.ai",
    siteName: "AgentMem",
    title: "AgentMem - 智能记忆管理平台",
    description: "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎。为 AI 代理提供强大的记忆能力。",
    images: [
      {
        url: "https://agentmem.ai/og-image.png",
        width: 1200,
        height: 630,
        alt: "AgentMem - 智能记忆管理平台",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    site: "@AgentMem",
    creator: "@AgentMem",
    title: "AgentMem - 智能记忆管理平台",
    description: "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎。",
    images: ["https://agentmem.ai/twitter-image.png"],
  },
  verification: {
    google: "your-google-verification-code",
    yandex: "your-yandex-verification-code",
    yahoo: "your-yahoo-verification-code",
  },
  category: "Technology",
  classification: "AI & Machine Learning",
  referrer: "origin-when-cross-origin",
  formatDetection: {
    email: false,
    address: false,
    telephone: false,
  },
};

export const viewport = {
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const structuredData = {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    "name": "AgentMem",
    "description": "基于 Rust 构建的下一代智能记忆管理平台，集成 DeepSeek 推理引擎。为 AI 代理提供强大的记忆能力，支持语义搜索、智能推理和实时学习。",
    "url": "https://agentmem.ai",
    "applicationCategory": "DeveloperApplication",
    "operatingSystem": "Cross-platform",
    "programmingLanguage": "Rust",
    "author": {
      "@type": "Organization",
      "name": "AgentMem Team",
      "url": "https://agentmem.ai/about"
    },
    "publisher": {
      "@type": "Organization",
      "name": "AgentMem",
      "url": "https://agentmem.ai",
      "logo": {
        "@type": "ImageObject",
        "url": "https://agentmem.ai/logo.png",
        "width": 512,
        "height": 512
      }
    },
    "offers": {
      "@type": "Offer",
      "price": "0",
      "priceCurrency": "USD",
      "availability": "https://schema.org/InStock",
      "category": "Free"
    },
    "aggregateRating": {
      "@type": "AggregateRating",
      "ratingValue": "4.8",
      "ratingCount": "150",
      "bestRating": "5",
      "worstRating": "1"
    },
    "featureList": [
      "智能记忆管理",
      "语义搜索",
      "DeepSeek 推理引擎",
      "实时学习",
      "API 集成",
      "多语言支持",
      "开源"
    ],
    "screenshot": "https://agentmem.ai/screenshot.png",
    "downloadUrl": "https://github.com/agentmem/agentmem",
    "softwareVersion": "1.0.0",
    "datePublished": "2024-01-15",
    "dateModified": "2024-01-15",
    "license": "https://opensource.org/licenses/MIT"
  };

  const organizationData = {
    "@context": "https://schema.org",
    "@type": "Organization",
    "name": "AgentMem",
    "url": "https://agentmem.ai",
    "logo": "https://agentmem.ai/logo.png",
    "description": "专注于智能记忆管理技术的创新公司",
    "foundingDate": "2024",
    "industry": "Artificial Intelligence",
    "numberOfEmployees": "10-50",
    "address": {
      "@type": "PostalAddress",
      "addressCountry": "CN",
      "addressLocality": "北京"
    },
    "contactPoint": {
      "@type": "ContactPoint",
      "contactType": "customer service",
      "email": "support@agentmem.ai",
      "url": "https://agentmem.ai/support"
    },
    "sameAs": [
      "https://github.com/agentmem",
      "https://twitter.com/agentmem",
      "https://linkedin.com/company/agentmem"
    ]
  };

  return (
    <html lang="zh" suppressHydrationWarning>
      <head>
        {/* 结构化数据 */}
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(structuredData),
          }}
        />
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(organizationData),
          }}
        />
        
        {/* 预连接到外部资源 */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        
        {/* DNS 预取 */}
        <link rel="dns-prefetch" href="//api.agentmem.ai" />
        <link rel="dns-prefetch" href="//cdn.agentmem.ai" />
        
        {/* 网站图标 */}
        <link rel="icon" href="/favicon.ico" sizes="any" />
        <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
        <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
        <link rel="manifest" href="/manifest.json" />
        
        {/* 主题颜色 */}
        <meta name="theme-color" content="#7c3aed" />
        <meta name="msapplication-TileColor" content="#7c3aed" />
        
        {/* 安全策略 */}
        <meta httpEquiv="X-Content-Type-Options" content="nosniff" />
        <meta httpEquiv="X-Frame-Options" content="DENY" />
        <meta httpEquiv="X-XSS-Protection" content="1; mode=block" />
        
        {/* 性能提示 */}
        <meta httpEquiv="Accept-CH" content="DPR, Viewport-Width, Width" />
      </head>
      <body className={inter.className}>
        <ThemeProvider
          attribute="class"
          defaultTheme="dark"
          enableSystem
          disableTransitionOnChange
        >
          <LanguageProvider>
            <PageLoadingProgress />
            {children}
          </LanguageProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
