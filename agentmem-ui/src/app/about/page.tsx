import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Breadcrumb, BreadcrumbContainer } from "@/components/ui/breadcrumb";
import { 
  ScrollToTopButton, 
  PageNavigation, 
  MobilePageNavigation, 
  ScrollProgressIndicator 
} from "@/components/ui/smooth-scroll";
import { Brain, Users, Target, Rocket, Globe, Award, TrendingUp, Building, Mail, Github, Twitter, Linkedin, Cpu, Database, Network, Shield, Zap, Code, Settings, BarChart3, Activity } from "lucide-react";
import Link from "next/link";

/**
 * 关于页面组件 - 展示团队介绍、商业化方向和未来规划
 */
export default function AboutPage() {
  // 页面锚点配置
  const pageAnchors = [
    { id: 'company-intro', label: '公司介绍' },
    { id: 'tech-architecture', label: '技术架构' },
    { id: 'team', label: '团队介绍' },
    { id: 'business', label: '商业化方向' },
    { id: 'roadmap', label: '未来规划' },
    { id: 'investment', label: '投资与合作' },
    { id: 'contact', label: '联系我们' }
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* 滚动进度指示器 */}
      <ScrollProgressIndicator />
      {/* 导航栏 */}
      <nav className="border-b border-slate-800 bg-slate-900/50 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center">
              <Link href="/" className="flex items-center">
                <Brain className="h-8 w-8 text-purple-400" />
                <span className="ml-2 text-xl font-bold text-white">AgentMem</span>
              </Link>
            </div>
            <div className="flex items-center space-x-8">
              <Link href="/" className="text-slate-300 hover:text-white transition-colors">
                首页
              </Link>
              <Link href="/docs" className="text-slate-300 hover:text-white transition-colors">
                文档
              </Link>
              <Link href="/demo" className="text-slate-300 hover:text-white transition-colors">
                演示
              </Link>
              <Link href="/about" className="text-white font-semibold">
                关于
              </Link>
              <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                GitHub
              </Button>
            </div>
          </div>
        </div>
      </nav>

      {/* 面包屑导航 */}
      <BreadcrumbContainer>
        <Breadcrumb />
      </BreadcrumbContainer>

      {/* 页面内容 */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        {/* 公司介绍 */}
        <section id="company-intro" className="mb-20">
          <div className="text-center mb-16">
            <h1 className="text-5xl font-bold text-white mb-6">
              重新定义
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-pink-400">
                AI 记忆
              </span>
            </h1>
            <p className="text-xl text-slate-300 max-w-4xl mx-auto">
              AgentMem 致力于为 AI 代理提供最先进的记忆管理能力，让人工智能拥有真正的记忆和学习能力。
              我们相信，记忆是智能的基础，而智能记忆管理将推动 AI 应用的下一次革命。
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Target className="h-12 w-12 text-purple-400 mx-auto mb-4" />
                <CardTitle className="text-white">我们的使命</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>
                  让每个 AI 代理都拥有智能的记忆能力，
                  推动人工智能向更高层次的智能发展。
                </p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Globe className="h-12 w-12 text-blue-400 mx-auto mb-4" />
                <CardTitle className="text-white">我们的愿景</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>
                  成为全球领先的智能记忆管理平台，
                  为 AI 时代的到来奠定坚实基础。
                </p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Award className="h-12 w-12 text-yellow-400 mx-auto mb-4" />
                <CardTitle className="text-white">我们的价值观</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>
                  创新、开放、可靠、高效。
                  我们致力于构建最优秀的技术产品。
                </p>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* 技术架构 */}
        <section id="tech-architecture" className="mb-20">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <Cpu className="h-10 w-10 text-blue-400 mr-4" />
              技术架构
            </h2>
            <p className="text-xl text-slate-300">
              基于现代化架构设计，确保高性能、高可用性和可扩展性
            </p>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-12">
            {/* 架构图 */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl flex items-center">
                  <Network className="h-6 w-6 text-purple-400 mr-2" />
                  系统架构
                </CardTitle>
                <CardDescription className="text-slate-300">
                  分层架构设计，模块化组件
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {/* 应用层 */}
                  <div className="p-4 bg-purple-500/10 border border-purple-500/30 rounded-lg">
                    <div className="flex items-center mb-2">
                      <Code className="h-5 w-5 text-purple-400 mr-2" />
                      <span className="text-white font-semibold">应用层</span>
                    </div>
                    <div className="text-sm text-slate-300 grid grid-cols-2 gap-2">
                      <span>• REST API</span>
                      <span>• GraphQL</span>
                      <span>• WebSocket</span>
                      <span>• SDK/CLI</span>
                    </div>
                  </div>

                  {/* 服务层 */}
                  <div className="p-4 bg-blue-500/10 border border-blue-500/30 rounded-lg">
                    <div className="flex items-center mb-2">
                      <Settings className="h-5 w-5 text-blue-400 mr-2" />
                      <span className="text-white font-semibold">服务层</span>
                    </div>
                    <div className="text-sm text-slate-300 grid grid-cols-2 gap-2">
                      <span>• 记忆管理</span>
                      <span>• 智能推理</span>
                      <span>• 搜索引擎</span>
                      <span>• 用户管理</span>
                    </div>
                  </div>

                  {/* 数据层 */}
                  <div className="p-4 bg-green-500/10 border border-green-500/30 rounded-lg">
                    <div className="flex items-center mb-2">
                      <Database className="h-5 w-5 text-green-400 mr-2" />
                      <span className="text-white font-semibold">数据层</span>
                    </div>
                    <div className="text-sm text-slate-300 grid grid-cols-2 gap-2">
                      <span>• PostgreSQL</span>
                      <span>• Qdrant</span>
                      <span>• Redis</span>
                      <span>• MinIO</span>
                    </div>
                  </div>

                  {/* 基础设施层 */}
                  <div className="p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
                    <div className="flex items-center mb-2">
                      <Shield className="h-5 w-5 text-yellow-400 mr-2" />
                      <span className="text-white font-semibold">基础设施</span>
                    </div>
                    <div className="text-sm text-slate-300 grid grid-cols-2 gap-2">
                      <span>• Kubernetes</span>
                      <span>• Docker</span>
                      <span>• Prometheus</span>
                      <span>• Grafana</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* 技术特性 */}
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl flex items-center">
                  <Zap className="h-6 w-6 text-yellow-400 mr-2" />
                  核心特性
                </CardTitle>
                <CardDescription className="text-slate-300">
                  先进的技术特性和性能优势
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div>
                  <h4 className="text-white font-semibold mb-3 flex items-center">
                    <Activity className="h-4 w-4 text-green-400 mr-2" />
                    高性能
                  </h4>
                  <div className="space-y-2 text-sm text-slate-300">
                    <div className="flex justify-between">
                      <span>响应时间</span>
                      <span className="text-green-400">&lt; 50ms</span>
                    </div>
                    <div className="flex justify-between">
                      <span>并发处理</span>
                      <span className="text-green-400">10K+ QPS</span>
                    </div>
                    <div className="flex justify-between">
                      <span>内存使用</span>
                      <span className="text-green-400">优化 90%</span>
                    </div>
                  </div>
                </div>

                <Separator className="bg-slate-700" />

                <div>
                  <h4 className="text-white font-semibold mb-3 flex items-center">
                    <Shield className="h-4 w-4 text-blue-400 mr-2" />
                    安全性
                  </h4>
                  <div className="space-y-2 text-sm text-slate-300">
                    <div className="flex items-center">
                      <div className="w-2 h-2 bg-blue-400 rounded-full mr-2"></div>
                      <span>端到端加密</span>
                    </div>
                    <div className="flex items-center">
                      <div className="w-2 h-2 bg-blue-400 rounded-full mr-2"></div>
                      <span>零信任架构</span>
                    </div>
                    <div className="flex items-center">
                      <div className="w-2 h-2 bg-blue-400 rounded-full mr-2"></div>
                      <span>GDPR 合规</span>
                    </div>
                  </div>
                </div>

                <Separator className="bg-slate-700" />

                <div>
                  <h4 className="text-white font-semibold mb-3 flex items-center">
                    <BarChart3 className="h-4 w-4 text-purple-400 mr-2" />
                    可扩展性
                  </h4>
                  <div className="space-y-2 text-sm text-slate-300">
                    <div className="flex items-center">
                      <div className="w-2 h-2 bg-purple-400 rounded-full mr-2"></div>
                      <span>水平扩展</span>
                    </div>
                    <div className="flex items-center">
                      <div className="w-2 h-2 bg-purple-400 rounded-full mr-2"></div>
                      <span>微服务架构</span>
                    </div>
                    <div className="flex items-center">
                      <div className="w-2 h-2 bg-purple-400 rounded-full mr-2"></div>
                      <span>云原生设计</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* 技术栈 */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white text-2xl text-center flex items-center justify-center">
                <Code className="h-6 w-6 text-green-400 mr-2" />
                技术栈
              </CardTitle>
              <CardDescription className="text-slate-300 text-center">
                采用业界领先的技术栈，确保系统的稳定性和先进性
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                <div className="text-center">
                  <div className="w-16 h-16 bg-orange-500/20 rounded-lg flex items-center justify-center mx-auto mb-3">
                    <span className="text-2xl font-bold text-orange-400">Rs</span>
                  </div>
                  <h4 className="text-white font-semibold">Rust</h4>
                  <p className="text-slate-400 text-sm">核心引擎</p>
                </div>
                <div className="text-center">
                  <div className="w-16 h-16 bg-blue-500/20 rounded-lg flex items-center justify-center mx-auto mb-3">
                    <span className="text-2xl font-bold text-blue-400">Py</span>
                  </div>
                  <h4 className="text-white font-semibold">Python</h4>
                  <p className="text-slate-400 text-sm">AI 模型</p>
                </div>
                <div className="text-center">
                  <div className="w-16 h-16 bg-green-500/20 rounded-lg flex items-center justify-center mx-auto mb-3">
                    <span className="text-2xl font-bold text-green-400">TS</span>
                  </div>
                  <h4 className="text-white font-semibold">TypeScript</h4>
                  <p className="text-slate-400 text-sm">前端界面</p>
                </div>
                <div className="text-center">
                  <div className="w-16 h-16 bg-purple-500/20 rounded-lg flex items-center justify-center mx-auto mb-3">
                    <span className="text-2xl font-bold text-purple-400">K8s</span>
                  </div>
                  <h4 className="text-white font-semibold">Kubernetes</h4>
                  <p className="text-slate-400 text-sm">容器编排</p>
                </div>
              </div>
            </CardContent>
          </Card>
        </section>

        {/* 团队介绍 */}
        <section id="team" className="mb-20">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <Users className="h-10 w-10 text-purple-400 mr-4" />
              核心团队
            </h2>
            <p className="text-xl text-slate-300">
              由经验丰富的 AI 研究者和工程师组成的世界级团队
            </p>
          </div>

          {/* 核心团队 */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 mb-16">
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-purple-400 to-pink-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">LZ</span>
                </div>
                <CardTitle className="text-white">张磊</CardTitle>
                <CardDescription className="text-slate-300">创始人 & CEO</CardDescription>
                <div className="flex justify-center mt-2">
                  <Badge className="bg-purple-500/20 text-purple-300 text-xs">AI 研究专家</Badge>
                </div>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4 text-sm">
                  前 Google AI 研究员，专注于大语言模型和记忆系统研究 8 年，
                  发表顶级会议论文 20+ 篇，拥有多项 AI 相关专利。
                </p>
                <div className="space-y-2 mb-4">
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">教育背景:</span> 清华大学计算机博士
                  </div>
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">专业领域:</span> 大语言模型、记忆系统
                  </div>
                </div>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-purple-400">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-purple-400">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-purple-400">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 hover:border-blue-500/50 transition-all duration-300">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-blue-400 to-cyan-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">WY</span>
                </div>
                <CardTitle className="text-white">王宇</CardTitle>
                <CardDescription className="text-slate-300">CTO & 联合创始人</CardDescription>
                <div className="flex justify-center mt-2">
                  <Badge className="bg-blue-500/20 text-blue-300 text-xs">系统架构师</Badge>
                </div>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4 text-sm">
                  前 OpenAI 高级工程师，Rust 生态核心贡献者，
                  在分布式系统和高性能计算领域有 10+ 年丰富经验。
                </p>
                <div className="space-y-2 mb-4">
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">教育背景:</span> MIT 计算机硕士
                  </div>
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">专业领域:</span> 分布式系统、高性能计算
                  </div>
                </div>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-blue-400">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-blue-400">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-blue-400">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 hover:border-green-500/50 transition-all duration-300">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-green-400 to-emerald-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">LM</span>
                </div>
                <CardTitle className="text-white">李明</CardTitle>
                <CardDescription className="text-slate-300">首席科学家</CardDescription>
                <div className="flex justify-center mt-2">
                  <Badge className="bg-green-500/20 text-green-300 text-xs">认知科学家</Badge>
                </div>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4 text-sm">
                  斯坦福大学 AI 博士，专注于认知科学和记忆模型研究，
                  在 Nature、Science 等顶级期刊发表论文 15+ 篇。
                </p>
                <div className="space-y-2 mb-4">
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">教育背景:</span> 斯坦福大学 AI 博士
                  </div>
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">专业领域:</span> 认知科学、记忆模型
                  </div>
                </div>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-green-400">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-green-400">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-green-400">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>

            {/* 新增团队成员 */}
            <Card className="bg-slate-800/50 border-slate-700 hover:border-yellow-500/50 transition-all duration-300">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-yellow-400 to-orange-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">CX</span>
                </div>
                <CardTitle className="text-white">陈雪</CardTitle>
                <CardDescription className="text-slate-300">产品总监</CardDescription>
                <div className="flex justify-center mt-2">
                  <Badge className="bg-yellow-500/20 text-yellow-300 text-xs">产品专家</Badge>
                </div>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4 text-sm">
                  前腾讯高级产品经理，拥有 8 年 AI 产品设计经验，
                  主导过多个千万级用户产品的设计和运营。
                </p>
                <div className="space-y-2 mb-4">
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">教育背景:</span> 北京大学工商管理硕士
                  </div>
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">专业领域:</span> AI 产品设计、用户体验
                  </div>
                </div>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-yellow-400">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-yellow-400">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-yellow-400">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 hover:border-pink-500/50 transition-all duration-300">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-pink-400 to-rose-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">ZH</span>
                </div>
                <CardTitle className="text-white">赵辉</CardTitle>
                <CardDescription className="text-slate-300">工程总监</CardDescription>
                <div className="flex justify-center mt-2">
                  <Badge className="bg-pink-500/20 text-pink-300 text-xs">全栈工程师</Badge>
                </div>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4 text-sm">
                  前字节跳动技术专家，全栈工程师，擅长大规模系统设计，
                  拥有多个开源项目，GitHub 10K+ Stars。
                </p>
                <div className="space-y-2 mb-4">
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">教育背景:</span> 上海交大计算机硕士
                  </div>
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">专业领域:</span> 全栈开发、系统设计
                  </div>
                </div>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-pink-400">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-pink-400">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-pink-400">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 hover:border-indigo-500/50 transition-all duration-300">
              <CardHeader className="text-center">
                <div className="w-20 h-20 bg-gradient-to-r from-indigo-400 to-purple-400 rounded-full mx-auto mb-4 flex items-center justify-center">
                  <span className="text-2xl font-bold text-white">WL</span>
                </div>
                <CardTitle className="text-white">王丽</CardTitle>
                <CardDescription className="text-slate-300">市场总监</CardDescription>
                <div className="flex justify-center mt-2">
                  <Badge className="bg-indigo-500/20 text-indigo-300 text-xs">市场专家</Badge>
                </div>
              </CardHeader>
              <CardContent className="text-slate-300 text-center">
                <p className="mb-4 text-sm">
                  前阿里巴巴高级市场经理，拥有 10 年 B2B 市场经验，
                  成功推广过多个企业级 AI 产品，客户遍布全球。
                </p>
                <div className="space-y-2 mb-4">
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">教育背景:</span> 复旦大学市场营销硕士
                  </div>
                  <div className="text-xs text-slate-400">
                    <span className="font-semibold">专业领域:</span> B2B 市场、品牌推广
                  </div>
                </div>
                <div className="flex justify-center space-x-3">
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-indigo-400">
                    <Github className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-indigo-400">
                    <Twitter className="h-4 w-4" />
                  </Button>
                  <Button size="sm" variant="outline" className="border-slate-600 hover:border-indigo-400">
                    <Linkedin className="h-4 w-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* 团队文化 */}
          <Card className="bg-gradient-to-r from-slate-800/50 to-purple-800/20 border-slate-700">
            <CardHeader className="text-center">
              <CardTitle className="text-white text-2xl flex items-center justify-center">
                <Award className="h-6 w-6 text-yellow-400 mr-2" />
                团队文化与价值观
              </CardTitle>
              <CardDescription className="text-slate-300">
                我们相信优秀的文化是创新的基础
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div className="text-center">
                  <div className="w-16 h-16 bg-purple-500/20 rounded-lg flex items-center justify-center mx-auto mb-4">
                    <Target className="h-8 w-8 text-purple-400" />
                  </div>
                  <h4 className="text-white font-semibold mb-2">追求卓越</h4>
                  <p className="text-slate-300 text-sm">
                    我们对技术和产品有着极高的标准，
                    永远追求最佳的解决方案。
                  </p>
                </div>
                <div className="text-center">
                  <div className="w-16 h-16 bg-blue-500/20 rounded-lg flex items-center justify-center mx-auto mb-4">
                    <Users className="h-8 w-8 text-blue-400" />
                  </div>
                  <h4 className="text-white font-semibold mb-2">团队协作</h4>
                  <p className="text-slate-300 text-sm">
                    我们相信团队的力量，
                    通过协作创造出更大的价值。
                  </p>
                </div>
                <div className="text-center">
                  <div className="w-16 h-16 bg-green-500/20 rounded-lg flex items-center justify-center mx-auto mb-4">
                    <Rocket className="h-8 w-8 text-green-400" />
                  </div>
                  <h4 className="text-white font-semibold mb-2">持续创新</h4>
                  <p className="text-slate-300 text-sm">
                    我们拥抱变化，勇于尝试新技术，
                    不断推动行业发展。
                  </p>
                </div>
                <div className="text-center">
                  <div className="w-16 h-16 bg-yellow-500/20 rounded-lg flex items-center justify-center mx-auto mb-4">
                    <Globe className="h-8 w-8 text-yellow-400" />
                  </div>
                  <h4 className="text-white font-semibold mb-2">开放包容</h4>
                  <p className="text-slate-300 text-sm">
                    我们欢迎不同背景的人才，
                    营造多元化的工作环境。
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </section>

        {/* 商业化方向 */}
        <section id="business" className="mb-20">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <Building className="h-10 w-10 text-blue-400 mr-4" />
              商业化方向
            </h2>
            <p className="text-xl text-slate-300">
              多元化的商业模式，服务不同规模的客户需求
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">企业级解决方案</CardTitle>
                <CardDescription className="text-slate-300">
                  为大型企业提供定制化的智能记忆管理解决方案
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-purple-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">私有化部署</h4>
                      <p className="text-sm">支持本地部署，确保数据安全和合规性</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-purple-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">定制开发</h4>
                      <p className="text-sm">根据企业需求定制特定功能和集成方案</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-purple-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">技术支持</h4>
                      <p className="text-sm">7x24 小时技术支持和专业咨询服务</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-purple-500/20 text-purple-300 mr-2">企业级</Badge>
                  <Badge className="bg-blue-500/20 text-blue-300">定制化</Badge>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">云服务平台</CardTitle>
                <CardDescription className="text-slate-300">
                  基于云的 SaaS 服务，快速集成和部署
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-blue-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">API 服务</h4>
                      <p className="text-sm">RESTful API 和 SDK，支持多种编程语言</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-blue-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">弹性扩展</h4>
                      <p className="text-sm">根据使用量自动扩展，按需付费</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-blue-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">全球部署</h4>
                      <p className="text-sm">多地域部署，确保低延迟和高可用性</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-blue-500/20 text-blue-300 mr-2">云原生</Badge>
                  <Badge className="bg-green-500/20 text-green-300">按需付费</Badge>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">开发者生态</CardTitle>
                <CardDescription className="text-slate-300">
                  构建开放的开发者生态系统和社区
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">开源项目</h4>
                      <p className="text-sm">核心组件开源，促进社区发展和创新</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">开发者工具</h4>
                      <p className="text-sm">提供完整的开发工具链和调试工具</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-green-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">社区支持</h4>
                      <p className="text-sm">活跃的开发者社区和技术交流平台</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-green-500/20 text-green-300 mr-2">开源</Badge>
                  <Badge className="bg-yellow-500/20 text-yellow-300">社区驱动</Badge>
                </div>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <CardTitle className="text-white text-2xl">行业解决方案</CardTitle>
                <CardDescription className="text-slate-300">
                  针对特定行业的垂直解决方案
                </CardDescription>
              </CardHeader>
              <CardContent className="text-slate-300">
                <div className="space-y-4">
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-yellow-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">金融科技</h4>
                      <p className="text-sm">智能客服、风险评估、投资顾问等应用</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-yellow-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">医疗健康</h4>
                      <p className="text-sm">医疗问答、病历管理、诊断辅助等场景</p>
                    </div>
                  </div>
                  <div className="flex items-start">
                    <div className="w-2 h-2 bg-yellow-400 rounded-full mt-2 mr-3 flex-shrink-0"></div>
                    <div>
                      <h4 className="text-white font-semibold">教育培训</h4>
                      <p className="text-sm">个性化学习、知识管理、智能辅导等应用</p>
                    </div>
                  </div>
                </div>
                <div className="mt-6">
                  <Badge className="bg-yellow-500/20 text-yellow-300 mr-2">垂直行业</Badge>
                  <Badge className="bg-red-500/20 text-red-300">专业化</Badge>
                </div>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* 未来规划 */}
        <section id="roadmap" className="mb-20">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <TrendingUp className="h-10 w-10 text-green-400 mr-4" />
              未来规划
            </h2>
            <p className="text-xl text-slate-300">
              持续创新，引领智能记忆管理技术的发展
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <div className="w-12 h-12 bg-purple-500/20 rounded-lg flex items-center justify-center mb-4">
                  <span className="text-2xl font-bold text-purple-400">2024</span>
                </div>
                <CardTitle className="text-white">技术突破年</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 发布 AgentMem 3.0</li>
                  <li>• 多模态记忆支持</li>
                  <li>• 实时学习能力</li>
                  <li>• 联邦学习框架</li>
                  <li>• 边缘计算支持</li>
                </ul>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <div className="w-12 h-12 bg-blue-500/20 rounded-lg flex items-center justify-center mb-4">
                  <span className="text-2xl font-bold text-blue-400">2025</span>
                </div>
                <CardTitle className="text-white">生态扩展年</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• 全球市场拓展</li>
                  <li>• 合作伙伴生态</li>
                  <li>• 行业标准制定</li>
                  <li>• 开发者大会</li>
                  <li>• 认证培训体系</li>
                </ul>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700">
              <CardHeader>
                <div className="w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center mb-4">
                  <span className="text-2xl font-bold text-green-400">2026</span>
                </div>
                <CardTitle className="text-white">智能革命年</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <ul className="space-y-2">
                  <li>• AGI 记忆系统</li>
                  <li>• 量子计算集成</li>
                  <li>• 脑机接口研究</li>
                  <li>• 通用智能平台</li>
                  <li>• 下一代 AI 基础设施</li>
                </ul>
              </CardContent>
            </Card>
          </div>
        </section>

        {/* 投资与合作 */}
        <section id="investment" className="mb-20">
          <Card className="bg-gradient-to-r from-purple-600/20 to-pink-600/20 border-purple-500/30">
            <CardHeader className="text-center">
              <Rocket className="h-16 w-16 text-purple-400 mx-auto mb-4" />
              <CardTitle className="text-white text-3xl mb-4">
                加入我们的旅程
              </CardTitle>
              <CardDescription className="text-slate-300 text-lg">
                我们正在寻找志同道合的投资者、合作伙伴和人才
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
                <div className="text-center">
                  <div className="text-3xl font-bold text-purple-400 mb-2">A 轮</div>
                  <div className="text-slate-300">融资进行中</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-blue-400 mb-2">50+</div>
                  <div className="text-slate-300">合作伙伴</div>
                </div>
                <div className="text-center">
                  <div className="text-3xl font-bold text-green-400 mb-2">100+</div>
                  <div className="text-slate-300">团队成员</div>
                </div>
              </div>
              
              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button size="lg" className="bg-purple-600 hover:bg-purple-700">
                  投资合作
                </Button>
                <Button size="lg" variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                  商务合作
                </Button>
                <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                  加入团队
                </Button>
              </div>
            </CardContent>
          </Card>
        </section>

        {/* 联系我们 */}
        <section id="contact">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-white mb-4 flex items-center justify-center">
              <Mail className="h-10 w-10 text-blue-400 mr-4" />
              联系我们
            </h2>
            <p className="text-xl text-slate-300">
              我们期待与您的交流和合作
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Mail className="h-8 w-8 text-blue-400 mx-auto mb-2" />
                <CardTitle className="text-white">商务合作</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>business@agentmem.com</p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Users className="h-8 w-8 text-green-400 mx-auto mb-2" />
                <CardTitle className="text-white">技术支持</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>support@agentmem.com</p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Building className="h-8 w-8 text-purple-400 mx-auto mb-2" />
                <CardTitle className="text-white">媒体合作</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>media@agentmem.com</p>
              </CardContent>
            </Card>

            <Card className="bg-slate-800/50 border-slate-700 text-center">
              <CardHeader>
                <Rocket className="h-8 w-8 text-yellow-400 mx-auto mb-2" />
                <CardTitle className="text-white">投资合作</CardTitle>
              </CardHeader>
              <CardContent className="text-slate-300">
                <p>invest@agentmem.com</p>
              </CardContent>
            </Card>
          </div>
        </section>
      </div>
      
      {/* 页面导航 */}
      <PageNavigation anchors={pageAnchors} className="hidden lg:block" />
      <MobilePageNavigation anchors={pageAnchors} />
      
      {/* 回到顶部按钮 */}
      <ScrollToTopButton />
    </div>
  );
}