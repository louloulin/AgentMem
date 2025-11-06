"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Brain, Plane, ArrowLeft, TrendingUp, Shield, Zap, Database, 
  Network, Cloud, MapPin, BarChart3, Radio, AlertTriangle,
  Clock, Users, Target, CheckCircle2, LineChart
} from "lucide-react";
import Link from "next/link";
import { FadeIn, SlideIn, GradientText } from "@/components/ui/animations";

/**
 * 低空经济解决方案详情页
 */
export default function LowAltitudeEconomyPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 text-white">
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
                <Button variant="outline" size="sm" className="border-blue-400 text-blue-400 hover:bg-blue-400 hover:text-white">
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
                <Badge className="mb-4 bg-blue-500/20 text-blue-400 border-blue-500/30">
                  🔥 万亿级市场机遇
                </Badge>
                <h1 className="text-5xl md:text-6xl font-bold text-white mb-6">
                  低空经济
                  <br />
                  <GradientText className="bg-gradient-to-r from-blue-400 to-cyan-400">
                    智能数据平台
                  </GradientText>
                </h1>
                <p className="text-xl text-slate-300 mb-8">
                  为无人机、eVTOL等低空飞行器提供全生命周期的智能数据管理、调度优化和安全监控解决方案
                </p>
                <div className="flex flex-col sm:flex-row gap-4">
                  <Link href="/support">
                    <Button size="lg" className="bg-blue-600 hover:bg-blue-700 text-white">
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
                <div className="bg-gradient-to-br from-blue-500/20 to-cyan-500/20 rounded-2xl p-8 border border-blue-500/30">
                  <div className="grid grid-cols-2 gap-4">
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <Plane className="w-8 h-8 text-blue-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">10万+</div>
                      <div className="text-slate-300 text-sm">日飞行架次</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <MapPin className="w-8 h-8 text-cyan-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">全国</div>
                      <div className="text-slate-300 text-sm">空域覆盖</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <Clock className="w-8 h-8 text-green-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">&lt;1s</div>
                      <div className="text-slate-300 text-sm">调度响应</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4 text-center">
                      <Shield className="w-8 h-8 text-purple-400 mx-auto mb-2" />
                      <div className="text-2xl font-bold text-white">99.99%</div>
                      <div className="text-slate-300 text-sm">安全率</div>
                    </div>
                  </div>
                </div>
              </SlideIn>
            </div>
          </div>
        </div>
      </section>

      {/* 行业背景 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4">
              低空经济：新质生产力的代表
            </h2>
            <p className="text-xl text-slate-300 max-w-3xl mx-auto">
              2025年中国低空经济规模预计突破1.5万亿元，到2030年将达到3万亿元
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8 mb-12">
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <TrendingUp className="w-12 h-12 text-blue-400 mb-4" />
                <CardTitle className="text-white">市场规模</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-slate-300">
                  预计2030年形成万亿级市场，年复合增长率超过25%，成为经济新增长点
                </p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <Network className="w-12 h-12 text-cyan-400 mb-4" />
                <CardTitle className="text-white">应用场景</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-slate-300">
                  覆盖物流配送、农业植保、巡检维护、应急救援、城市管理等多个领域
                </p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <Database className="w-12 h-12 text-green-400 mb-4" />
                <CardTitle className="text-white">数据挑战</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-slate-300">
                  海量飞行数据、复杂空域管理、实时调度需求，亟需智能化数据管理平台
                </p>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* AgentMem 如何赋能 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4">
              AgentMem 如何为低空经济赋能
            </h2>
            <p className="text-xl text-slate-300">
              基于智能记忆平台的四大核心能力，全面提升低空经济运营效率
            </p>
          </div>

          <div className="space-y-12">
            {/* 1. 飞行数据智能管理 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <div>
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center mr-4">
                    <Database className="w-6 h-6 text-blue-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">飞行数据智能管理</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  整合飞行器实时状态、飞行轨迹、气象数据、空域信息等多源异构数据，构建统一的低空数据底座
                </p>
                <ul className="space-y-3">
                  {[
                    "存储千万级飞行记录，支持毫秒级查询",
                    "自动提取飞行模式、异常事件等关键信息",
                    "建立飞行器-空域-气象的知识图谱",
                    "支持历史数据回溯和趋势分析"
                  ].map((item) => (
                    <li key={item} className="flex items-start">
                      <CheckCircle2 className="w-5 h-5 text-green-400 mr-2 mt-0.5 flex-shrink-0" />
                      <span className="text-slate-300">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
              <Card className="bg-gradient-to-br from-blue-500/10 to-cyan-500/10 border-blue-500/30">
                <CardContent className="p-6">
                  <div className="space-y-4">
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-slate-400">数据存储容量</span>
                        <Badge className="bg-blue-500/20 text-blue-400">实时</Badge>
                      </div>
                      <div className="text-2xl font-bold text-white">1000万+ 条/日</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-slate-400">查询响应时间</span>
                        <Badge className="bg-green-500/20 text-green-400">优化</Badge>
                      </div>
                      <div className="text-2xl font-bold text-white">&lt; 50ms</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-slate-400">数据准确率</span>
                        <Badge className="bg-purple-500/20 text-purple-400">保障</Badge>
                      </div>
                      <div className="text-2xl font-bold text-white">99.9%</div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* 2. 智能调度系统 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <Card className="bg-gradient-to-br from-cyan-500/10 to-green-500/10 border-cyan-500/30 lg:order-1">
                <CardContent className="p-6">
                  <h4 className="text-white font-semibold mb-4">智能调度流程</h4>
                  <div className="space-y-3">
                    {[
                      { step: "1", title: "飞行申请", time: "实时接收", color: "blue" },
                      { step: "2", title: "路径规划", time: "&lt;500ms", color: "cyan" },
                      { step: "3", title: "冲突检测", time: "&lt;100ms", color: "green" },
                      { step: "4", title: "资源分配", time: "&lt;200ms", color: "purple" },
                      { step: "5", title: "任务下发", time: "即时推送", color: "pink" },
                    ].map((item) => (
                      <div key={item.step} className="flex items-center bg-slate-900/50 rounded-lg p-3">
                        <div className={`w-8 h-8 bg-${item.color}-500/20 rounded-full flex items-center justify-center mr-3 flex-shrink-0`}>
                          <span className="text-white font-bold text-sm">{item.step}</span>
                        </div>
                        <div className="flex-1">
                          <div className="text-white font-medium">{item.title}</div>
                          <div className="text-slate-400 text-sm">{item.time}</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </CardContent>
              </Card>
              <div className="lg:order-2">
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-cyan-500/20 rounded-lg flex items-center justify-center mr-4">
                    <Network className="w-6 h-6 text-cyan-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">智能调度系统</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  基于历史飞行数据和实时状态，AI驱动的智能调度引擎实现自动化路径规划和资源优化
                </p>
                <ul className="space-y-3">
                  {[
                    "自动规划最优飞行路径，考虑天气、空域、能耗等因素",
                    "实时冲突检测和避让策略生成",
                    "动态资源调配，提升空域利用率50%+",
                    "多机协同调度，支持编队飞行",
                  ].map((item) => (
                    <li key={item} className="flex items-start">
                      <CheckCircle2 className="w-5 h-5 text-green-400 mr-2 mt-0.5 flex-shrink-0" />
                      <span className="text-slate-300">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            </div>

            {/* 3. 安全监控预警 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <div>
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-red-500/20 rounded-lg flex items-center justify-center mr-4">
                    <AlertTriangle className="w-6 h-6 text-red-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">安全监控预警</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  7×24小时实时监控飞行器状态，基于机器学习的异常检测和预警系统，确保飞行安全
                </p>
                <ul className="space-y-3">
                  {[
                    "实时监控飞行器位置、高度、速度、电量等关键参数",
                    "AI异常检测，识别95%+的潜在风险",
                    "多级预警机制，提前3-5分钟告警",
                    "应急响应方案自动生成和执行",
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
                  <h4 className="text-white font-semibold mb-4 flex items-center">
                    <Radio className="w-5 h-5 mr-2 text-red-400" />
                    实时监控指标
                  </h4>
                  <div className="space-y-4">
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-slate-400">飞行器在线率</span>
                        <span className="text-green-400">99.8%</span>
                      </div>
                      <div className="w-full bg-slate-700 rounded-full h-2">
                        <div className="bg-green-500 h-2 rounded-full" style={{ width: "99.8%" }}></div>
                      </div>
                    </div>
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-slate-400">异常检测准确率</span>
                        <span className="text-blue-400">96.5%</span>
                      </div>
                      <div className="w-full bg-slate-700 rounded-full h-2">
                        <div className="bg-blue-500 h-2 rounded-full" style={{ width: "96.5%" }}></div>
                      </div>
                    </div>
                    <div>
                      <div className="flex justify-between text-sm mb-2">
                        <span className="text-slate-400">预警响应及时率</span>
                        <span className="text-purple-400">98.2%</span>
                      </div>
                      <div className="w-full bg-slate-700 rounded-full h-2">
                        <div className="bg-purple-500 h-2 rounded-full" style={{ width: "98.2%" }}></div>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* 4. 运营决策支持 */}
            <div className="grid lg:grid-cols-2 gap-8 items-center">
              <Card className="bg-gradient-to-br from-purple-500/10 to-pink-500/10 border-purple-500/30 lg:order-1">
                <CardContent className="p-6">
                  <h4 className="text-white font-semibold mb-4 flex items-center">
                    <BarChart3 className="w-5 h-5 mr-2 text-purple-400" />
                    数据洞察能力
                  </h4>
                  <div className="space-y-4">
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <LineChart className="w-8 h-8 text-blue-400 mb-2" />
                      <div className="text-white font-medium mb-1">飞行效率分析</div>
                      <div className="text-slate-400 text-sm">识别低效航线，优化飞行计划</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <Target className="w-8 h-8 text-green-400 mb-2" />
                      <div className="text-white font-medium mb-1">需求预测</div>
                      <div className="text-slate-400 text-sm">预测未来7天飞行需求，提前资源调配</div>
                    </div>
                    <div className="bg-slate-900/50 rounded-lg p-4">
                      <TrendingUp className="w-8 h-8 text-purple-400 mb-2" />
                      <div className="text-white font-medium mb-1">成本优化</div>
                      <div className="text-slate-400 text-sm">识别成本优化机会，降低运营成本20%+</div>
                    </div>
                  </div>
                </CardContent>
              </Card>
              <div className="lg:order-2">
                <div className="flex items-center mb-4">
                  <div className="w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center mr-4">
                    <BarChart3 className="w-6 h-6 text-purple-400" />
                  </div>
                  <h3 className="text-2xl font-bold text-white">运营决策支持</h3>
                </div>
                <p className="text-slate-300 mb-6">
                  基于海量历史数据的深度分析，为管理者提供数据驱动的决策支持，优化运营策略
                </p>
                <ul className="space-y-3">
                  {[
                    "自动生成飞行效率分析报告和优化建议",
                    "预测性维护，降低设备故障率70%+",
                    "空域热点分析，优化航线规划",
                    "运营成本分析，识别降本增效机会",
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

      {/* 技术优势 */}
      <section className="py-16 px-4 sm:px-6 lg:px-8 bg-slate-800/30">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4">
              技术优势
            </h2>
            <p className="text-xl text-slate-300">
              基于 Rust 的高性能架构，为低空经济提供可靠的技术保障
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {[
              {
                icon: Zap,
                title: "极致性能",
                desc: "毫秒级响应，支持10万+并发",
                color: "yellow"
              },
              {
                icon: Shield,
                title: "安全可靠",
                desc: "99.99%可用性，数据加密存储",
                color: "green"
              },
              {
                icon: Cloud,
                title: "弹性扩展",
                desc: "支持云边协同，灵活扩展",
                color: "blue"
              },
              {
                icon: Database,
                title: "海量存储",
                desc: "PB级数据存储，永久保存",
                color: "purple"
              }
            ].map((item) => (
              <Card key={item.title} className="bg-slate-800/50 border-slate-700 hover:border-blue-500/50 transition-colors">
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
                company: "某大型物流公司",
                scenario: "无人机配送网络",
                result: "日配送量提升200%，运营成本降低40%",
                metrics: { drones: "500+", routes: "1000+", savings: "3000万/年" }
              },
              {
                company: "某农业科技企业",
                scenario: "智慧农业植保",
                result: "作业效率提升150%，农药使用减少30%",
                metrics: { area: "50万亩", efficiency: "150%↑", cost: "30%↓" }
              },
              {
                company: "某城市管理局",
                scenario: "城市巡检监控",
                result: "巡检覆盖率100%，响应时间缩短80%",
                metrics: { coverage: "100%", response: "3min", events: "10万+/年" }
              }
            ].map((caseItem, index) => (
              <Card key={index} className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <Badge className="mb-2 bg-blue-500/20 text-blue-400 w-fit">
                    <Users className="w-3 h-3 mr-1" />
                    {caseItem.company}
                  </Badge>
                  <CardTitle className="text-white text-lg">{caseItem.scenario}</CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-slate-300 mb-4">{caseItem.result}</p>
                  <div className="space-y-2">
                    {Object.entries(caseItem.metrics).map(([key, value]) => (
                      <div key={key} className="flex justify-between text-sm">
                        <span className="text-slate-400">
                          {key === 'drones' ? '飞行器数量' :
                           key === 'routes' ? '覆盖航线' :
                           key === 'savings' ? '年节省成本' :
                           key === 'area' ? '作业面积' :
                           key === 'efficiency' ? '效率提升' :
                           key === 'cost' ? '成本降低' :
                           key === 'coverage' ? '巡检覆盖' :
                           key === 'response' ? '响应时间' :
                           key === 'events' ? '处理事件' : key}
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
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-gradient-to-r from-blue-900/30 to-cyan-900/30">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-4xl font-bold text-white mb-6">
            开启低空经济智能化之旅
          </h2>
          <p className="text-xl text-slate-300 mb-8">
            立即体验 AgentMem 低空经济解决方案，提升运营效率40%+
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link href="/support">
              <Button size="lg" className="bg-blue-600 hover:bg-blue-700 text-white">
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

