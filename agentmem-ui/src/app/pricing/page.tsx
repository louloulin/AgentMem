'use client';

import { useState, Suspense } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { FadeIn, SlideIn } from '@/components/ui/animations';
import { Check, X, Star, Zap, Shield, Users } from 'lucide-react';

/**
 * 定价页面内容组件
 * 展示 AgentMem 的各种定价方案和功能对比
 */
const PricingPageContent = () => {
  const [billingCycle, setBillingCycle] = useState<'monthly' | 'yearly'>('monthly');

  // 定价方案数据
  const pricingPlans = [
    {
      name: '免费版',
      nameEn: 'Free',
      price: { monthly: 0, yearly: 0 },
      description: '适合个人开发者和小型项目',
      features: [
        '1,000 记忆/月',
        '基础 API 访问',
        '社区支持',
        '基础文档',
        '单一存储后端',
      ],
      limitations: [
        '无高级分析',
        '无优先支持',
        '无自定义集成',
      ],
      popular: false,
      cta: '开始免费使用',
      icon: Users,
    },
    {
      name: '专业版',
      nameEn: 'Pro',
      price: { monthly: 99, yearly: 990 },
      description: '适合成长中的团队和中型企业',
      features: [
        '100,000 记忆/月',
        '高级 API 功能',
        '邮件技术支持',
        '完整文档和教程',
        '多存储后端支持',
        '性能分析仪表板',
        '自定义集成',
        '团队协作功能',
      ],
      limitations: [],
      popular: true,
      cta: '开始专业版',
      icon: Zap,
    },
    {
      name: '企业版',
      nameEn: 'Enterprise',
      price: { monthly: 999, yearly: 9990 },
      description: '适合大型企业和关键业务应用',
      features: [
        '无限记忆存储',
        '全功能 API 访问',
        '专属技术支持',
        '私有化部署选项',
        '所有存储后端',
        '高级安全功能',
        '自定义开发',
        'SLA 保障',
        '专属客户经理',
        '培训和咨询',
      ],
      limitations: [],
      popular: false,
      cta: '联系销售',
      icon: Shield,
    },
  ];

  // 功能对比数据
  const featureComparison = [
    { feature: '记忆存储量', free: '1,000/月', pro: '100,000/月', enterprise: '无限' },
    { feature: 'API 调用', free: '基础', pro: '高级', enterprise: '全功能' },
    { feature: '存储后端', free: '1个', pro: '多个', enterprise: '全部' },
    { feature: '技术支持', free: '社区', pro: '邮件', enterprise: '专属' },
    { feature: '分析仪表板', free: false, pro: true, enterprise: true },
    { feature: '自定义集成', free: false, pro: true, enterprise: true },
    { feature: '私有化部署', free: false, pro: false, enterprise: true },
    { feature: 'SLA 保障', free: false, pro: false, enterprise: true },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* 页面头部 */}
      <div className="container mx-auto px-4 py-16">
        <FadeIn>
          <div className="text-center mb-16">
            <Badge className="mb-4 bg-purple-500/20 text-purple-300 border-purple-500/30">
              定价方案
            </Badge>
            <h1 className="text-4xl md:text-6xl font-bold text-white mb-6">
              选择适合您的
              <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                定价方案
              </span>
            </h1>
            <p className="text-xl text-gray-300 max-w-3xl mx-auto">
              从免费版开始，随着业务增长升级到更高级的方案。所有方案都包含核心功能，无隐藏费用。
            </p>
          </div>
        </FadeIn>

        {/* 计费周期切换 */}
        <FadeIn delay={200}>
          <div className="flex justify-center mb-12">
            <div className="bg-slate-800/50 p-1 rounded-lg border border-slate-700">
              <button
                onClick={() => setBillingCycle('monthly')}
                className={`px-6 py-2 rounded-md transition-all ${
                  billingCycle === 'monthly'
                    ? 'bg-purple-600 text-white'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                按月付费
              </button>
              <button
                onClick={() => setBillingCycle('yearly')}
                className={`px-6 py-2 rounded-md transition-all relative ${
                  billingCycle === 'yearly'
                    ? 'bg-purple-600 text-white'
                    : 'text-gray-400 hover:text-white'
                }`}
              >
                按年付费
                <Badge className="absolute -top-2 -right-2 bg-green-500 text-white text-xs">
                  省20%
                </Badge>
              </button>
            </div>
          </div>
        </FadeIn>

        {/* 定价卡片 */}
        <div className="grid md:grid-cols-3 gap-8 mb-20">
          {pricingPlans.map((plan, index) => {
            const Icon = plan.icon;
            return (
              <SlideIn key={plan.name} delay={index * 100} direction="up">
                <Card className={`relative p-8 bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 ${
                  plan.popular ? 'ring-2 ring-purple-500/50 scale-105' : ''
                }`}>
                  {plan.popular && (
                    <Badge className="absolute -top-3 left-1/2 transform -translate-x-1/2 bg-purple-600 text-white">
                      <Star className="w-3 h-3 mr-1" />
                      最受欢迎
                    </Badge>
                  )}
                  
                  <div className="text-center mb-6">
                    <Icon className="w-12 h-12 text-purple-400 mx-auto mb-4" />
                    <h3 className="text-2xl font-bold text-white mb-2">{plan.name}</h3>
                    <p className="text-gray-400 mb-4">{plan.description}</p>
                    <div className="mb-4">
                      <span className="text-4xl font-bold text-white">
                        ${plan.price[billingCycle]}
                      </span>
                      <span className="text-gray-400 ml-2">
                        /{billingCycle === 'monthly' ? '月' : '年'}
                      </span>
                    </div>
                  </div>

                  <div className="space-y-3 mb-8">
                    {plan.features.map((feature, idx) => (
                      <div key={idx} className="flex items-center text-gray-300">
                        <Check className="w-5 h-5 text-green-400 mr-3 flex-shrink-0" />
                        <span>{feature}</span>
                      </div>
                    ))}
                    {plan.limitations.map((limitation, idx) => (
                      <div key={idx} className="flex items-center text-gray-500">
                        <X className="w-5 h-5 text-red-400 mr-3 flex-shrink-0" />
                        <span>{limitation}</span>
                      </div>
                    ))}
                  </div>

                  <Button 
                    className={`w-full ${
                      plan.popular 
                        ? 'bg-purple-600 hover:bg-purple-700' 
                        : 'bg-slate-700 hover:bg-slate-600'
                    }`}
                  >
                    {plan.cta}
                  </Button>
                </Card>
              </SlideIn>
            );
          })}
        </div>

        {/* 功能对比表 */}
        <FadeIn delay={400}>
          <div className="mb-20">
            <h2 className="text-3xl font-bold text-white text-center mb-12">
              详细功能对比
            </h2>
            <Card className="bg-slate-800/50 border-slate-700 overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-slate-700">
                      <th className="text-left p-4 text-white font-semibold">功能</th>
                      <th className="text-center p-4 text-white font-semibold">免费版</th>
                      <th className="text-center p-4 text-white font-semibold bg-purple-900/20">专业版</th>
                      <th className="text-center p-4 text-white font-semibold">企业版</th>
                    </tr>
                  </thead>
                  <tbody>
                    {featureComparison.map((row, index) => (
                      <tr key={index} className="border-b border-slate-700/50">
                        <td className="p-4 text-gray-300 font-medium">{row.feature}</td>
                        <td className="p-4 text-center">
                          {typeof row.free === 'boolean' ? (
                            row.free ? (
                              <Check className="w-5 h-5 text-green-400 mx-auto" />
                            ) : (
                              <X className="w-5 h-5 text-red-400 mx-auto" />
                            )
                          ) : (
                            <span className="text-gray-300">{row.free}</span>
                          )}
                        </td>
                        <td className="p-4 text-center bg-purple-900/10">
                          {typeof row.pro === 'boolean' ? (
                            row.pro ? (
                              <Check className="w-5 h-5 text-green-400 mx-auto" />
                            ) : (
                              <X className="w-5 h-5 text-red-400 mx-auto" />
                            )
                          ) : (
                            <span className="text-gray-300">{row.pro}</span>
                          )}
                        </td>
                        <td className="p-4 text-center">
                          {typeof row.enterprise === 'boolean' ? (
                            row.enterprise ? (
                              <Check className="w-5 h-5 text-green-400 mx-auto" />
                            ) : (
                              <X className="w-5 h-5 text-red-400 mx-auto" />
                            )
                          ) : (
                            <span className="text-gray-300">{row.enterprise}</span>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </Card>
          </div>
        </FadeIn>

        {/* FAQ 部分 */}
        <FadeIn delay={600}>
          <div className="text-center">
            <h2 className="text-3xl font-bold text-white mb-8">常见问题</h2>
            <div className="grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
              <Card className="p-6 bg-slate-800/50 border-slate-700 text-left">
                <h3 className="text-xl font-semibold text-white mb-3">可以随时升级或降级吗？</h3>
                <p className="text-gray-300">
                  是的，您可以随时升级或降级您的方案。升级立即生效，降级将在当前计费周期结束后生效。
                </p>
              </Card>
              <Card className="p-6 bg-slate-800/50 border-slate-700 text-left">
                <h3 className="text-xl font-semibold text-white mb-3">是否有免费试用期？</h3>
                <p className="text-gray-300">
                  免费版永久免费使用。专业版和企业版提供 14 天免费试用，无需信用卡。
                </p>
              </Card>
              <Card className="p-6 bg-slate-800/50 border-slate-700 text-left">
                <h3 className="text-xl font-semibold text-white mb-3">支持哪些支付方式？</h3>
                <p className="text-gray-300">
                  我们支持信用卡、借记卡、PayPal 和银行转账。企业客户还可以选择发票付款。
                </p>
              </Card>
              <Card className="p-6 bg-slate-800/50 border-slate-700 text-left">
                <h3 className="text-xl font-semibold text-white mb-3">数据安全如何保障？</h3>
                <p className="text-gray-300">
                  我们采用企业级安全措施，包括端到端加密、定期安全审计和 SOC 2 合规认证。
                </p>
              </Card>
            </div>
          </div>
        </FadeIn>
      </div>
    </div>
  );
};

/**
 * 定价页面
 * 使用Suspense包裹内容组件以解决预渲染问题
 */
export default function PricingPage() {
  return (
    <Suspense fallback={<div className="text-white text-center py-20">加载中...</div>}>
      <PricingPageContent />
    </Suspense>
  );
}