"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { FadeIn, SlideIn } from "@/components/ui/animations";
import { ChevronDown, ChevronUp, HelpCircle, MessageCircle, Book, Zap } from "lucide-react";
import Link from "next/link";

// Disable static generation for this page
export const dynamic = 'force-dynamic';

interface FAQItem {
  id: string;
  question: string;
  answer: string;
  category: string;
}

const faqData: FAQItem[] = [
  {
    id: "1",
    question: "AgentMem 与其他记忆管理解决方案有什么区别？",
    answer: "AgentMem 基于 Rust 构建，提供更高的性能和安全性。集成 DeepSeek 推理引擎，支持智能事实提取和冲突解决。100% Mem0 API 兼容，支持无缝迁移现有应用。",
    category: "产品特性"
  },
  {
    id: "2",
    question: "如何开始使用 AgentMem？",
    answer: "您可以通过 Cargo 安装 AgentMem，或使用我们的 Docker 镜像快速部署。我们提供详细的文档和示例代码，帮助您快速集成到现有项目中。",
    category: "快速开始"
  },
  {
    id: "3",
    question: "AgentMem 支持哪些存储后端？",
    answer: "AgentMem 支持多种存储后端，包括向量数据库（Pinecone、Qdrant、Chroma）、关系数据库（PostgreSQL）、缓存数据库（Redis）和图数据库（Neo4j、Memgraph）等。",
    category: "技术架构"
  },
  {
    id: "4",
    question: "AgentMem 的性能如何？",
    answer: "基于 Rust 和 Tokio 异步运行时，AgentMem 提供毫秒级响应时间。支持高并发处理，内置多级缓存系统和批量处理优化，能够满足大规模生产环境需求。",
    category: "性能"
  },
  {
    id: "5",
    question: "是否支持企业级部署？",
    answer: "是的，AgentMem 提供企业级特性，包括分布式部署、监控遥测、完整的测试覆盖、类型安全保证等。支持私有化部署和云端 SaaS 服务。",
    category: "企业服务"
  },
  {
    id: "6",
    question: "如何从 Mem0 迁移到 AgentMem？",
    answer: "AgentMem 100% 兼容 Mem0 API，迁移过程无需修改代码。只需更换依赖包和配置文件，即可享受更高的性能和更丰富的功能。",
    category: "迁移指南"
  },
  {
    id: "7",
    question: "AgentMem 的定价模式是什么？",
    answer: "AgentMem 核心库开源免费。我们提供多种商业服务：云端 SaaS 服务按使用量计费，企业版提供专业支持和定制开发，还有培训和咨询服务。",
    category: "定价"
  },
  {
    id: "8",
    question: "如何获得技术支持？",
    answer: "我们提供多种支持渠道：GitHub Issues、官方文档、社区论坛、企业客户专享的技术支持服务。还有定期的在线研讨会和技术分享。",
    category: "技术支持"
  }
];

const categories = ["全部", "产品特性", "快速开始", "技术架构", "性能", "企业服务", "迁移指南", "定价", "技术支持"];

export default function FAQPage() {
  const [selectedCategory, setSelectedCategory] = useState("全部");
  const [expandedItems, setExpandedItems] = useState<string[]>([]);

  const filteredFAQs = selectedCategory === "全部" 
    ? faqData 
    : faqData.filter(item => item.category === selectedCategory);

  const toggleExpanded = (id: string) => {
    setExpandedItems(prev => 
      prev.includes(id) 
        ? prev.filter(item => item !== id)
        : [...prev, id]
    );
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <Link href="/" className="flex items-center">
                <HelpCircle className="h-8 w-8 text-purple-400" />
                <span className="ml-2 text-xl font-bold text-white">AgentMem FAQ</span>
              </Link>
            </div>
            <div className="flex items-center space-x-4">
              <Link href="/" className="text-slate-300 hover:text-white transition-colors">
                首页
              </Link>
              <Link href="/docs" className="text-slate-300 hover:text-white transition-colors">
                文档
              </Link>
              <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                联系支持
              </Button>
            </div>
          </div>
        </div>
      </nav>

      {/* 页面内容 */}
      <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        {/* 页面标题 */}
        <FadeIn>
          <div className="text-center mb-12">
            <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">
              常见问题
            </h1>
            <p className="text-xl text-slate-300 max-w-3xl mx-auto">
              找到关于 AgentMem 的常见问题解答，如果您有其他问题，请随时联系我们的支持团队。
            </p>
          </div>
        </FadeIn>

        {/* 分类筛选 */}
        <SlideIn direction="up" delay={200}>
          <div className="flex flex-wrap gap-2 justify-center mb-8">
            {categories.map((category) => (
              <Button
                key={category}
                variant={selectedCategory === category ? "default" : "outline"}
                size="sm"
                onClick={() => setSelectedCategory(category)}
                className={`transition-all duration-300 ${
                  selectedCategory === category
                    ? "bg-purple-600 text-white"
                    : "border-slate-600 text-slate-300 hover:bg-slate-800"
                }`}
              >
                {category}
              </Button>
            ))}
          </div>
        </SlideIn>

        {/* FAQ 列表 */}
        <div className="space-y-4">
          {filteredFAQs.map((faq, index) => (
            <SlideIn key={faq.id} direction="up" delay={300 + index * 100}>
              <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
                <CardHeader 
                  className="cursor-pointer" 
                  onClick={() => toggleExpanded(faq.id)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <Badge variant="outline" className="border-purple-400 text-purple-400">
                        {faq.category}
                      </Badge>
                      <CardTitle className="text-white text-lg">
                        {faq.question}
                      </CardTitle>
                    </div>
                    {expandedItems.includes(faq.id) ? (
                      <ChevronUp className="h-5 w-5 text-slate-400" />
                    ) : (
                      <ChevronDown className="h-5 w-5 text-slate-400" />
                    )}
                  </div>
                </CardHeader>
                {expandedItems.includes(faq.id) && (
                  <CardContent>
                    <p className="text-slate-300 leading-relaxed">
                      {faq.answer}
                    </p>
                  </CardContent>
                )}
              </Card>
            </SlideIn>
          ))}
        </div>

        {/* 联系支持 */}
        <SlideIn direction="up" delay={800}>
          <div className="mt-16 text-center">
            <Card className="bg-gradient-to-r from-purple-900/50 to-pink-900/50 border-purple-500/30">
              <CardHeader>
                <CardTitle className="text-white text-2xl mb-2">
                  没有找到您要的答案？
                </CardTitle>
                <CardDescription className="text-slate-300 text-lg">
                  我们的技术支持团队随时为您提供帮助
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex flex-col sm:flex-row gap-4 justify-center">
                  <Button className="bg-purple-600 hover:bg-purple-700 text-white">
                    <MessageCircle className="mr-2 h-4 w-4" />
                    联系支持
                  </Button>
                  <Button variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                    <Book className="mr-2 h-4 w-4" />
                    查看文档
                  </Button>
                  <Button variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                    <Zap className="mr-2 h-4 w-4" />
                    加入社区
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>
        </SlideIn>
      </div>
    </div>
  );
}