'use client';

import { useState, Suspense } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { FadeIn, SlideIn } from '@/components/ui/animations';
import { 
  Search, 
  MessageCircle, 
  Book, 
  Video, 
  Mail, 
  Phone, 
  Clock, 
  CheckCircle,
  AlertCircle,
  HelpCircle,
  FileText,
  Users,
  Zap,
  Shield
} from 'lucide-react';

/**
 * 支持中心页面组件
 * 提供全面的客户支持服务，包括文档、FAQ、联系方式等
 */
const SupportPageContent = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [contactForm, setContactForm] = useState({
    name: '',
    email: '',
    subject: '',
    message: '',
    priority: 'medium'
  });

  // 支持资源数据
  const supportResources = [
    {
      id: 1,
      title: '快速入门指南',
      description: '5分钟快速了解 AgentMem 的核心功能和使用方法',
      category: 'getting-started',
      type: 'guide',
      icon: Book,
      link: '/docs/getting-started',
      popular: true
    },
    {
      id: 2,
      title: 'API 文档',
      description: '完整的 API 参考文档，包含所有接口和示例代码',
      category: 'api',
      type: 'documentation',
      icon: FileText,
      link: '/docs/api',
      popular: true
    },
    {
      id: 3,
      title: '视频教程',
      description: '通过视频学习 AgentMem 的高级功能和最佳实践',
      category: 'tutorials',
      type: 'video',
      icon: Video,
      link: '/tutorials',
      popular: false
    },
    {
      id: 4,
      title: '故障排除',
      description: '常见问题的解决方案和调试技巧',
      category: 'troubleshooting',
      type: 'guide',
      icon: AlertCircle,
      link: '/docs/troubleshooting',
      popular: true
    },
    {
      id: 5,
      title: '社区论坛',
      description: '与其他开发者交流经验，获取社区支持',
      category: 'community',
      type: 'forum',
      icon: Users,
      link: '/community',
      popular: false
    },
    {
      id: 6,
      title: '性能优化',
      description: '提升 AgentMem 性能的专业建议和配置指南',
      category: 'optimization',
      type: 'guide',
      icon: Zap,
      link: '/docs/optimization',
      popular: false
    }
  ];

  // FAQ 数据
  const faqs = [
    {
      question: '如何开始使用 AgentMem？',
      answer: '您可以从免费版开始，注册账户后即可获得 1,000 次/月的记忆存储额度。我们提供详细的快速入门指南和示例代码帮助您快速上手。',
      category: 'getting-started'
    },
    {
      question: 'AgentMem 支持哪些编程语言？',
      answer: '我们提供多种语言的 SDK，包括 Python、JavaScript/TypeScript、Rust、Go、Java 等。同时支持 REST API，可以与任何编程语言集成。',
      category: 'api'
    },
    {
      question: '如何升级到付费方案？',
      answer: '您可以在控制台中随时升级方案。升级后立即生效，按比例计费。我们支持信用卡、PayPal 等多种支付方式。',
      category: 'billing'
    },
    {
      question: '数据安全如何保障？',
      answer: '我们采用企业级安全措施，包括端到端加密、定期安全审计、SOC 2 合规认证。您的数据完全隔离，我们无法访问您的具体内容。',
      category: 'security'
    },
    {
      question: '支持私有化部署吗？',
      answer: '企业版支持私有化部署，可以部署在您的私有云或本地环境中。我们提供完整的部署指南和技术支持。',
      category: 'deployment'
    },
    {
      question: '如何获得技术支持？',
      answer: '免费版用户可以通过社区论坛获得支持，付费用户享有邮件技术支持，企业版用户还可以获得专属技术支持和电话服务。',
      category: 'support'
    }
  ];

  // 支持计划数据
  const supportPlans = [
    {
      name: '社区支持',
      description: '免费版用户',
      features: [
        '社区论坛',
        '文档和教程',
        '基础 FAQ',
        '社区驱动的解答'
      ],
      responseTime: '社区响应',
      price: '免费',
      icon: Users
    },
    {
      name: '邮件支持',
      description: '专业版用户',
      features: [
        '邮件技术支持',
        '优先级处理',
        '详细问题分析',
        '最佳实践建议'
      ],
      responseTime: '24小时内',
      price: '包含在专业版中',
      icon: Mail
    },
    {
      name: '专属支持',
      description: '企业版用户',
      features: [
        '专属客户经理',
        '电话技术支持',
        '定制解决方案',
        'SLA 保障'
      ],
      responseTime: '4小时内',
      price: '包含在企业版中',
      icon: Shield
    }
  ];

  // 过滤资源
  const filteredResources = supportResources.filter(resource => {
    const matchesSearch = resource.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         resource.description.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesCategory = selectedCategory === 'all' || resource.category === selectedCategory;
    return matchesSearch && matchesCategory;
  });

  // 处理联系表单提交
  const handleContactSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // 这里处理表单提交逻辑
    console.log('Contact form submitted:', contactForm);
    alert('感谢您的反馈！我们会尽快回复您。');
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="container mx-auto px-4 py-16">
        {/* 页面头部 */}
        <div className="text-center mb-16">
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">支持中心</h1>
          <p className="text-xl text-gray-300 max-w-3xl mx-auto">
            我们提供全面的支持服务，帮助您充分利用 AgentMem 的强大功能
          </p>
        </div>

        {/* 快速搜索 */}
        <FadeIn delay={200}>
          <div className="max-w-2xl mx-auto mb-16">
            <div className="relative">
              <Search className="absolute left-4 top-1/2 transform -translate-y-1/2 text-gray-400 w-6 h-6" />
              <Input
                placeholder="搜索文档、教程或常见问题..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="pl-12 py-4 text-lg bg-slate-800/50 border-slate-700 text-white placeholder-gray-400 rounded-xl"
              />
            </div>
          </div>
        </FadeIn>

        {/* 支持计划 */}
        <div className="mb-20">
          <FadeIn delay={300}>
            <h2 className="text-3xl font-bold text-white text-center mb-12">支持计划</h2>
            <div className="grid md:grid-cols-3 gap-8">
              {supportPlans.map((plan, index) => {
                const Icon = plan.icon;
                return (
                  <SlideIn key={plan.name} delay={index * 100} direction="up">
                    <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 p-6 text-center">
                      <Icon className="w-12 h-12 text-purple-400 mx-auto mb-4" />
                      <h3 className="text-xl font-semibold text-white mb-2">{plan.name}</h3>
                      <p className="text-gray-400 mb-4">{plan.description}</p>
                      <div className="space-y-2 mb-6">
                        {plan.features.map((feature, idx) => (
                          <div key={idx} className="flex items-center text-gray-300">
                            <CheckCircle className="w-4 h-4 text-green-400 mr-2 flex-shrink-0" />
                            <span className="text-sm">{feature}</span>
                          </div>
                        ))}
                      </div>
                      <div className="border-t border-slate-700 pt-4">
                        <div className="flex items-center justify-center text-sm text-gray-400 mb-2">
                          <Clock className="w-4 h-4 mr-1" />
                          <span>响应时间: {plan.responseTime}</span>
                        </div>
                        <p className="text-purple-400 font-semibold">{plan.price}</p>
                      </div>
                    </Card>
                  </SlideIn>
                );
              })}
            </div>
          </FadeIn>
        </div>

        {/* 支持资源 */}
        <div className="mb-20">
          <FadeIn delay={400}>
            <h2 className="text-3xl font-bold text-white text-center mb-12">支持资源</h2>
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
              {filteredResources.map((resource, index) => {
                const Icon = resource.icon;
                return (
                  <SlideIn key={resource.id} delay={index * 50} direction="up">
                    <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300 p-6 group cursor-pointer">
                      <div className="flex items-start gap-4">
                        <div className="p-3 bg-purple-600/20 rounded-lg">
                          <Icon className="w-6 h-6 text-purple-400" />
                        </div>
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <h3 className="text-lg font-semibold text-white group-hover:text-purple-400 transition-colors">
                              {resource.title}
                            </h3>
                            {resource.popular && (
                              <Badge className="bg-green-500/20 text-green-400 border-green-500/30 text-xs">
                                热门
                              </Badge>
                            )}
                          </div>
                          <p className="text-gray-300 text-sm">{resource.description}</p>
                        </div>
                      </div>
                    </Card>
                  </SlideIn>
                );
              })}
            </div>
          </FadeIn>
        </div>

        {/* 常见问题 */}
        <div className="mb-20">
          <FadeIn delay={500}>
            <h2 className="text-3xl font-bold text-white text-center mb-12">常见问题</h2>
            <div className="max-w-4xl mx-auto space-y-4">
              {faqs.map((faq, index) => (
                <SlideIn key={index} delay={index * 50} direction="up">
                  <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
                    <details className="group">
                      <summary className="p-6 cursor-pointer list-none">
                        <div className="flex items-center justify-between">
                          <h3 className="text-lg font-semibold text-white group-hover:text-purple-400 transition-colors">
                            {faq.question}
                          </h3>
                          <HelpCircle className="w-5 h-5 text-gray-400 group-hover:text-purple-400 transition-colors" />
                        </div>
                      </summary>
                      <div className="px-6 pb-6 pt-0">
                        <p className="text-gray-300 leading-relaxed">{faq.answer}</p>
                      </div>
                    </details>
                  </Card>
                </SlideIn>
              ))}
            </div>
          </FadeIn>
        </div>

        {/* 联系我们 */}
        <div className="grid md:grid-cols-2 gap-12">
          {/* 联系表单 */}
          <FadeIn delay={600}>
            <Card className="bg-slate-800/50 border-slate-700 p-8">
              <h3 className="text-2xl font-bold text-white mb-6">联系我们</h3>
              <form onSubmit={handleContactSubmit} className="space-y-4">
                <div className="grid md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">姓名</label>
                    <Input
                      value={contactForm.name}
                      onChange={(e) => setContactForm({...contactForm, name: e.target.value})}
                      className="bg-slate-700/50 border-slate-600 text-white"
                      required
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-300 mb-2">邮箱</label>
                    <Input
                      type="email"
                      value={contactForm.email}
                      onChange={(e) => setContactForm({...contactForm, email: e.target.value})}
                      className="bg-slate-700/50 border-slate-600 text-white"
                      required
                    />
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">主题</label>
                  <Input
                    value={contactForm.subject}
                    onChange={(e) => setContactForm({...contactForm, subject: e.target.value})}
                    className="bg-slate-700/50 border-slate-600 text-white"
                    required
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">优先级</label>
                  <select
                    value={contactForm.priority}
                    onChange={(e) => setContactForm({...contactForm, priority: e.target.value})}
                    className="w-full p-3 bg-slate-700/50 border border-slate-600 rounded-md text-white"
                  >
                    <option value="low">低</option>
                    <option value="medium">中</option>
                    <option value="high">高</option>
                    <option value="urgent">紧急</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-300 mb-2">详细描述</label>
                  <Textarea
                    value={contactForm.message}
                    onChange={(e) => setContactForm({...contactForm, message: e.target.value})}
                    className="bg-slate-700/50 border-slate-600 text-white min-h-[120px]"
                    placeholder="请详细描述您遇到的问题或需要的帮助..."
                    required
                  />
                </div>
                <Button type="submit" className="w-full bg-purple-600 hover:bg-purple-700">
                  <MessageCircle className="w-4 h-4 mr-2" />
                  发送消息
                </Button>
              </form>
            </Card>
          </FadeIn>

          {/* 联系信息 */}
          <FadeIn delay={700}>
            <div className="space-y-8">
              <Card className="bg-slate-800/50 border-slate-700 p-6">
                <h3 className="text-xl font-semibold text-white mb-4">联系方式</h3>
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <Mail className="w-5 h-5 text-purple-400" />
                    <div>
                      <p className="text-white font-medium">邮箱支持</p>
                      <p className="text-gray-400 text-sm">support@agentmem.ai</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <Phone className="w-5 h-5 text-purple-400" />
                    <div>
                      <p className="text-white font-medium">电话支持</p>
                      <p className="text-gray-400 text-sm">+86 400-123-4567 (企业版)</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <Clock className="w-5 h-5 text-purple-400" />
                    <div>
                      <p className="text-white font-medium">服务时间</p>
                      <p className="text-gray-400 text-sm">周一至周五 9:00-18:00 (GMT+8)</p>
                    </div>
                  </div>
                </div>
              </Card>

              <Card className="bg-gradient-to-r from-purple-900/50 to-pink-900/50 border-purple-500/30 p-6">
                <h3 className="text-xl font-semibold text-white mb-4">紧急支持</h3>
                <p className="text-gray-300 mb-4">
                  如果您遇到影响业务的紧急问题，企业版用户可以通过以下方式获得优先支持：
                </p>
                <Button className="bg-red-600 hover:bg-red-700">
                  <Phone className="w-4 h-4 mr-2" />
                  紧急热线
                </Button>
              </Card>
            </div>
          </FadeIn>
        </div>
      </div>
    </div>
  );
};

/**
 * 支持中心页面
 * 使用Suspense包裹内容组件以解决预渲染问题
 */
export default function SupportPage() {
  return (
    <Suspense fallback={<div className="text-white text-center py-20">加载中...</div>}>
      <SupportPageContent />
    </Suspense>
  );
}