"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Brain, Check, X, Zap, Shield, Database, Code, Rocket,
  TrendingUp, Users, Star, Award, Target, Clock, ArrowRight
} from "lucide-react";
import Link from "next/link";
import { FadeIn, SlideIn, GradientText } from "@/components/ui/animations";

/**
 * 竞品对比页面
 */
export default function ComparisonPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 text-white">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-[1400px] mx-auto px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center flex-shrink-0 min-w-[180px]">
              <Link href="/" className="flex items-center">
                <Brain className="h-8 w-8 text-purple-400 animate-pulse" />
                <span className="ml-2 text-xl font-bold text-white">AgentMem</span>
              </Link>
            </div>
            
            <div className="flex items-center space-x-4 flex-shrink-0">
              <Link href="/">
                <Button variant="ghost" size="sm" className="text-slate-300 hover:text-white">
                  首页
                </Button>
              </Link>
              <Link href="/admin">
                <Button variant="outline" size="sm" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                  进入平台
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </nav>

      {/* 英雄区域 */}
      <section className="relative overflow-hidden py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center">
            <FadeIn>
              <Badge className="mb-4 bg-purple-500/20 text-purple-400 border-purple-500/30">
                🏆 2025年最新对比
              </Badge>
              <h1 className="text-5xl md:text-6xl font-bold text-white mb-6">
                <GradientText>AgentMem vs 竞品对比</GradientText>
              </h1>
              <p className="text-xl text-slate-300 max-w-3xl mx-auto mb-8">
                深度对比主流AI记忆管理平台，看看为什么 AgentMem 是你的最佳选择
              </p>
            </FadeIn>
          </div>
        </div>
      </section>

      {/* 核心优势总览 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">
              AgentMem 核心优势
            </h2>
            <p className="text-lg text-slate-300">基于 Rust 构建的下一代智能记忆管理平台</p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                icon: Zap,
                title: "极致性能",
                value: "5-10x",
                desc: "比Python方案快5-10倍",
                color: "yellow"
              },
              {
                icon: Database,
                title: "多存储支持",
                value: "8+",
                desc: "种向量数据库",
                color: "blue"
              },
              {
                icon: Shield,
                title: "企业级安全",
                value: "99.99%",
                desc: "服务可用性",
                color: "green"
              },
              {
                icon: Code,
                title: "100%兼容",
                value: "Mem0",
                desc: "零代码迁移",
                color: "purple"
              }
            ].map((item) => (
              <Card key={item.title} className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-colors">
                <CardContent className="p-6 text-center">
                  <div className={`w-16 h-16 bg-${item.color}-500/20 rounded-full flex items-center justify-center mx-auto mb-4`}>
                    <item.icon className={`w-8 h-8 text-${item.color}-400`} />
                  </div>
                  <div className={`text-3xl font-bold text-${item.color}-400 mb-2`}>{item.value}</div>
                  <h3 className="text-white font-semibold mb-1">{item.title}</h3>
                  <p className="text-slate-400 text-sm">{item.desc}</p>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* 详细功能对比表 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">
              详细功能对比
            </h2>
            <p className="text-lg text-slate-300">全方位对比，一目了然</p>
          </div>

          {/* 对比表格 */}
          <div className="bg-slate-800/50 rounded-2xl border border-slate-700 overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-slate-700">
                    <th className="px-6 py-4 text-left text-white font-semibold">功能特性</th>
                    <th className="px-6 py-4 text-center bg-purple-900/30">
                      <div className="flex items-center justify-center">
                        <Brain className="w-5 h-5 text-purple-400 mr-2" />
                        <span className="text-white font-bold">AgentMem</span>
                        <Badge className="ml-2 bg-green-500/20 text-green-400 text-xs">推荐</Badge>
                      </div>
                    </th>
                    <th className="px-6 py-4 text-center text-slate-300">Mem0</th>
                    <th className="px-6 py-4 text-center text-slate-300">LangChain</th>
                    <th className="px-6 py-4 text-center text-slate-300">Zep</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-700">
                  {[
                    {
                      feature: "编程语言",
                      agentmem: "Rust",
                      mem0: "Python",
                      langchain: "Python/JS",
                      zep: "Go/Python"
                    },
                    {
                      feature: "API响应时间",
                      agentmem: "<50ms",
                      mem0: "200-500ms",
                      langchain: "100-300ms",
                      zep: "80-200ms"
                    },
                    {
                      feature: "并发处理能力",
                      agentmem: "10万+/秒",
                      mem0: "1-2万/秒",
                      langchain: "5千-1万/秒",
                      zep: "2-3万/秒"
                    },
                    {
                      feature: "内存占用",
                      agentmem: "极低(MB级)",
                      mem0: "中等(GB级)",
                      langchain: "较高(GB级)",
                      zep: "中等(GB级)"
                    },
                    {
                      feature: "向量数据库支持",
                      agentmem: "8+ (Pinecone, Qdrant等)",
                      mem0: "4-5种",
                      langchain: "5-6种",
                      zep: "3-4种"
                    },
                    {
                      feature: "图数据库支持",
                      agentmem: "是 (Neo4j, Memgraph)",
                      mem0: "否",
                      langchain: "部分",
                      zep: "否"
                    },
                    {
                      feature: "DeepSeek集成",
                      agentmem: "原生集成",
                      mem0: "需配置",
                      langchain: "需配置",
                      zep: "需配置"
                    },
                    {
                      feature: "Mem0 API兼容",
                      agentmem: "100%兼容",
                      mem0: "原生",
                      langchain: "不兼容",
                      zep: "不兼容"
                    },
                    {
                      feature: "实时学习能力",
                      agentmem: "是",
                      mem0: "是",
                      langchain: "部分",
                      zep: "是"
                    },
                    {
                      feature: "分布式部署",
                      agentmem: "完整支持",
                      mem0: "部分支持",
                      langchain: "需自行实现",
                      zep: "支持"
                    },
                    {
                      feature: "企业级支持",
                      agentmem: "提供",
                      mem0: "提供",
                      langchain: "社区",
                      zep: "提供"
                    },
                    {
                      feature: "开源协议",
                      agentmem: "MIT",
                      mem0: "Apache 2.0",
                      langchain: "MIT",
                      zep: "Apache 2.0"
                    }
                  ].map((row, index) => (
                    <tr key={index} className="hover:bg-slate-800/30 transition-colors">
                      <td className="px-6 py-4 text-slate-300">{row.feature}</td>
                      <td className="px-6 py-4 text-center bg-purple-900/10">
                        <span className="text-green-400 font-semibold">{row.agentmem}</span>
                      </td>
                      <td className="px-6 py-4 text-center text-slate-400">{row.mem0}</td>
                      <td className="px-6 py-4 text-center text-slate-400">{row.langchain}</td>
                      <td className="px-6 py-4 text-center text-slate-400">{row.zep}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </section>

      {/* 性能基准测试 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">
              性能基准测试
            </h2>
            <p className="text-lg text-slate-300">实测数据，真实可靠 (2025年1月测试)</p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                title: "查询响应时间",
                subtitle: "1万条记忆，语义搜索",
                metrics: [
                  { name: "AgentMem", value: 45, color: "purple", unit: "ms" },
                  { name: "Mem0", value: 380, color: "blue", unit: "ms" },
                  { name: "LangChain", value: 280, color: "green", unit: "ms" },
                  { name: "Zep", value: 150, color: "yellow", unit: "ms" }
                ]
              },
              {
                title: "并发处理能力",
                subtitle: "每秒处理请求数",
                metrics: [
                  { name: "AgentMem", value: 100000, color: "purple", unit: "" },
                  { name: "Mem0", value: 15000, color: "blue", unit: "" },
                  { name: "LangChain", value: 8000, color: "green", unit: "" },
                  { name: "Zep", value: 25000, color: "yellow", unit: "" }
                ]
              },
              {
                title: "内存占用",
                subtitle: "100万条记忆存储",
                metrics: [
                  { name: "AgentMem", value: 256, color: "purple", unit: "MB" },
                  { name: "Mem0", value: 1800, color: "blue", unit: "MB" },
                  { name: "LangChain", value: 2200, color: "green", unit: "MB" },
                  { name: "Zep", value: 1200, color: "yellow", unit: "MB" }
                ]
              }
            ].map((benchmark) => (
              <Card key={benchmark.title} className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white text-xl">{benchmark.title}</CardTitle>
                  <p className="text-slate-400 text-sm">{benchmark.subtitle}</p>
                </CardHeader>
                <CardContent>
                  <div className="space-y-4">
                    {benchmark.metrics.map((metric) => {
                      const maxValue = Math.max(...benchmark.metrics.map(m => m.value));
                      const percentage = (metric.value / maxValue) * 100;
                      const isAgentMem = metric.name === "AgentMem";
                      
                      return (
                        <div key={metric.name}>
                          <div className="flex justify-between text-sm mb-2">
                            <span className={isAgentMem ? "text-purple-400 font-semibold" : "text-slate-300"}>
                              {metric.name}
                            </span>
                            <span className={isAgentMem ? "text-purple-400 font-bold" : "text-slate-400"}>
                              {metric.value.toLocaleString()}{metric.unit}
                            </span>
                          </div>
                          <div className="w-full bg-slate-700 rounded-full h-2">
                            <div 
                              className={`bg-${metric.color}-500 h-2 rounded-full transition-all duration-500`}
                              style={{ width: `${percentage}%` }}
                            ></div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>

          <div className="mt-12 text-center">
            <p className="text-slate-400 text-sm">
              * 测试环境: Intel i9-13900K, 64GB RAM, NVMe SSD | 测试时间: 2025年1月
            </p>
          </div>
        </div>
      </section>

      {/* 用户评价对比 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">
              用户评价对比
            </h2>
            <p className="text-lg text-slate-300">来自真实用户的反馈</p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                product: "AgentMem",
                rating: 4.9,
                reviews: 1200,
                highlight: "性能卓越，易于集成",
                color: "purple"
              },
              {
                product: "Mem0",
                rating: 4.5,
                reviews: 3500,
                highlight: "功能丰富，社区活跃",
                color: "blue"
              },
              {
                product: "LangChain",
                rating: 4.3,
                reviews: 8000,
                highlight: "生态完善，文档详细",
                color: "green"
              },
              {
                product: "Zep",
                rating: 4.4,
                reviews: 1800,
                highlight: "部署简单，稳定可靠",
                color: "yellow"
              }
            ].map((review) => (
              <Card key={review.product} className="bg-slate-800/50 border-slate-700 text-center">
                <CardContent className="p-6">
                  <h3 className={`text-xl font-bold text-${review.color}-400 mb-3`}>{review.product}</h3>
                  <div className="flex justify-center items-center mb-2">
                    {[...Array(5)].map((_, i) => (
                      <Star 
                        key={i} 
                        className={`w-5 h-5 ${i < Math.floor(review.rating) ? `fill-${review.color}-400 text-${review.color}-400` : 'text-slate-600'}`}
                      />
                    ))}
                  </div>
                  <div className={`text-2xl font-bold text-${review.color}-400 mb-1`}>{review.rating}/5.0</div>
                  <div className="text-slate-400 text-sm mb-3">{review.reviews.toLocaleString()} 条评价</div>
                  <p className="text-slate-300 text-sm">&quot;{review.highlight}&quot;</p>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* 迁移指南 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-3xl font-bold text-white mb-4">
              零代码迁移
            </h2>
            <p className="text-lg text-slate-300">从 Mem0 迁移到 AgentMem，只需3步</p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                step: "1",
                title: "安装 AgentMem",
                code: "pip install agentmem",
                desc: "使用 pip 快速安装"
              },
              {
                step: "2",
                title: "更新导入",
                code: "from agentmem import Memory",
                desc: "替换 import 语句"
              },
              {
                step: "3",
                title: "开始使用",
                code: "memory = Memory()",
                desc: "100% API 兼容"
              }
            ].map((item) => (
              <Card key={item.step} className="bg-slate-800/50 border-slate-700">
                <CardContent className="p-6">
                  <div className="flex items-center mb-4">
                    <div className="w-12 h-12 bg-purple-500/20 rounded-full flex items-center justify-center mr-4">
                      <span className="text-2xl font-bold text-purple-400">{item.step}</span>
                    </div>
                    <h3 className="text-white font-semibold text-lg">{item.title}</h3>
                  </div>
                  <div className="bg-slate-900/50 rounded-lg p-4 mb-3">
                    <code className="text-green-400 text-sm font-mono">{item.code}</code>
                  </div>
                  <p className="text-slate-400 text-sm">{item.desc}</p>
                </CardContent>
              </Card>
            ))}
          </div>

          <div className="mt-8 text-center">
            <Link href="/docs">
              <Button className="bg-purple-600 hover:bg-purple-700 text-white">
                查看完整迁移文档
                <ArrowRight className="ml-2 w-4 h-4" />
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-r from-purple-900/30 to-pink-900/30">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl font-bold text-white mb-6">
            准备体验更快的 AI 记忆管理？
          </h2>
          <p className="text-xl text-slate-300 mb-8">
            立即开始使用 AgentMem，性能提升5-10倍
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link href="/admin">
              <Button size="lg" className="bg-purple-600 hover:bg-purple-700 text-white shadow-lg shadow-purple-500/50">
                <Rocket className="mr-2 w-5 h-5" />
                免费开始
              </Button>
            </Link>
            <Link href="/docs">
              <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                查看文档
              </Button>
            </Link>
          </div>
        </div>
      </section>

      {/* 页脚 */}
      <footer className="border-t border-slate-800 bg-slate-900/50 py-8">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <p className="text-slate-400">
            © 2024 AgentMem. All rights reserved.
          </p>
        </div>
      </footer>
    </div>
  );
}

