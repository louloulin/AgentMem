"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Brain, Plane, Banknote, Building2, Hospital, GraduationCap, Factory, ArrowRight, CheckCircle, TrendingUp, Shield, Zap, Database, Network } from "lucide-react";
import Link from "next/link";
import { FadeIn, SlideIn, GradientText } from "@/components/ui/animations";

/**
 * 行业解决方案主页
 */
export default function SolutionsPage() {
  const solutions = [
    {
      id: "low-altitude-economy",
      title: "低空经济",
      icon: Plane,
      color: "blue",
      description: "为无人机、eVTOL等低空飞行器提供智能数据管理和调度解决方案",
      features: ["飞行数据管理", "智能调度系统", "空域优化", "安全监控"],
      benefits: ["提升运营效率40%+", "降低安全事故60%+", "优化空域利用50%+"],
      href: "/solutions/low-altitude-economy",
      badge: "🔥 热门",
      stats: { users: "50+", coverage: "全国", uptime: "99.99%" }
    },
    {
      id: "finance",
      title: "金融行业",
      icon: Banknote,
      color: "green",
      description: "为金融机构提供智能风控、客户画像和投资决策支持系统",
      features: ["智能风控", "客户画像", "投资决策", "合规管理"],
      benefits: ["风险识别率95%+", "客户转化率提升30%+", "决策效率提高50%+"],
      href: "/solutions/finance",
      badge: "⭐ 企业级",
      stats: { users: "100+", aum: "1000亿+", compliance: "SOC 2" }
    },
    {
      id: "healthcare",
      title: "医疗健康",
      icon: Hospital,
      color: "red",
      description: "为医疗机构提供智能诊断辅助和患者记忆管理系统",
      features: ["病历管理", "诊断辅助", "用药提醒", "健康追踪"],
      benefits: ["诊断准确率提升40%+", "医生效率提升60%+", "患者满意度95%+"],
      href: "/solutions/healthcare",
      badge: "即将推出",
      stats: { patients: "10万+", hospitals: "50+", accuracy: "98%" }
    },
    {
      id: "education",
      title: "教育科技",
      icon: GraduationCap,
      color: "purple",
      description: "为教育机构提供个性化学习和知识图谱构建系统",
      features: ["学习路径", "知识图谱", "智能推荐", "效果评估"],
      benefits: ["学习效率提升50%+", "知识留存率80%+", "个性化精准度95%+"],
      href: "/solutions/education",
      badge: "即将推出",
      stats: { students: "50万+", courses: "1000+", satisfaction: "4.9/5" }
    },
    {
      id: "manufacturing",
      title: "智能制造",
      icon: Factory,
      color: "orange",
      description: "为制造企业提供设备记忆和预测性维护解决方案",
      features: ["设备监控", "预测维护", "工艺优化", "质量追溯"],
      benefits: ["设备故障率降低70%+", "维护成本降低40%+", "产能提升30%+"],
      href: "/solutions/manufacturing",
      badge: "即将推出",
      stats: { factories: "200+", devices: "10万+", savings: "5亿+" }
    },
    {
      id: "enterprise",
      title: "企业服务",
      icon: Building2,
      color: "indigo",
      description: "为企业提供知识管理和智能协作平台",
      features: ["知识库", "智能搜索", "协作工作流", "决策支持"],
      benefits: ["知识复用率90%+", "协作效率提升40%+", "决策质量提升50%+"],
      href: "/solutions/enterprise",
      badge: "即将推出",
      stats: { companies: "500+", users: "10万+", docs: "1000万+" }
    },
  ];

  const getIconColor = (color: string) => {
    const colors: Record<string, string> = {
      blue: "text-blue-400 bg-blue-500/20",
      green: "text-green-400 bg-green-500/20",
      red: "text-red-400 bg-red-500/20",
      purple: "text-purple-400 bg-purple-500/20",
      orange: "text-orange-400 bg-orange-500/20",
      indigo: "text-indigo-400 bg-indigo-500/20",
    };
    return colors[color] || colors.blue;
  };

  const getBadgeColor = (badge: string) => {
    if (badge.includes("热门")) return "bg-red-500/20 text-red-400 border-red-500/30";
    if (badge.includes("企业级")) return "bg-purple-500/20 text-purple-400 border-purple-500/30";
    return "bg-slate-500/20 text-slate-400 border-slate-500/30";
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900 text-white">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-[1400px] mx-auto px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            {/* Logo区域 - 左侧固定 */}
            <div className="flex items-center flex-shrink-0 min-w-[180px]">
              <Link href="/" className="flex items-center">
                <Brain className="h-8 w-8 text-purple-400 animate-pulse" />
                <span className="ml-2 text-xl font-bold text-white">AgentMem</span>
              </Link>
            </div>
            
            {/* 右侧按钮区域 */}
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
              <h1 className="text-5xl md:text-6xl font-bold text-white mb-6">
                <GradientText>行业解决方案</GradientText>
              </h1>
              <p className="text-xl text-slate-300 mb-12 max-w-3xl mx-auto">
                基于 AgentMem 智能记忆平台，为各行各业提供定制化的数据赋能解决方案
              </p>
            </FadeIn>

            {/* 核心价值 */}
            <SlideIn direction="up" delay={300}>
              <div className="grid grid-cols-1 md:grid-cols-4 gap-6 max-w-5xl mx-auto mb-16">
                <div className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
                  <Database className="w-10 h-10 text-blue-400 mx-auto mb-3" />
                  <div className="text-2xl font-bold text-white mb-1">千万级</div>
                  <div className="text-slate-300 text-sm">数据处理能力</div>
                </div>
                <div className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
                  <Zap className="w-10 h-10 text-yellow-400 mx-auto mb-3" />
                  <div className="text-2xl font-bold text-white mb-1">&lt;100ms</div>
                  <div className="text-slate-300 text-sm">实时响应速度</div>
                </div>
                <div className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
                  <Shield className="w-10 h-10 text-green-400 mx-auto mb-3" />
                  <div className="text-2xl font-bold text-white mb-1">99.99%</div>
                  <div className="text-slate-300 text-sm">服务可用性</div>
                </div>
                <div className="bg-slate-800/50 rounded-lg p-6 border border-slate-700">
                  <TrendingUp className="w-10 h-10 text-purple-400 mx-auto mb-3" />
                  <div className="text-2xl font-bold text-white mb-1">40%+</div>
                  <div className="text-slate-300 text-sm">效率提升</div>
                </div>
              </div>
            </SlideIn>
          </div>
        </div>

        {/* 背景装饰 */}
        <div className="absolute inset-0 overflow-hidden pointer-events-none">
          <div className="absolute -top-40 -right-40 w-80 h-80 bg-purple-500/20 rounded-full blur-3xl"></div>
          <div className="absolute -bottom-40 -left-40 w-80 h-80 bg-blue-500/20 rounded-full blur-3xl"></div>
        </div>
      </section>

      {/* 解决方案列表 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {solutions.map((solution, index) => {
              const Icon = solution.icon;
              return (
                <SlideIn key={solution.id} direction="up" delay={index * 100}>
                  <Link href={solution.href}>
                    <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 h-full group cursor-pointer hover:scale-105">
                      <CardHeader>
                        <div className="flex items-start justify-between mb-4">
                          <div className={`p-3 rounded-lg ${getIconColor(solution.color)} group-hover:scale-110 transition-transform`}>
                            <Icon className="h-8 w-8" />
                          </div>
                          <Badge className={`${getBadgeColor(solution.badge)} border`}>
                            {solution.badge}
                          </Badge>
                        </div>
                        <CardTitle className="text-white text-2xl group-hover:text-purple-400 transition-colors">
                          {solution.title}
                        </CardTitle>
                        <CardDescription className="text-slate-300">
                          {solution.description}
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        {/* 核心功能 */}
                        <div className="mb-4">
                          <h4 className="text-white font-semibold mb-2 text-sm">核心功能</h4>
                          <div className="flex flex-wrap gap-2">
                            {solution.features.map((feature) => (
                              <span key={feature} className="text-xs bg-slate-700/50 text-slate-300 px-2 py-1 rounded">
                                {feature}
                              </span>
                            ))}
                          </div>
                        </div>

                        {/* 业务价值 */}
                        <div className="mb-4">
                          <h4 className="text-white font-semibold mb-2 text-sm">业务价值</h4>
                          <ul className="space-y-1">
                            {solution.benefits.map((benefit) => (
                              <li key={benefit} className="text-slate-300 text-sm flex items-start">
                                <CheckCircle className="w-4 h-4 text-green-400 mr-1 mt-0.5 flex-shrink-0" />
                                <span>{benefit}</span>
                              </li>
                            ))}
                          </ul>
                        </div>

                        {/* 查看详情 */}
                        <div className="flex items-center text-purple-400 group-hover:text-purple-300 transition-colors">
                          <span className="text-sm font-medium">查看详情</span>
                          <ArrowRight className="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" />
                        </div>
                      </CardContent>
                    </Card>
                  </Link>
                </SlideIn>
              );
            })}
          </div>
        </div>
      </section>

      {/* CTA 区域 */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-r from-purple-900/30 to-blue-900/30">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl font-bold text-white mb-6">
            找不到适合您的解决方案？
          </h2>
          <p className="text-xl text-slate-300 mb-8">
            我们提供定制化服务，根据您的业务需求量身打造专属解决方案
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link href="/support">
              <Button size="lg" className="bg-purple-600 hover:bg-purple-700 text-white">
                联系我们
              </Button>
            </Link>
            <Link href="/demo">
              <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                预约演示
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

