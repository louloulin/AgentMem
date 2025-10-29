import { TranslationKeys } from '@/lib/i18n';

/**
 * 中文翻译
 */
export const zh: TranslationKeys = {
  common: {
    loading: '加载中...',
    error: '错误',
    success: '成功',
    cancel: '取消',
    confirm: '确认',
    save: '保存',
    delete: '删除',
    edit: '编辑',
    view: '查看',
    search: '搜索',
    searchPlaceholder: '搜索文档、功能和更多内容...',
    noResults: '未找到相关结果',
    backToTop: '回到顶部',
    copyCode: '复制代码',
    copied: '已复制',
    download: '下载',
    expand: '展开',
    collapse: '收起'
  },
  
  nav: {
    home: '首页',
    docs: '文档',
    demo: '演示',
    about: '关于',
    pricing: '定价',
    blog: '博客',
    support: '支持',
    github: 'GitHub',
    language: '语言',
    theme: '主题'
  },
  
  home: {
    title: 'AgentMem',
    subtitle: '下一代智能记忆管理平台',
    description: '基于 Rust 构建的高性能记忆管理系统，集成 DeepSeek 推理引擎，为 AI 代理提供强大的记忆能力。',
    getStarted: '开始使用',
    viewDocs: '查看文档',
    learnMore: '了解更多',
    features: {
      title: '核心功能',
      subtitle: '为 AI 代理提供全面的记忆管理解决方案',
      items: {
        memory: {
          title: '智能记忆管理',
          description: '高效存储和检索 AI 代理的记忆数据，支持复杂的记忆结构和关联关系。'
        },
        search: {
          title: '语义搜索',
          description: '基于向量数据库的语义搜索，快速找到相关记忆内容，提升 AI 代理的响应效率。'
        },
        reasoning: {
          title: 'DeepSeek 推理',
          description: '集成 DeepSeek 推理引擎，提供强大的逻辑推理和知识关联能力。'
        },
        api: {
          title: 'RESTful API',
          description: '完整的 API 接口，支持多种编程语言，轻松集成到现有系统中。'
        },
        performance: {
          title: '高性能',
          description: '基于 Rust 构建，提供卓越的性能和内存安全保障，支持大规模并发访问。'
        },
        security: {
          title: '安全可靠',
          description: '企业级安全保障，支持数据加密、访问控制和审计日志等安全功能。'
        }
      }
    },
    stats: {
      users: '活跃用户',
      stars: 'GitHub Stars',
      downloads: '下载量',
      uptime: '系统可用性'
    },
    testimonials: {
      title: '用户评价',
      subtitle: '来自全球开发者的真实反馈'
    },
    cta: {
      title: '准备开始了吗？',
      subtitle: '立即体验 AgentMem 的强大功能，为您的 AI 代理提供智能记忆能力。',
      button: '免费开始'
    }
  },
  
  docs: {
    title: '文档',
    subtitle: '完整的开发指南和 API 参考',
    quickStart: {
      title: '快速开始',
      description: '几分钟内快速上手 AgentMem，了解基本概念和核心功能。'
    },
    tutorials: {
      title: '教程指南',
      description: '详细的教程和最佳实践，帮助您充分利用 AgentMem 的功能。'
    },
    api: {
      title: 'API 参考',
      description: '完整的 API 文档，包含所有接口的详细说明和示例代码。'
    },
    examples: {
      title: '代码示例',
      description: '丰富的代码示例和使用场景，快速理解和应用 AgentMem。'
    }
  },
  
  demo: {
    title: '在线演示',
    subtitle: '体验 AgentMem 的强大功能',
    interactive: {
      title: '交互式演示',
      description: '通过实际操作了解 AgentMem 的核心功能和使用方法。'
    },
    examples: {
      title: '使用示例',
      description: '查看不同场景下的实际应用案例和最佳实践。'
    }
  },
  
  about: {
    title: '关于我们',
    subtitle: '了解 AgentMem 团队和我们的使命',
    company: {
      title: '公司介绍',
      description: 'AgentMem 致力于为 AI 代理提供最先进的记忆管理解决方案，推动人工智能技术的发展。'
    },
    mission: {
      title: '我们的使命',
      description: '通过创新的记忆管理技术，让 AI 代理更加智能和高效，为人类创造更大的价值。'
    },
    team: {
      title: '团队介绍',
      description: '我们是一支充满激情的技术团队，专注于人工智能和系统架构领域。'
    },
    technology: {
      title: '技术架构',
      description: '基于现代化的技术栈，构建高性能、可扩展的记忆管理平台。'
    }
  },
  
  pricing: {
    title: '定价方案',
    subtitle: '选择适合您需求的方案',
    free: {
      title: '免费版',
      price: '¥0',
      description: '适合个人开发者和小型项目',
      features: [
        '基础记忆管理',
        '1GB 存储空间',
        '1000 次/月 API 调用',
        '社区支持',
        '基础文档'
      ],
      button: '免费开始'
    },
    pro: {
      title: '专业版',
      price: '¥99/月',
      description: '适合中小型企业和团队',
      features: [
        '高级记忆管理',
        '100GB 存储空间',
        '100,000 次/月 API 调用',
        '优先技术支持',
        '完整文档和教程',
        '数据分析面板'
      ],
      button: '选择专业版'
    },
    enterprise: {
      title: '企业版',
      price: '联系我们',
      description: '适合大型企业和定制需求',
      features: [
        '无限记忆管理',
        '无限存储空间',
        '无限 API 调用',
        '专属技术支持',
        '定制开发服务',
        '私有化部署',
        'SLA 保障'
      ],
      button: '联系销售'
    }
  },
  
  blog: {
    title: '博客',
    subtitle: '最新的技术分享和产品动态',
    readMore: '阅读更多',
    publishedOn: '发布于',
    author: '作者',
    tags: '标签',
    categories: '分类'
  },
  
  support: {
    title: '支持中心',
    subtitle: '获取帮助和技术支持',
    faq: {
      title: '常见问题',
      description: '查找常见问题的解答和解决方案。'
    },
    contact: {
      title: '联系我们',
      description: '有问题或建议？我们很乐意为您提供帮助。',
      form: {
        name: '姓名',
        email: '邮箱',
        subject: '主题',
        message: '消息',
        submit: '发送消息'
      }
    },
    community: {
      title: '社区支持',
      description: '加入我们的社区，与其他开发者交流经验。'
    }
  },
  
  footer: {
    description: 'AgentMem 是基于 Rust 构建的下一代智能记忆管理平台，为 AI 代理提供强大的记忆能力。',
    links: {
      product: '产品',
      resources: '资源',
      company: '公司',
      legal: '法律'
    },
    copyright: '© 2024 AgentMem. 保留所有权利。'
  }
};