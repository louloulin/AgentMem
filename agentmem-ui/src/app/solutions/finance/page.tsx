"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Brain, Banknote, ArrowLeft, Shield, TrendingUp, Users, 
  Target, AlertCircle, CheckCircle2, BarChart3, Lock,
  Activity, Zap, Database, LineChart, FileSearch
} from "lucide-react";
import Link from "next/link";
import { FadeIn, SlideIn, GradientText } from "@/components/ui/animations";

/**
 * 金融行业解决方案详情页
 */
export default function FinancePage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-amber-900 to-red-950 text-white">
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm sticky top-0 z-40">
        <div className="max-w-[1400px] mx-auto px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            {/* Logo和返回按钮区域 - 左侧固定 */}
            <div className="flex items-center space-x-8 flex-shrink-0 min-w-[320px]">
              <Link href="/" className="flex items-center">
                <Brain className="h-8 w-8 text-purple-400" />
                <span className="ml-2 text-xl font-bold text-white whitespace-nowrap">AgentMem</span>
              </Link>
              <Link href="/solutions" className="flex items-center text-slate-300 hover:text-white transition-colors text-sm whitespace-nowrap">
                <ArrowLeft className="w-4 h-4 mr-1" />
                返回解决方案
              </Link>
            </div>
            
            {/* 右侧按钮区域 */}
            <div className="flex items-center space-x-4 flex-shrink-0">
              <Link href="/demo">
                <Button variant="outline" size="sm" className="border-amber-400 text-amber-400 hover:bg-amber-400 hover:text-white">
                  预约演示
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </nav>

      {/* 英雄区域 */}
      <section className="relative overflow-hidden py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <FadeIn>
                <Badge className="mb-4 bg-amber-500/20 text-amber-400 border-amber-500/30">
                  ⭐ 企业级解决方案
                </Badge>
                <h1 className="text-5xl md:text-6xl font-bold text-white mb-6">
                  金融智能
                  <br />
                  <GradientText className="bg-gradient-to-r from-amber-400 via-yellow-400 to-orange-400">
                    风控决策平台
                  </GradientText>
                </h1>
                <p className="text-xl text-slate-300 mb-8">
                  为金融机构提供智能风控、客户画像、投资决策和合规管理的全流程AI赋能解决方案
                </p>
                <div className="flex flex-col sm:flex-row gap-4">
                <Link href="/support">
                  <Button size="lg" className="bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-700 hover:to-orange-700 text-white shadow-lg shadow-amber-500/50">
                    立即咨询
                  </Button>
                </Link>
                  <Link href="/demo">
                    <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                      查看案例
                    </Button>
                  </Link>
                </div>
              </FadeIn>
            </div>
            
            <div className="relative">
              <SlideIn direction="right">
                <div className="bg-gradient-to-br from-amber-500/20 to-orange-500/20 rounded-2xl p-8 border border-amber-500/30">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <Users className="w-8 h-8 text-amber-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">100+</div>
                      <div className="text-slate-300 text-sm">金融机构</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <Database className="w-8 h-8 text-yellow-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">1000亿+</div>
                      <div className="text-slate-300 text-sm">管理资产</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <Shield className="w-8 h-8 text-orange-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">95%+</div>
                      <div className="text-slate-300 text-sm">风险识别率</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <Lock className="w-8 h-8 text-red-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">SOC 2</div>
                      <div className="text-slate-300 text-sm">合规认证</div>
                    </div>
                  </div>
                </div>
              </SlideIn>
            </div>
          </div>
        </div>
      </section>

      {/* 行业挑战 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4">
              金融行业面临的数据挑战
            </h2>
            <p className="text-xl text-slate-300">
              数据量大、实时性要求高、监管严格，需要智能化数据管理平台
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                icon: AlertCircle,
                title: "风险难识别",
                desc: "传统规则难以覆盖复杂风险场景",
                color: "red"
              },
              {
                icon: Users,
                title: "客户画像不精准",
                desc: "数据孤岛导致客户信息不完整",
                color: "blue"
              },
              {
                icon: TrendingUp,
                title: "决策效率低",
                desc: "人工分析耗时长，错失投资机会",
                color: "yellow"
              },
              {
                icon: Lock,
                title: "合规成本高",
                desc: "监管要求严格，人工审核成本高",
                color: "purple"
              }
            ].map((item) => (
              <Card key={item.title} className="bg-slate-800/50 border-slate-700">
                <CardContent className="p-6 text-center">
                  <div className={`w-16 h-16 bg-${item.color}-500/20 rounded-full flex items-center justify-center mx-auto mb-4`}>
                    <item.icon className={`w-8 h-8 text-${item.color}-400`} />
                  </div>
                  <h3 className="text-white font-semibold mb-2">{item.title}</h3>
                  <p className="text-slate-300 text-sm">{item.desc}</p>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* 解决方案 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4">
              AgentMem 金融解决方案
            </h2>
            <p className="text-xl text-slate-300">
              四大核心模块，全面提升金融业务智能化水平
            </p>
          </div>

          <div className="space-y-12">
            {/* 1. 智能风控 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <div>
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-red-500/20 rounded-lg flex items-center justify-center mr-4">
                    <Shield className="w-6 h-6 text-red-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">智能风控系统</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  基于海量历史交易数据和用户行为数据，构建多维度风险评估模型，实时识别欺诈和信用风险
                </p>
                <ul className="space-y-3">
                  {[
                    "自动提取用户交易特征和行为模式",
                    "多维度风险评分，识别率95%+",
                    "实时风险监控和预警，&lt;100ms响应",
                    "自适应风控策略，持续优化模型",
                  ].map((item) => (
                    <li key={item} className="flex items-start">
                      <CheckCircle2 className="w-5 h-5 text-green-400 mr-2 mt-0.5 flex-shrink-0" />
                      <span className="text-slate-300">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
              <Card className="bg-gradient-to-br from-red-500/10 to-orange-500/10 border-red-500/30">
                <CardContent className="p-6">
                  <h4 className="text-white font-semibold mb-4">风控效果对比</h4>
                  <div className="space-y-4">
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-slate-400">欺诈识别率</span>
                        <span className="text-green-400">从75% → 96%</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="flex-1 bg-slate-700 rounded-full h-2">
                          <div className="bg-slate-500 h-2 rounded-full" style={{ width: "75%" }}></div>
                        </div>
                        <ArrowLeft className="rotate-180 w-4 h-4 text-green-400" />
                        <div className="flex-1 bg-slate-700 rounded-full h-2">
                          <div className="bg-green-500 h-2 rounded-full" style={{ width: "96%" }}></div>
                        </div>
                      </div>
                    </div>
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-slate-400">误报率</span>
                        <span className="text-green-400">从15% → 3%</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <div className="flex-1 bg-slate-700 rounded-full h-2">
                          <div className="bg-red-500 h-2 rounded-full" style={{ width: "15%" }}></div>
                        </div>
                        <ArrowLeft className="rotate-180 w-4 h-4 text-green-400" />
                        <div className="flex-1 bg-slate-700 rounded-full h-2">
                          <div className="bg-green-500 h-2 rounded-full" style={{ width: "3%" }}></div>
                        </div>
                      </div>
                    </div>
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-slate-400">响应时间</span>
                        <span className="text-green-400">从2s → 80ms</span>
                      </div>
                      <div className="bg-slate-900/50 rounded-lg p-3 text-center">
                        <div className="text-3xl font-bold text-green-400">25x</div>
                        <div className="text-slate-400 text-xs">性能提升</div>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* 2. 客户画像 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <Card className="bg-gradient-to-br from-blue-500/10 to-cyan-500/10 border-blue-500/30 lg:order-1">
                <CardContent className="p-6">
                  <h4 className="text-white font-semibold mb-4">360°客户视图</h4>
                  <div className="space-y-3">
                    {[
                      { category: "基础信息", items: "姓名、年龄、职业、收入", icon: Users, color: "blue" },
                      { category: "交易行为", items: "交易频率、金额、偏好", icon: Activity, color: "green" },
                      { category: "风险偏好", items: "投资风格、风险承受能力", icon: Target, color: "purple" },
                      { category: "生命周期", items: "客户阶段、价值预测", icon: TrendingUp, color: "yellow" },
                      { category: "社交关系", items: "社交网络、影响力评估", icon: Users, color: "pink" },
                    ].map((item) => (
                      <div key={item.category} className="bg-slate-900/50 rounded-lg p-3">
                        <div className="flex items-center mb-1">
                          <item.icon className={`w-4 h-4 text-${item.color}-400 mr-2`} />
                          <span className="text-white font-medium text-sm">{item.category}</span>
                        </div>
                        <div className="text-slate-400 text-xs ml-6">{item.items}</div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
              <div className="lg:order-2">
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center mr-4">
                    <Users className="w-6 h-6 text-blue-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">智能客户画像</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  整合多渠道客户数据，构建360°全景客户画像，支持精准营销和个性化服务
                </p>
                <ul className="space-y-3">
                  {[
                    "自动整合银行、证券、保险等多渠道数据",
                    "AI自动标签体系，1000+维度特征",
                    "实时更新客户画像，保持数据新鲜度",
                    "客户分群和价值预测，提升转化率30%+",
                  ].map((item) => (
                    <li key={item} className="flex items-start">
                      <CheckCircle2 className="w-5 h-5 text-green-400 mr-2 mt-0.5 flex-shrink-0" />
                      <span className="text-slate-300">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>

            {/* 3. 投资决策 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <div>
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-yellow-500/20 rounded-lg flex items-center justify-center mr-4">
                    <TrendingUp className="w-6 h-6 text-yellow-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">智能投资决策</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  基于历史市场数据和投资组合表现，提供AI驱动的投资建议和风险管理策略
                </p>
                <ul className="space-y-3">
                  {[
                    "多因子量化模型，捕捉市场机会",
                    "投资组合优化，风险收益平衡",
                    "实时市场监控和投资机会预警",
                    "投资绩效归因分析和策略优化",
                  ].map((item) => (
                    <li key={item} className="flex items-start">
                      <CheckCircle2 className="w-5 h-5 text-green-400 mr-2 mt-0.5 flex-shrink-0" />
                      <span className="text-slate-300">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
              <Card className="bg-gradient-to-br from-yellow-500/10 to-orange-500/10 border-yellow-500/30">
                <CardContent className="p-6">
                  <h4 className="text-white font-semibold mb-4 flex items-center">
                    <LineChart className="w-5 h-5 mr-2 text-yellow-400" />
                    投资决策支持
                  </h4>
                  <div className="space-y-4">
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-slate-400 text-sm">策略回测准确率</span>
                        <Badge className="bg-green-500/20 text-green-400">优秀</Badge>
                      </div>
                      <div className="text-2xl font-bold text-white mb-1">87%</div>
                      <div className="text-slate-400 text-xs">历史3年数据验证</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-slate-400 text-sm">年化收益率提升</span>
                        <Badge className="bg-blue-500/20 text-blue-400">显著</Badge>
                      </div>
                      <div className="text-2xl font-bold text-white mb-1">+8.5%</div>
                      <div className="text-slate-400 text-xs">相比基准指数</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-slate-400 text-sm">决策效率提升</span>
                        <Badge className="bg-purple-500/20 text-purple-400">高效</Badge>
                      </div>
                      <div className="text-2xl font-bold text-white mb-1">50%↑</div>
                      <div className="text-slate-400 text-xs">从数小时到分钟级</div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* 4. 合规管理 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <Card className="bg-gradient-to-br from-purple-500/10 to-pink-500/10 border-purple-500/30 lg:order-1">
                <CardContent className="p-6">
                  <h4 className="text-white font-semibold mb-4 flex items-center">
                    <FileSearch className="w-5 h-5 mr-2 text-purple-400" />
                    合规检查项
                  </h4>
                  <div className="space-y-3">
                    {[
                      { item: "反洗钱(AML)监控", coverage: "100%", status: "通过" },
                      { item: "KYC客户尽职调查", coverage: "100%", status: "通过" },
                      { item: "交易合规性审查", coverage: "100%", status: "通过" },
                      { item: "信息披露合规", coverage: "100%", status: "通过" },
                      { item: "数据隐私保护", coverage: "100%", status: "通过" },
                    ].map((check) => (
                      <div key={check.item} className="bg-slate-900/50 rounded-lg p-3 flex items-center justify-between">
                        <div className="flex items-center">
                          <CheckCircle2 className="w-4 h-4 text-green-400 mr-2" />
                          <span className="text-white text-sm">{check.item}</span>
                        </div>
                        <Badge className="bg-green-500/20 text-green-400 text-xs">
                          {check.status}
                        </Badge>
                      </div>
                    ))}
                  </div>
                  <div className="mt-4 bg-slate-900/50 rounded-lg p-3 text-center">
                    <Lock className="w-6 h-6 text-purple-400 mx-auto mb-2" />
                    <div className="text-white font-semibold">SOC 2 Type II 认证</div>
                    <div className="text-slate-400 text-xs">企业级安全合规保障</div>
                  </div>
                </CardContent>
              </Card>
              <div className="lg:order-2">
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center mr-4">
                    <Lock className="w-6 h-6 text-purple-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">智能合规管理</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  自动化合规检查和审计，确保业务操作符合监管要求，降低合规成本60%+
                </p>
                <ul className="space-y-3">
                  {[
                    "自动识别可疑交易，实时合规预警",
                    "智能KYC审核，提升客户onboarding效率",
                    "完整审计日志，支持监管报告自动生成",
                    "数据加密存储，符合GDPR、等保2.0要求",
                  ].map((item) => (
                    <li key={item} className="flex items-start">
                      <CheckCircle2 className="w-5 h-5 text-green-400 mr-2 mt-0.5 flex-shrink-0" />
                      <span className="text-slate-300">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* 核心优势 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4">
              为什么选择 AgentMem
            </h2>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                icon: Zap,
                title: "实时处理",
                desc: "毫秒级响应，支持百万级并发",
                metric: "&lt;100ms",
                color: "amber"
              },
              {
                icon: Shield,
                title: "金融级安全",
                desc: "端到端加密，SOC 2认证",
                metric: "99.99%",
                color: "orange"
              },
              {
                icon: Database,
                title: "海量数据",
                desc: "PB级数据处理能力",
                metric: "10亿+条",
                color: "yellow"
              },
              {
                icon: BarChart3,
                title: "精准决策",
                desc: "AI驱动，准确率95%+",
                metric: "95%+",
                color: "red"
              }
            ].map((item) => (
              <Card key={item.title} className="bg-slate-800/50 border-slate-700 hover:border-amber-500/50 transition-colors">
                <CardContent className="p-6 text-center">
                  <div className={`w-16 h-16 bg-${item.color}-500/20 rounded-full flex items-center justify-center mx-auto mb-4`}>
                    <item.icon className={`w-8 h-8 text-${item.color}-400`} />
                  </div>
                  <h3 className="text-white font-semibold mb-2">{item.title}</h3>
                  <p className="text-slate-300 text-sm mb-3">{item.desc}</p>
                  <div className={`text-2xl font-bold text-${item.color}-400`}>{item.metric}</div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* 客户案例 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4">
              客户成功案例
            </h2>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                type: "商业银行",
                scenario: "零售信贷风控",
                result: "坏账率降低40%，审批效率提升60%",
                metrics: { customers: "500万+", loans: "1000亿", npl: "40%↓" }
              },
              {
                type: "证券公司",
                scenario: "智能投顾平台",
                result: "客户AUM增长80%，服务成本降低50%",
                metrics: { users: "10万+", aum: "500亿", retention: "85%" }
              },
              {
                type: "保险公司",
                scenario: "理赔反欺诈",
                result: "欺诈识别率96%，理赔周期缩短70%",
                metrics: { claims: "100万/年", fraud: "96%", time: "3天→1天" }
              }
            ].map((caseItem, index) => (
              <Card key={index} className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <Badge className="mb-2 bg-amber-500/20 text-amber-400 w-fit">
                    <Banknote className="w-3 h-3 mr-1" />
                    {caseItem.type}
                  </Badge>
                  <CardTitle className="text-white text-lg">{caseItem.scenario}</CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-slate-300 mb-4 text-sm">{caseItem.result}</p>
                  <div className="space-y-2">
                    {Object.entries(caseItem.metrics).map(([key, value]) => (
                      <div key={key} className="flex justify-between text-sm">
                        <span className="text-slate-400">
                          {key === 'customers' ? '服务客户' :
                           key === 'loans' ? '贷款规模' :
                           key === 'npl' ? '不良率降低' :
                           key === 'users' ? '用户数' :
                           key === 'aum' ? '管理资产' :
                           key === 'retention' ? '留存率' :
                           key === 'claims' ? '理赔量' :
                           key === 'fraud' ? '识别率' :
                           key === 'time' ? '处理时长' : key}
                        </span>
                        <span className="text-white font-semibold">{value}</span>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-r from-amber-900/30 via-orange-900/30 to-red-900/30">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl font-bold text-white mb-6">
            开启金融智能化转型
          </h2>
          <p className="text-xl text-slate-300 mb-8">
            立即体验 AgentMem 金融解决方案，提升业务效率50%+
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link href="/support">
              <Button size="lg" className="bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-700 hover:to-orange-700 text-white shadow-lg shadow-amber-500/50">
                预约专家咨询
              </Button>
            </Link>
            <Link href="/demo">
              <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                申请试用
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

