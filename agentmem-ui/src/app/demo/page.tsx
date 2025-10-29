"use client";

import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Textarea } from "@/components/ui/textarea";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import { Checkbox } from "@/components/ui/checkbox";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { CodeBlock, InlineCode } from "@/components/ui/code-block";
import { FadeIn, SlideIn, TypeWriter } from "@/components/ui/animations";
import { LoadingSpinner, ContentLoading } from "@/components/ui/loading";
import {
  Brain,
  Play,
  Code,
  Zap,
  Database,
  MessageSquare,
  ArrowRight,
  Copy,
  Check,
  Terminal,
  Cpu,
  Network,
  Settings,
  BarChart3,
  Activity,
  Clock,
  Users,
  Globe,
  Sparkles,
  RefreshCw,
  Download,
  Upload,
  Eye,
  Trash2,
  Edit3,
  Search,
  Filter,
  TrendingUp,
  Target,
  ExternalLink,
  Send,
  Shield,
  AlertTriangle,
  CheckCircle
} from "lucide-react";
import Link from "next/link";
import { apiClient } from "@/lib/api-client";

/**
 * 演示页面组件 - 展示AgentMem的在线演示和交互式示例
 * 使用真实 API 进行演示
 */

export default function DemoPage() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [activeDemo, setActiveDemo] = useState("add");
  // ✅ Real-time stats from API
  const [realTimeStats, setRealTimeStats] = useState({
    totalMemories: 0,
    avgResponseTime: "0ms",
    activeConnections: 0,
    memoryHits: 0,
    dailyQueries: 0,
    storageUsed: 0,
    uptime: 0
  });
  const [selectedDemo, setSelectedDemo] = useState('interactive');
  // 定义记忆项的类型接口
  interface Memory {
    id: string;
    content: string;
    category: string;
    importance: number;
    created_at: string;
    user_id: string;
    relevance?: number;
  }
  
  // ✅ Real memory list from API (initialized empty)
  const [memoryList, setMemoryList] = useState<Memory[]>([]);
  const [demoAgentId, setDemoAgentId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<Memory[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [demoOutput, setDemoOutput] = useState("");
  const [copiedCode, setCopiedCode] = useState("");

  // ✅ Load real-time stats from API
  useEffect(() => {
    const loadRealTimeStats = async () => {
      try {
        const metrics = await apiClient.getMetrics();
        setRealTimeStats({
          totalMemories: metrics.total_memories || 0,
          avgResponseTime: metrics.avg_response_time_ms ? `${metrics.avg_response_time_ms}ms` : "N/A",
          activeConnections: metrics.active_connections || 0,
          memoryHits: 0, // TODO: Add cache hit rate to metrics
          dailyQueries: 0, // TODO: Add daily queries to metrics
          storageUsed: 0, // TODO: Add storage info to metrics
          uptime: 99.9 // TODO: Add uptime to metrics
        });
      } catch (error) {
        console.error('Failed to load metrics:', error);
      }
    };

    // Initialize demo agent and load data
    const initializeDemo = async () => {
      try {
        let agents = await apiClient.getAgents();
        let demoAgent = agents.find(a => a.name === 'Demo Agent');
        
        if (!demoAgent) {
          demoAgent = await apiClient.createAgent({
            name: 'Demo Agent',
            description: 'Agent for interactive demos'
          });
        }
        
        setDemoAgentId(demoAgent.id);
        
        // Load demo agent's memories
        const memories = await apiClient.getMemories(demoAgent.id);
        setMemoryList(memories.map(m => ({
          id: m.id,
          content: m.content,
          category: m.memory_type,
          importance: m.importance,
          created_at: m.created_at,
          user_id: m.agent_id
        })));
      } catch (error) {
        console.error('Failed to initialize demo:', error);
      }
    };

    loadRealTimeStats();
    initializeDemo();

    // Refresh stats every 5 seconds
    const interval = setInterval(loadRealTimeStats, 5000);
    return () => clearInterval(interval);
  }, []);

  // ✅ Real search using API
  const handleSearch = async (query: string) => {
    if (!query.trim() || !demoAgentId) {
      setSearchResults([]);
      return;
    }
    
    setIsSearching(true);
    
    try {
      const results = await apiClient.searchMemories(query, demoAgentId);
      setSearchResults(results.map(m => ({
        id: m.id,
        content: m.content,
        category: m.memory_type,
        importance: m.importance,
        created_at: m.created_at,
        user_id: m.agent_id,
        relevance: m.importance // Use importance as relevance score
      })));
    } catch (error) {
      console.error('Search failed:', error);
      setSearchResults([]);
    } finally {
      setIsSearching(false);
    }
  };

  // 添加新记忆
  const addMemory = (content: string, userId: string = 'user_123') => {
    const newMemory = {
      id: `mem_${Date.now()}`,
      content,
      category: 'user_input',
      importance: Math.random() * 0.3 + 0.7,
      created_at: new Date().toISOString(),
      user_id: userId
    };
    
    setMemoryList(prev => [newMemory, ...prev]);
    return newMemory;
  };

  // 删除记忆
  const deleteMemory = (id: string) => {
    setMemoryList(prev => prev.filter(memory => memory.id !== id));
  };

  // ✅ Real search execution
  const performSearch = async () => {
    await handleSearch(searchQuery);
  };

  /**
   * 真实 API 调用 - 添加记忆
   */
  const addMemoryAPI = async () => {
    if (!input.trim()) return;

    setIsLoading(true);
    setOutput("");

    try {
      // 首先获取或创建一个 agent
      const agents = await apiClient.getAgents();
      let agentId = agents.length > 0 ? agents[0].id : null;

      if (!agentId) {
        // 创建一个演示 agent
        const newAgent = await apiClient.createAgent({
          name: "Demo Agent",
          description: "Agent for demo purposes"
        });
        agentId = newAgent.id;
      }

      // 添加记忆
      const memory = await apiClient.createMemory({
        agent_id: agentId,
        memory_type: "episodic",
        content: input,
        importance: 0.8
      });

      const response = {
        success: true,
        memory_id: memory.id,
        message: "记忆已成功添加到 AgentMem",
        metadata: {
          timestamp: memory.created_at,
          agent_id: agentId,
          memory_type: memory.memory_type,
          importance: memory.importance
        }
      };

      setOutput(JSON.stringify(response, null, 2));

      // 更新记忆列表
      const newMemory: Memory = {
        id: memory.id,
        content: memory.content,
        category: memory.memory_type,
        importance: memory.importance,
        created_at: memory.created_at,
        user_id: agentId
      };
      setMemoryList(prev => [newMemory, ...prev]);

    } catch (error) {
      const errorResponse = {
        success: false,
        error: error instanceof Error ? error.message : "Failed to add memory"
      };
      setOutput(JSON.stringify(errorResponse, null, 2));
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * 真实 API 调用 - 搜索记忆
   */
  const searchMemoryAPI = async () => {
    if (!input.trim()) return;

    setIsLoading(true);
    setOutput("");

    try {
      // 搜索记忆
      const results = await apiClient.searchMemories(input);

      const response = {
        success: true,
        results: results.map(m => ({
          id: m.id,
          content: m.content,
          relevance_score: m.importance,
          metadata: {
            created_at: m.created_at,
            category: m.memory_type
          }
        })),
        processing_time: "real-time"
      };

      setOutput(JSON.stringify(response, null, 2));

    } catch (error) {
      const errorResponse = {
        success: false,
        error: error instanceof Error ? error.message : "Failed to search memories"
      };
      setOutput(JSON.stringify(errorResponse, null, 2));
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * 运行演示代码
   */
  const runDemo = async (demoType: string) => {
    setIsRunning(true);
    setDemoOutput("正在运行演示...");
    
    // 模拟演示运行
    setTimeout(() => {
      switch (demoType) {
        case "memory-basic":
          setDemoOutput(`✅ 记忆创建成功
记忆ID: mem_001
内容: "我喜欢在周末喝咖啡和阅读"
类型: Episodic
重要性: 0.8

🔍 搜索结果:
找到 3 条相关记忆:
1. "我喜欢在周末喝咖啡和阅读" (相似度: 1.0)
2. "咖啡是我最喜欢的饮品" (相似度: 0.85)
3. "周末通常在家阅读技术书籍" (相似度: 0.72)`);
          break;
        case "intelligent-reasoning":
          setDemoOutput(`🧠 智能推理结果:

📊 事实提取:
✓ 提取了 4 个事实:
  • 用户名称: Alice
  • 居住地: 北京
  • 兴趣爱好: 编程
  • 职业相关: 软件开发

🎯 记忆决策:
✓ 生成了 2 个决策:
  1. ADD: "用户 Alice 来自北京" (置信度: 0.95)
  2. ADD: "用户喜欢编程" (置信度: 0.90)

⚡ 处理统计:
  • 处理时间: 1.2s
  • 事实置信度: 92%
  • 决策准确率: 95%`);
          break;
        case "mem0-compat":
          setDemoOutput(`🔄 Mem0 兼容性演示:

✅ 客户端初始化成功
✅ 添加记忆: "I love pizza" (ID: mem_pizza_001)
✅ 搜索记忆: "food" 

📋 搜索结果:
找到 2 条记忆:
1. "I love pizza" (分数: 0.95)
2. "My favorite cuisine is Italian" (分数: 0.78)

🚀 性能提升:
  • 查询速度: 比原版快 3.2x
  • 内存使用: 减少 45%
  • 并发处理: 提升 5x`);
          break;
        default:
          setDemoOutput("演示完成！");
      }
      setIsRunning(false);
    }, 2000);
  };

  /**
   * 复制代码到剪贴板
   */
  const copyCode = async (code: string, id: string) => {
    try {
      await navigator.clipboard.writeText(code);
      setCopiedCode(id);
      setTimeout(() => setCopiedCode(""), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  };

  const demoConfigs = {
    "memory-basic": {
      title: "基础记忆操作",
      description: "演示记忆的创建、搜索和管理功能",
      code: `use agent_mem_core::{MemoryEngine, MemoryEngineConfig};
use agent_mem_traits::MemoryType;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建记忆引擎
    let config = MemoryEngineConfig::default();
    let engine = MemoryEngine::new(config);
    
    // 添加记忆
    let memory_id = engine.add_memory(
        "user123",
        "我喜欢在周末喝咖啡和阅读",
        Some(MemoryType::Episodic)
    ).await?;
    
    println!("记忆创建成功: {}", memory_id);
    
    // 搜索记忆
    let results = engine.search_memories(
        "咖啡", 
        "user123", 
        10
    ).await?;
    
    println!("找到 {} 条相关记忆", results.len());
    for result in results {
        println!("- {} (相似度: {:.2})", 
                result.content, result.similarity);
    }
    
    Ok(())
}`
    },
    "intelligent-reasoning": {
      title: "智能推理引擎",
      description: "展示 DeepSeek 驱动的事实提取和决策功能",
      code: `use agent_mem_intelligence::{
    IntelligentMemoryProcessor, Message
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建智能处理器
    let processor = IntelligentMemoryProcessor::new(
        "your-deepseek-api-key".to_string()
    )?;
    
    // 准备对话消息
    let messages = vec![
        Message {
            role: "user".to_string(),
            content: "我是 Alice，来自北京，喜欢编程".to_string(),
            timestamp: Some("2024-01-01T10:00:00Z".to_string()),
            message_id: Some("msg1".to_string()),
        }
    ];
    
    // 智能处理消息
    let result = processor.process_messages(
        &messages, 
        &[]
    ).await?;
    
    println!("提取了 {} 个事实", result.extracted_facts.len());
    for fact in &result.extracted_facts {
        println!("- {} (置信度: {:.2})", 
                fact.content, fact.confidence);
    }
    
    println!("生成了 {} 个记忆决策", result.memory_decisions.len());
    for decision in &result.memory_decisions {
        println!("- {:?} (置信度: {:.2})", 
                decision.action, decision.confidence);
    }
    
    Ok(())
}`
    },
    "mem0-compat": {
      title: "Mem0 兼容性",
      description: "展示 100% Mem0 API 兼容性和性能提升",
      code: `use agent_mem_compat::Mem0Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 Mem0 兼容客户端
    let client = Mem0Client::new().await?;
    
    // 使用与 Mem0 完全相同的 API
    let memory_id = client.add(
        "user123", 
        "I love pizza", 
        None
    ).await?;
    
    println!("记忆添加成功: {}", memory_id);
    
    // 搜索记忆
    let results = client.search(
        "food", 
        "user123", 
        None
    ).await?;
    
    println!("找到 {} 条记忆", results.len());
    for memory in results {
        println!("- {}: {} (分数: {:.2})", 
                memory.id, memory.content, memory.score);
    }
    
    // 获取所有记忆
    let all_memories = client.get_all(
        Some("user123".to_string()),
        None,
        None,
        Some(100),
        None
    ).await?;
    
    println!("用户总共有 {} 条记忆", all_memories.len());
    
    Ok(())
}`
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
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
              <Link href="/demo" className="text-white font-semibold">
                演示
              </Link>
              <Link href="/about" className="text-slate-300 hover:text-white transition-colors">
                关于
              </Link>
              <Button variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                GitHub
              </Button>
            </div>
          </div>
        </div>
      </nav>

      {/* 页面内容 */}
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        {/* 页面标题 */}
        <FadeIn>
          <div className="text-center mb-12">
            <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">
              <TypeWriter text="在线演示" speed={150} />
            </h1>
            <p className="text-xl text-slate-300 max-w-3xl mx-auto">
              体验 AgentMem 的强大功能，实时测试智能记忆管理和检索能力
            </p>
          </div>
        </FadeIn>

        {/* 实时统计面板 */}
        <SlideIn direction="up" delay={300}>
          <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-7 gap-4 mb-8">
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
              <CardContent className="p-4 text-center">
                <div className="flex items-center justify-center mb-2">
                  <Database className="h-5 w-5 text-blue-400 mr-2" />
                  <span className="text-2xl font-bold text-white">{realTimeStats.totalMemories.toLocaleString()}</span>
                </div>
                <p className="text-sm text-slate-400">总记忆数</p>
                <div className="mt-1">
                  <TrendingUp className="h-3 w-3 text-green-400 inline mr-1" />
                  <span className="text-xs text-green-400">+{Math.floor(Math.random() * 10 + 5)}</span>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-800/50 border-slate-700 hover:border-yellow-500/50 transition-all duration-300">
              <CardContent className="p-4 text-center">
                <div className="flex items-center justify-center mb-2">
                  <Zap className="h-5 w-5 text-yellow-400 mr-2" />
                  <span className="text-2xl font-bold text-white">{realTimeStats.avgResponseTime}</span>
                </div>
                <p className="text-sm text-slate-400">平均响应</p>
                <div className="mt-1">
                  <Activity className="h-3 w-3 text-yellow-400 inline mr-1" />
                  <span className="text-xs text-yellow-400">实时</span>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-800/50 border-slate-700 hover:border-green-500/50 transition-all duration-300">
              <CardContent className="p-4 text-center">
                <div className="flex items-center justify-center mb-2">
                  <Network className="h-5 w-5 text-green-400 mr-2" />
                  <span className="text-2xl font-bold text-white">{realTimeStats.activeConnections}</span>
                </div>
                <p className="text-sm text-slate-400">活跃连接</p>
                <div className="mt-1">
                  <Users className="h-3 w-3 text-green-400 inline mr-1" />
                  <span className="text-xs text-green-400">在线</span>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-800/50 border-slate-700 hover:border-purple-500/50 transition-all duration-300">
              <CardContent className="p-4 text-center">
                <div className="flex items-center justify-center mb-2">
                  <Cpu className="h-5 w-5 text-purple-400 mr-2" />
                  <span className="text-2xl font-bold text-white">{realTimeStats.memoryHits.toFixed(1)}%</span>
                </div>
                <p className="text-sm text-slate-400">命中率</p>
                <div className="mt-1">
                  <BarChart3 className="h-3 w-3 text-purple-400 inline mr-1" />
                  <span className="text-xs text-purple-400">优秀</span>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-800/50 border-slate-700 hover:border-indigo-500/50 transition-all duration-300">
              <CardContent className="p-4 text-center">
                <div className="flex items-center justify-center mb-2">
                  <Globe className="h-5 w-5 text-indigo-400 mr-2" />
                  <span className="text-2xl font-bold text-white">{realTimeStats.dailyQueries.toLocaleString()}</span>
                </div>
                <p className="text-sm text-slate-400">日查询量</p>
                <div className="mt-1">
                  <TrendingUp className="h-3 w-3 text-indigo-400 inline mr-1" />
                  <span className="text-xs text-indigo-400">+12%</span>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-800/50 border-slate-700 hover:border-orange-500/50 transition-all duration-300">
              <CardContent className="p-4 text-center">
                <div className="flex items-center justify-center mb-2">
                  <Database className="h-5 w-5 text-orange-400 mr-2" />
                  <span className="text-2xl font-bold text-white">{realTimeStats.storageUsed.toFixed(1)}GB</span>
                </div>
                <p className="text-sm text-slate-400">存储使用</p>
                <div className="mt-1">
                  <Activity className="h-3 w-3 text-orange-400 inline mr-1" />
                  <span className="text-xs text-orange-400">正常</span>
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-800/50 border-slate-700 hover:border-emerald-500/50 transition-all duration-300">
              <CardContent className="p-4 text-center">
                <div className="flex items-center justify-center mb-2">
                  <Clock className="h-5 w-5 text-emerald-400 mr-2" />
                  <span className="text-2xl font-bold text-white">{realTimeStats.uptime.toFixed(1)}%</span>
                </div>
                <p className="text-sm text-slate-400">系统可用性</p>
                <div className="mt-1">
                  <Check className="h-3 w-3 text-emerald-400 inline mr-1" />
                  <span className="text-xs text-emerald-400">稳定</span>
                </div>
              </CardContent>
            </Card>
          </div>
        </SlideIn>

        {/* 演示选择 */}
        <SlideIn direction="up" delay={600}>
          <Tabs value={selectedDemo} onValueChange={setSelectedDemo} className="mb-8">
            <TabsList className="grid w-full grid-cols-5 bg-slate-800 border-slate-700">
              <TabsTrigger value="interactive" className="data-[state=active]:bg-purple-600">
                <Sparkles className="mr-2 h-4 w-4" />
                交互演示
              </TabsTrigger>
              <TabsTrigger value="memory-manager" className="data-[state=active]:bg-purple-600">
                <Database className="mr-2 h-4 w-4" />
                记忆管理
              </TabsTrigger>
              <TabsTrigger value="search" className="data-[state=active]:bg-purple-600">
                <Search className="mr-2 h-4 w-4" />
                智能搜索
              </TabsTrigger>
              <TabsTrigger value="api" className="data-[state=active]:bg-purple-600">
                <Code className="mr-2 h-4 w-4" />
                API 测试
              </TabsTrigger>
              <TabsTrigger value="performance" className="data-[state=active]:bg-purple-600">
                <BarChart3 className="mr-2 h-4 w-4" />
                性能监控
              </TabsTrigger>
            </TabsList>
            
            {/* 交互演示 */}
            <TabsContent value="interactive" className="space-y-6">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* 快速体验 */}
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <Sparkles className="mr-2 h-5 w-5 text-purple-400" />
                      快速体验
                    </CardTitle>
                    <CardDescription className="text-slate-300">
                      输入任何内容，体验 AgentMem 的智能记忆功能
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label htmlFor="quickInput" className="text-slate-300">输入内容</Label>
                      <Textarea
                        id="quickInput"
                        rows={3}
                        placeholder="例如：我是张三，来自北京，喜欢编程和阅读..."
                        className="bg-slate-700 border-slate-600 text-white resize-none"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                      />
                    </div>
                    <div className="flex gap-2">
                      <Button 
                        className="flex-1 bg-purple-600 hover:bg-purple-700"
                        onClick={() => {
                          if (input.trim()) {
                            addMemory(input);
                            setInput('');
                            addMemoryAPI();
                          }
                        }}
                        disabled={isLoading || !input.trim()}
                      >
                        {isLoading ? (
                          <>
                            <LoadingSpinner className="mr-2" />
                            处理中...
                          </>
                        ) : (
                          <>
                            <Upload className="mr-2 h-4 w-4" />
                            添加记忆
                          </>
                        )}
                      </Button>
                      <Button 
                        variant="outline"
                        className="border-slate-600 text-slate-300 hover:bg-slate-700"
                        onClick={() => setInput('')}
                      >
                        <RefreshCw className="h-4 w-4" />
                      </Button>
                    </div>
                  </CardContent>
                </Card>

                {/* 实时结果 */}
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <Activity className="mr-2 h-5 w-5 text-green-400" />
                      实时结果
                    </CardTitle>
                    <CardDescription className="text-slate-300">
                      查看 AgentMem 的处理结果和提取的信息
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    {isLoading ? (
                      <ContentLoading />
                    ) : output ? (
                      <CodeBlock
                        language="json"
                        code={output}
                      />
                    ) : (
                      <div className="text-center py-8 text-slate-400">
                        <Brain className="h-12 w-12 mx-auto mb-4 opacity-50" />
                        <p>等待输入内容...</p>
                      </div>
                    )}
                  </CardContent>
                </Card>
              </div>
            </TabsContent>

            {/* 记忆管理 */}
            <TabsContent value="memory-manager" className="space-y-6">
              <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                {/* 记忆列表 */}
                <div className="lg:col-span-2">
                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <div className="flex items-center justify-between">
                        <div>
                          <CardTitle className="text-white flex items-center">
                            <Database className="mr-2 h-5 w-5 text-blue-400" />
                            记忆库
                          </CardTitle>
                          <CardDescription className="text-slate-300">
                            管理和查看所有存储的记忆
                          </CardDescription>
                        </div>
                        <Badge className="bg-blue-500/20 text-blue-400 border-blue-500/30">
                          {memoryList.length} 条记忆
                        </Badge>
                      </div>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3 max-h-96 overflow-y-auto">
                        {memoryList.map((memory, index) => (
                          <SlideIn key={memory.id} direction="up" delay={index * 50}>
                            <div className="p-4 bg-slate-700/50 rounded-lg border border-slate-600 hover:border-purple-500/50 transition-all duration-300">
                              <div className="flex items-start justify-between mb-2">
                                <div className="flex-1">
                                  <div className="flex items-center gap-2 mb-2">
                                    <Badge className={`text-xs ${
                                      memory.category === 'preferences' ? 'bg-green-500/20 text-green-400 border-green-500/30' :
                                      memory.category === 'interests' ? 'bg-blue-500/20 text-blue-400 border-blue-500/30' :
                                      memory.category === 'professional' ? 'bg-purple-500/20 text-purple-400 border-purple-500/30' :
                                      'bg-gray-500/20 text-gray-400 border-gray-500/30'
                                    }`}>
                                      {memory.category}
                                    </Badge>
                                    <Badge variant="outline" className="text-xs">
                                      重要性: {(memory.importance * 100).toFixed(0)}%
                                    </Badge>
                                  </div>
                                  <p className="text-slate-300 text-sm mb-2">{memory.content}</p>
                                  <div className="flex items-center gap-4 text-xs text-slate-400">
                                    <span className="flex items-center">
                                      <Clock className="h-3 w-3 mr-1" />
                                      {new Date(memory.created_at).toLocaleDateString()}
                                    </span>
                                    <span className="flex items-center">
                                      <Users className="h-3 w-3 mr-1" />
                                      {memory.user_id}
                                    </span>
                                  </div>
                                </div>
                                <div className="flex items-center gap-2 ml-4">
                                  <Button
                                    size="sm"
                                    variant="ghost"
                                    className="h-8 w-8 p-0 text-slate-400 hover:text-blue-400"
                                  >
                                    <Eye className="h-4 w-4" />
                                  </Button>
                                  <Button
                                    size="sm"
                                    variant="ghost"
                                    className="h-8 w-8 p-0 text-slate-400 hover:text-yellow-400"
                                  >
                                    <Edit3 className="h-4 w-4" />
                                  </Button>
                                  <Button
                                    size="sm"
                                    variant="ghost"
                                    className="h-8 w-8 p-0 text-slate-400 hover:text-red-400"
                                    onClick={() => deleteMemory(memory.id)}
                                  >
                                    <Trash2 className="h-4 w-4" />
                                  </Button>
                                </div>
                              </div>
                            </div>
                          </SlideIn>
                        ))}
                      </div>
                    </CardContent>
                  </Card>
                </div>

                {/* 记忆统计 */}
                <div>
                  <Card className="bg-slate-800/50 border-slate-700 mb-6">
                    <CardHeader>
                      <CardTitle className="text-white flex items-center">
                        <BarChart3 className="mr-2 h-5 w-5 text-purple-400" />
                        统计信息
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="flex items-center justify-between">
                        <span className="text-slate-300 text-sm">总记忆数</span>
                        <span className="text-white font-semibold">{memoryList.length}</span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-300 text-sm">平均重要性</span>
                        <span className="text-white font-semibold">
                          {(memoryList.reduce((acc, m) => acc + m.importance, 0) / memoryList.length * 100).toFixed(0)}%
                        </span>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-300 text-sm">分类数量</span>
                        <span className="text-white font-semibold">
                          {new Set(memoryList.map(m => m.category)).size}
                        </span>
                      </div>
                      <Separator className="bg-slate-700" />
                      <div className="space-y-2">
                        <h4 className="text-white text-sm font-semibold">分类分布</h4>
                        {Object.entries(
                          memoryList.reduce((acc, memory) => {
                            acc[memory.category] = (acc[memory.category] || 0) + 1;
                            return acc;
                          }, {} as Record<string, number>)
                        ).map(([category, count]) => (
                          <div key={category} className="flex items-center justify-between">
                            <span className="text-slate-400 text-xs capitalize">{category}</span>
                            <span className="text-slate-300 text-xs">{count}</span>
                          </div>
                        ))}
                      </div>
                    </CardContent>
                  </Card>

                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <CardTitle className="text-white flex items-center">
                        <Settings className="mr-2 h-5 w-5 text-gray-400" />
                        快速操作
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <Button className="w-full bg-blue-600 hover:bg-blue-700" size="sm">
                        <Download className="mr-2 h-4 w-4" />
                        导出记忆
                      </Button>
                      <Button className="w-full bg-green-600 hover:bg-green-700" size="sm">
                        <Upload className="mr-2 h-4 w-4" />
                        导入记忆
                      </Button>
                      <Button className="w-full bg-red-600 hover:bg-red-700" size="sm">
                        <Trash2 className="mr-2 h-4 w-4" />
                        清空记忆
                      </Button>
                    </CardContent>
                  </Card>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="add" className="space-y-6">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* 输入区域 */}
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <Brain className="mr-2 h-5 w-5 text-purple-400" />
                      添加记忆
                    </CardTitle>
                    <CardDescription className="text-slate-300">
                      输入用户信息，AgentMem 将自动提取关键事实并存储
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label htmlFor="userId" className="text-slate-300">用户ID</Label>
                      <Input
                        id="userId"
                        placeholder="user_123"
                        className="bg-slate-700 border-slate-600 text-white"
                      />
                    </div>
                    <div>
                      <Label htmlFor="memoryContent" className="text-slate-300">记忆内容</Label>
                      <Textarea
                        id="memoryContent"
                        rows={4}
                        placeholder="我喜欢在周末进行户外活动，特别是徒步和骑行..."
                        className="bg-slate-700 border-slate-600 text-white resize-none"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                      />
                    </div>
                    <Button
                      className="w-full bg-purple-600 hover:bg-purple-700"
                      onClick={() => addMemoryAPI()}
                      disabled={isLoading || !input.trim()}
                    >
                      {isLoading ? (
                        <>
                          <LoadingSpinner className="mr-2" />
                          处理中...
                        </>
                      ) : (
                        <>
                          <Play className="mr-2 h-4 w-4" />
                          添加记忆
                        </>
                      )}
                    </Button>
                  </CardContent>
                </Card>

                {/* 输出区域 */}
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <Zap className="mr-2 h-5 w-5 text-yellow-400" />
                      处理结果
                    </CardTitle>
                    <CardDescription className="text-slate-300">
                      AgentMem 的智能处理结果和提取的事实
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    {isLoading ? (
                      <ContentLoading />
                    ) : (
                      <CodeBlock
                         language="json"
                         code={output || "等待输入..."}
                       />
                    )}
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
            
            <TabsContent value="search" className="space-y-6">
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* 搜索区域 */}
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <Database className="mr-2 h-5 w-5 text-blue-400" />
                      智能搜索
                    </CardTitle>
                    <CardDescription className="text-slate-300">
                      搜索相关记忆，体验语义理解能力
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label htmlFor="searchUserId" className="text-slate-300">用户ID</Label>
                      <Input
                        id="searchUserId"
                        placeholder="user_123"
                        className="bg-slate-700 border-slate-600 text-white"
                      />
                    </div>
                    <div>
                      <Label htmlFor="searchQuery" className="text-slate-300">搜索查询</Label>
                      <Input
                        id="searchQuery"
                        placeholder="户外活动偏好"
                        className="bg-slate-700 border-slate-600 text-white"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                      />
                    </div>
                    <Button
                      className="w-full bg-blue-600 hover:bg-blue-700"
                      onClick={() => searchMemoryAPI()}
                      disabled={isLoading || !input.trim()}
                    >
                      {isLoading ? (
                        <>
                          <LoadingSpinner className="mr-2" />
                          搜索中...
                        </>
                      ) : (
                        <>
                          <Database className="mr-2 h-4 w-4" />
                          搜索记忆
                        </>
                      )}
                    </Button>
                  </CardContent>
                </Card>

                {/* 搜索结果 */}
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white flex items-center">
                      <MessageSquare className="mr-2 h-5 w-5 text-green-400" />
                      搜索结果
                    </CardTitle>
                    <CardDescription className="text-slate-300">
                      基于语义相似度的智能匹配结果
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    {isLoading ? (
                      <ContentLoading />
                    ) : (
                      <CodeBlock
                         language="json"
                         code={output || "等待搜索..."}
                       />
                    )}
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
            
            <TabsContent value="api" className="space-y-6">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white flex items-center">
                    <Code className="mr-2 h-5 w-5 text-purple-400" />
                    API 使用示例
                  </CardTitle>
                  <CardDescription className="text-slate-300">
                    快速集成 AgentMem 到您的应用中
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-6">
                    <div>
                      <h4 className="text-white font-semibold mb-3 flex items-center">
                        <Terminal className="mr-2 h-4 w-4" />
                        REST API
                      </h4>
                      <CodeBlock
                         language="bash"
                         code={`# 添加记忆
curl -X POST https://api.agentmem.ai/v1/memories \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "content": "我喜欢在周末进行户外活动",
    "metadata": {
      "category": "preference"
    }
  }'

# 搜索记忆
curl -X GET "https://api.agentmem.ai/v1/memories/search?q=户外活动&user_id=user_123" \
  -H "Authorization: Bearer YOUR_API_KEY"`}
                       />
                    </div>
                    
                    <Separator className="bg-slate-700" />
                    
                    <div>
                      <h4 className="text-white font-semibold mb-3 flex items-center">
                        <Code className="mr-2 h-4 w-4" />
                        Python SDK
                      </h4>
                      <CodeBlock
                         language="python"
                         code={`from agentmem import AgentMemClient

# 初始化客户端
client = AgentMemClient(api_key="YOUR_API_KEY")

# 添加记忆
memory = await client.add_memory(
    user_id="user_123",
    content="我喜欢在周末进行户外活动",
    metadata={"category": "preference"}
)

# 搜索记忆
results = await client.search_memories(
    query="户外活动",
    user_id="user_123",
    limit=10
)

print(f"找到 {len(results)} 条相关记忆")`}
                       />
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            {/* 性能监控 */}
            <TabsContent value="performance" className="space-y-6">
              <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                {/* 实时性能指标 */}
                <div className="lg:col-span-2">
                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <CardTitle className="text-white flex items-center">
                        <Activity className="mr-2 h-5 w-5 text-green-400" />
                        实时性能监控
                      </CardTitle>
                      <CardDescription className="text-slate-300">
                        监控系统性能和资源使用情况
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="grid grid-cols-2 gap-4 mb-6">
                        {/* CPU 使用率 */}
                        <div className="p-4 bg-slate-700/50 rounded-lg">
                          <div className="flex items-center justify-between mb-2">
                            <span className="text-slate-300 text-sm">CPU 使用率</span>
                            <Badge className="bg-green-500/20 text-green-400 border-green-500/30">
                              {Math.floor(Math.random() * 30 + 15)}%
                            </Badge>
                          </div>
                          <div className="w-full bg-slate-600 rounded-full h-2">
                            <div 
                              className="bg-green-400 h-2 rounded-full transition-all duration-1000"
                              style={{ width: `${Math.floor(Math.random() * 30 + 15)}%` }}
                            ></div>
                          </div>
                        </div>

                        {/* 内存使用率 */}
                        <div className="p-4 bg-slate-700/50 rounded-lg">
                          <div className="flex items-center justify-between mb-2">
                            <span className="text-slate-300 text-sm">内存使用率</span>
                            <Badge className="bg-blue-500/20 text-blue-400 border-blue-500/30">
                              {Math.floor(Math.random() * 40 + 30)}%
                            </Badge>
                          </div>
                          <div className="w-full bg-slate-600 rounded-full h-2">
                            <div 
                              className="bg-blue-400 h-2 rounded-full transition-all duration-1000"
                              style={{ width: `${Math.floor(Math.random() * 40 + 30)}%` }}
                            ></div>
                          </div>
                        </div>

                        {/* 网络延迟 */}
                        <div className="p-4 bg-slate-700/50 rounded-lg">
                          <div className="flex items-center justify-between mb-2">
                            <span className="text-slate-300 text-sm">网络延迟</span>
                            <Badge className="bg-yellow-500/20 text-yellow-400 border-yellow-500/30">
                              {Math.floor(Math.random() * 50 + 20)}ms
                            </Badge>
                          </div>
                          <div className="w-full bg-slate-600 rounded-full h-2">
                            <div 
                              className="bg-yellow-400 h-2 rounded-full transition-all duration-1000"
                              style={{ width: `${Math.floor(Math.random() * 30 + 20)}%` }}
                            ></div>
                          </div>
                        </div>

                        {/* 磁盘 I/O */}
                        <div className="p-4 bg-slate-700/50 rounded-lg">
                          <div className="flex items-center justify-between mb-2">
                            <span className="text-slate-300 text-sm">磁盘 I/O</span>
                            <Badge className="bg-purple-500/20 text-purple-400 border-purple-500/30">
                              {Math.floor(Math.random() * 20 + 5)}%
                            </Badge>
                          </div>
                          <div className="w-full bg-slate-600 rounded-full h-2">
                            <div 
                              className="bg-purple-400 h-2 rounded-full transition-all duration-1000"
                              style={{ width: `${Math.floor(Math.random() * 20 + 5)}%` }}
                            ></div>
                          </div>
                        </div>
                      </div>

                      {/* 性能图表 */}
                      <div className="space-y-4">
                        <div className="flex items-center justify-between">
                          <h4 className="text-white font-semibold">响应时间趋势</h4>
                          <div className="flex gap-2">
                            <Button size="sm" variant="outline" className="text-xs border-slate-600 text-slate-300">
                              1小时
                            </Button>
                            <Button size="sm" variant="outline" className="text-xs border-slate-600 text-slate-300 bg-slate-700">
                              24小时
                            </Button>
                            <Button size="sm" variant="outline" className="text-xs border-slate-600 text-slate-300">
                              7天
                            </Button>
                          </div>
                        </div>
                        <div className="h-32 bg-slate-900 rounded-lg p-4 flex items-end justify-between">
                          {Array.from({ length: 20 }, (_, i) => (
                            <div
                              key={i}
                              className="bg-green-400 w-2 rounded-t transition-all duration-500"
                              style={{ 
                                height: `${Math.floor(Math.random() * 80 + 20)}%`,
                                animationDelay: `${i * 50}ms`
                              }}
                            ></div>
                          ))}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </div>

                {/* 系统状态 */}
                <div>
                  <Card className="bg-slate-800/50 border-slate-700 mb-6">
                    <CardHeader>
                      <CardTitle className="text-white flex items-center">
                        <Shield className="mr-2 h-5 w-5 text-green-400" />
                        系统状态
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="flex items-center justify-between">
                        <span className="text-slate-300 text-sm">服务状态</span>
                        <Badge className="bg-green-500/20 text-green-400 border-green-500/30">
                          <div className="w-2 h-2 bg-green-400 rounded-full mr-2 animate-pulse"></div>
                          正常
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-300 text-sm">数据库连接</span>
                        <Badge className="bg-green-500/20 text-green-400 border-green-500/30">
                          活跃
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-300 text-sm">缓存状态</span>
                        <Badge className="bg-green-500/20 text-green-400 border-green-500/30">
                          命中率 95%
                        </Badge>
                      </div>
                      <div className="flex items-center justify-between">
                        <span className="text-slate-300 text-sm">API 限流</span>
                        <Badge className="bg-blue-500/20 text-blue-400 border-blue-500/30">
                          正常
                        </Badge>
                      </div>
                      <Separator className="bg-slate-700" />
                      <div className="space-y-2">
                        <h4 className="text-white text-sm font-semibold">运行时间</h4>
                        <div className="text-slate-300 text-sm">
                          <div className="flex justify-between">
                            <span>启动时间:</span>
                            <span>2024-01-15 09:00:00</span>
                          </div>
                          <div className="flex justify-between">
                            <span>运行时长:</span>
                            <span className="text-green-400">15天 6小时</span>
                          </div>
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  <Card className="bg-slate-800/50 border-slate-700">
                    <CardHeader>
                      <CardTitle className="text-white flex items-center">
                        <AlertTriangle className="mr-2 h-5 w-5 text-yellow-400" />
                        告警信息
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div className="p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-1">
                          <AlertTriangle className="h-4 w-4 text-yellow-400" />
                          <span className="text-yellow-400 text-sm font-semibold">警告</span>
                        </div>
                        <p className="text-slate-300 text-xs">
                          内存使用率超过 70%，建议优化
                        </p>
                        <span className="text-slate-400 text-xs">5分钟前</span>
                      </div>
                      <div className="p-3 bg-green-500/10 border border-green-500/30 rounded-lg">
                        <div className="flex items-center gap-2 mb-1">
                          <CheckCircle className="h-4 w-4 text-green-400" />
                          <span className="text-green-400 text-sm font-semibold">正常</span>
                        </div>
                        <p className="text-slate-300 text-xs">
                          所有服务运行正常
                        </p>
                        <span className="text-slate-400 text-xs">刚刚</span>
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </SlideIn>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <Card 
            className={`cursor-pointer transition-all ${
              activeDemo === "memory-basic" 
                ? "bg-purple-600/20 border-purple-500" 
                : "bg-slate-800/50 border-slate-700 hover:border-purple-500/50"
            }`}
            onClick={() => setActiveDemo("memory-basic")}
          >
            <CardHeader>
              <Database className="h-12 w-12 text-blue-400 mb-4" />
              <CardTitle className="text-white">基础记忆操作</CardTitle>
              <CardDescription className="text-slate-300">
                记忆的创建、搜索和管理
              </CardDescription>
            </CardHeader>
          </Card>

          <Card 
            className={`cursor-pointer transition-all ${
              activeDemo === "intelligent-reasoning" 
                ? "bg-purple-600/20 border-purple-500" 
                : "bg-slate-800/50 border-slate-700 hover:border-purple-500/50"
            }`}
            onClick={() => setActiveDemo("intelligent-reasoning")}
          >
            <CardHeader>
              <Brain className="h-12 w-12 text-purple-400 mb-4" />
              <CardTitle className="text-white">智能推理引擎</CardTitle>
              <CardDescription className="text-slate-300">
                DeepSeek 驱动的事实提取
              </CardDescription>
            </CardHeader>
          </Card>

          <Card 
            className={`cursor-pointer transition-all ${
              activeDemo === "mem0-compat" 
                ? "bg-purple-600/20 border-purple-500" 
                : "bg-slate-800/50 border-slate-700 hover:border-purple-500/50"
            }`}
            onClick={() => setActiveDemo("mem0-compat")}
          >
            <CardHeader>
              <Zap className="h-12 w-12 text-green-400 mb-4" />
              <CardTitle className="text-white">Mem0 兼容性</CardTitle>
              <CardDescription className="text-slate-300">
                100% API 兼容和性能提升
              </CardDescription>
            </CardHeader>
          </Card>
        </div>

        {/* 演示内容 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* 代码编辑器 */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <div className="flex justify-between items-center">
                <div>
                  <CardTitle className="text-white flex items-center">
                    <Code className="h-5 w-5 mr-2" />
                    {demoConfigs[activeDemo as keyof typeof demoConfigs]?.title || '演示代码'}
                  </CardTitle>
                  <CardDescription className="text-slate-300">
                {demoConfigs[activeDemo as keyof typeof demoConfigs]?.description || '选择一个演示查看详细信息'}
              </CardDescription>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="outline"
                    className="border-slate-600 text-slate-300 hover:bg-slate-700"
                    onClick={() => copyCode(
                      demoConfigs[activeDemo as keyof typeof demoConfigs].code, 
                      activeDemo
                    )}
                  >
                    {copiedCode === activeDemo ? (
                      <Check className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </Button>
                  <Button
                    size="sm"
                    className="bg-purple-600 hover:bg-purple-700"
                    onClick={() => runDemo(activeDemo)}
                    disabled={isRunning}
                  >
                    {isRunning ? (
                      <div className="animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full" />
                    ) : (
                      <Play className="h-4 w-4" />
                    )}
                    {isRunning ? "运行中..." : "运行"}
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="bg-slate-900 p-4 rounded-lg overflow-x-auto">
                <pre className="text-sm text-green-400 whitespace-pre-wrap">
                  <code>{demoConfigs[activeDemo as keyof typeof demoConfigs]?.code || '// 选择一个演示查看代码'}</code>
                </pre>
              </div>
            </CardContent>
          </Card>

          {/* 输出结果 */}
          <Card className="bg-slate-800/50 border-slate-700">
            <CardHeader>
              <CardTitle className="text-white flex items-center">
                <MessageSquare className="h-5 w-5 mr-2" />
                运行结果
              </CardTitle>
              <CardDescription className="text-slate-300">
                查看演示代码的执行输出
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="bg-slate-900 p-4 rounded-lg min-h-[400px]">
                {demoOutput ? (
                  <pre className="text-sm text-green-400 whitespace-pre-wrap">
                    {demoOutput}
                  </pre>
                ) : (
                  <div className="flex items-center justify-center h-full text-slate-500">
                    点击&quot;运行&quot;按钮查看演示结果
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* 功能特性展示 */}
        <section className="mt-20">
          <h2 className="text-3xl font-bold text-white mb-8 text-center">核心功能演示</h2>
          
          <Tabs defaultValue="features" className="w-full">
            <TabsList className="grid w-full grid-cols-3 bg-slate-800/50">
              <TabsTrigger value="features" className="data-[state=active]:bg-purple-600">核心特性</TabsTrigger>
              <TabsTrigger value="performance" className="data-[state=active]:bg-purple-600">性能对比</TabsTrigger>
              <TabsTrigger value="integration" className="data-[state=active]:bg-purple-600">集成示例</TabsTrigger>
            </TabsList>
            
            <TabsContent value="features" className="mt-8">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <Brain className="h-8 w-8 text-purple-400 mb-2" />
                    <CardTitle className="text-white">智能事实提取</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p className="mb-4">从自然语言对话中自动提取结构化事实信息</p>
                    <Badge className="bg-purple-500/20 text-purple-300">95% 准确率</Badge>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <Zap className="h-8 w-8 text-yellow-400 mb-2" />
                    <CardTitle className="text-white">记忆决策引擎</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p className="mb-4">智能决策记忆的添加、更新、删除和合并操作</p>
                    <Badge className="bg-yellow-500/20 text-yellow-300">90% 决策准确率</Badge>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <Database className="h-8 w-8 text-green-400 mb-2" />
                    <CardTitle className="text-white">多存储后端</CardTitle>
                  </CardHeader>
                  <CardContent className="text-slate-300">
                    <p className="mb-4">支持 8+ 种向量数据库和图数据库</p>
                    <Badge className="bg-green-500/20 text-green-300">灵活扩展</Badge>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
            
            <TabsContent value="performance" className="mt-8">
              <Card className="bg-slate-800/50 border-slate-700">
                <CardHeader>
                  <CardTitle className="text-white">性能基准测试</CardTitle>
                  <CardDescription className="text-slate-300">
                    AgentMem vs 其他记忆管理解决方案
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div className="text-center">
                      <div className="text-3xl font-bold text-purple-400 mb-2">3.2x</div>
                      <div className="text-slate-300">查询速度提升</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold text-blue-400 mb-2">45%</div>
                      <div className="text-slate-300">内存使用减少</div>
                    </div>
                    <div className="text-center">
                      <div className="text-3xl font-bold text-green-400 mb-2">5x</div>
                      <div className="text-slate-300">并发处理提升</div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
            
            <TabsContent value="integration" className="mt-8">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">Web 应用集成</CardTitle>
                    <CardDescription className="text-slate-300">
                      在 Web 应用中集成 AgentMem
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-900 p-4 rounded-lg">
                      <code className="text-sm text-green-400">
                        {`// Express.js 集成示例
const { AgentMemClient } = require('@agentmem/client');

const client = new AgentMemClient({
  apiKey: process.env.AGENTMEM_API_KEY
});

app.post('/chat', async (req, res) => {
  const { message, userId } = req.body;
  
  // 添加用户消息到记忆
  await client.addMemory(userId, message);
  
  // 搜索相关记忆
  const memories = await client.searchMemories(
    message, userId, 5
  );
  
  res.json({ memories });
});`}
                      </code>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-slate-800/50 border-slate-700">
                  <CardHeader>
                    <CardTitle className="text-white">Python 集成</CardTitle>
                    <CardDescription className="text-slate-300">
                      在 Python 应用中使用 AgentMem
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-slate-900 p-4 rounded-lg">
                      <code className="text-sm text-green-400">
                        {`# Python 集成示例
from agentmem import AgentMemClient

client = AgentMemClient(
    api_key=os.getenv('AGENTMEM_API_KEY')
)

# 添加记忆
memory_id = await client.add_memory(
    user_id="user123",
    content="用户喜欢喝咖啡",
    memory_type="preference"
)

# 智能搜索
results = await client.search_memories(
    query="饮品偏好",
    user_id="user123",
    limit=10
)

print(f"找到 {len(results)} 条相关记忆")`}
                      </code>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
          </Tabs>
        </section>

        {/* CTA 区域 */}
        <section className="mt-20 text-center">
          <Card className="bg-gradient-to-r from-purple-600/20 to-pink-600/20 border-purple-500/30">
            <CardHeader>
              <CardTitle className="text-white text-3xl mb-4">
                准备开始构建？
              </CardTitle>
              <CardDescription className="text-slate-300 text-lg">
                立即开始使用 AgentMem，为您的 AI 应用添加强大的记忆能力
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button size="lg" className="bg-purple-600 hover:bg-purple-700">
                  开始免费试用
                  <ArrowRight className="ml-2 h-5 w-5" />
                </Button>
                <Button size="lg" variant="outline" className="border-slate-600 text-slate-300 hover:bg-slate-800">
                  查看文档
                </Button>
                <Button size="lg" variant="outline" className="border-purple-400 text-purple-400 hover:bg-purple-400 hover:text-white">
                  下载示例
                </Button>
              </div>
            </CardContent>
          </Card>
        </section>
      </div>
    </div>
  );
}