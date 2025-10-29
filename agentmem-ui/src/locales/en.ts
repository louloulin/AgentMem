import { TranslationKeys } from '@/lib/i18n';

/**
 * English translations
 */
export const en: TranslationKeys = {
  common: {
    loading: 'Loading...',
    error: 'Error',
    success: 'Success',
    cancel: 'Cancel',
    confirm: 'Confirm',
    save: 'Save',
    delete: 'Delete',
    edit: 'Edit',
    view: 'View',
    search: 'Search',
    searchPlaceholder: 'Search docs, features and more...',
    noResults: 'No results found',
    backToTop: 'Back to Top',
    copyCode: 'Copy Code',
    copied: 'Copied',
    download: 'Download',
    expand: 'Expand',
    collapse: 'Collapse'
  },
  
  nav: {
    home: 'Home',
    docs: 'Docs',
    demo: 'Demo',
    about: 'About',
    pricing: 'Pricing',
    blog: 'Blog',
    support: 'Support',
    github: 'GitHub',
    language: 'Language',
    theme: 'Theme'
  },
  
  home: {
    title: 'AgentMem',
    subtitle: 'Next-Generation Intelligent Memory Management Platform',
    description: 'High-performance memory management system built with Rust, integrated with DeepSeek reasoning engine, providing powerful memory capabilities for AI agents.',
    getStarted: 'Get Started',
    viewDocs: 'View Docs',
    learnMore: 'Learn More',
    features: {
      title: 'Core Features',
      subtitle: 'Comprehensive memory management solutions for AI agents',
      items: {
        memory: {
          title: 'Intelligent Memory Management',
          description: 'Efficiently store and retrieve AI agent memory data, supporting complex memory structures and associations.'
        },
        search: {
          title: 'Semantic Search',
          description: 'Vector database-based semantic search to quickly find relevant memory content and improve AI agent response efficiency.'
        },
        reasoning: {
          title: 'DeepSeek Reasoning',
          description: 'Integrated DeepSeek reasoning engine providing powerful logical reasoning and knowledge association capabilities.'
        },
        api: {
          title: 'RESTful API',
          description: 'Complete API interface supporting multiple programming languages, easily integrated into existing systems.'
        },
        performance: {
          title: 'High Performance',
          description: 'Built with Rust, providing exceptional performance and memory safety, supporting large-scale concurrent access.'
        },
        security: {
          title: 'Secure & Reliable',
          description: 'Enterprise-grade security with data encryption, access control, and audit logging capabilities.'
        }
      }
    },
    stats: {
      users: 'Active Users',
      stars: 'GitHub Stars',
      downloads: 'Downloads',
      uptime: 'System Uptime'
    },
    testimonials: {
      title: 'User Testimonials',
      subtitle: 'Real feedback from developers worldwide'
    },
    cta: {
      title: 'Ready to Get Started?',
      subtitle: 'Experience the powerful features of AgentMem and provide intelligent memory capabilities for your AI agents.',
      button: 'Start Free'
    }
  },
  
  docs: {
    title: 'Documentation',
    subtitle: 'Complete development guide and API reference',
    quickStart: {
      title: 'Quick Start',
      description: 'Get up and running with AgentMem in minutes, learn basic concepts and core features.'
    },
    tutorials: {
      title: 'Tutorials',
      description: 'Detailed tutorials and best practices to help you make full use of AgentMem features.'
    },
    api: {
      title: 'API Reference',
      description: 'Complete API documentation with detailed descriptions and example code for all interfaces.'
    },
    examples: {
      title: 'Code Examples',
      description: 'Rich code examples and use cases to quickly understand and apply AgentMem.'
    }
  },
  
  demo: {
    title: 'Live Demo',
    subtitle: 'Experience the powerful features of AgentMem',
    interactive: {
      title: 'Interactive Demo',
      description: 'Learn about AgentMem core features and usage through hands-on experience.'
    },
    examples: {
      title: 'Use Cases',
      description: 'View real-world application examples and best practices in different scenarios.'
    }
  },
  
  about: {
    title: 'About Us',
    subtitle: 'Learn about the AgentMem team and our mission',
    company: {
      title: 'Company Overview',
      description: 'AgentMem is dedicated to providing the most advanced memory management solutions for AI agents, driving the development of artificial intelligence technology.'
    },
    mission: {
      title: 'Our Mission',
      description: 'Through innovative memory management technology, make AI agents smarter and more efficient, creating greater value for humanity.'
    },
    team: {
      title: 'Our Team',
      description: 'We are a passionate technical team focused on artificial intelligence and system architecture.'
    },
    technology: {
      title: 'Technology Stack',
      description: 'Built on modern technology stack to create a high-performance, scalable memory management platform.'
    }
  },
  
  pricing: {
    title: 'Pricing Plans',
    subtitle: 'Choose the plan that fits your needs',
    free: {
      title: 'Free',
      price: '$0',
      description: 'Perfect for individual developers and small projects',
      features: [
        'Basic memory management',
        '1GB storage space',
        '1,000 API calls/month',
        'Community support',
        'Basic documentation'
      ],
      button: 'Get Started Free'
    },
    pro: {
      title: 'Professional',
      price: '$19/month',
      description: 'Ideal for small to medium businesses and teams',
      features: [
        'Advanced memory management',
        '100GB storage space',
        '100,000 API calls/month',
        'Priority technical support',
        'Complete documentation and tutorials',
        'Analytics dashboard'
      ],
      button: 'Choose Professional'
    },
    enterprise: {
      title: 'Enterprise',
      price: 'Contact Us',
      description: 'Perfect for large enterprises and custom requirements',
      features: [
        'Unlimited memory management',
        'Unlimited storage space',
        'Unlimited API calls',
        'Dedicated technical support',
        'Custom development services',
        'Private deployment',
        'SLA guarantee'
      ],
      button: 'Contact Sales'
    }
  },
  
  blog: {
    title: 'Blog',
    subtitle: 'Latest technical insights and product updates',
    readMore: 'Read More',
    publishedOn: 'Published on',
    author: 'Author',
    tags: 'Tags',
    categories: 'Categories'
  },
  
  support: {
    title: 'Support Center',
    subtitle: 'Get help and technical support',
    faq: {
      title: 'FAQ',
      description: 'Find answers to frequently asked questions and solutions.'
    },
    contact: {
      title: 'Contact Us',
      description: 'Have questions or suggestions? We\'d love to help you.',
      form: {
        name: 'Name',
        email: 'Email',
        subject: 'Subject',
        message: 'Message',
        submit: 'Send Message'
      }
    },
    community: {
      title: 'Community Support',
      description: 'Join our community to exchange experiences with other developers.'
    }
  },
  
  footer: {
    description: 'AgentMem is a next-generation intelligent memory management platform built with Rust, providing powerful memory capabilities for AI agents.',
    links: {
      product: 'Product',
      resources: 'Resources',
      company: 'Company',
      legal: 'Legal'
    },
    copyright: '© 2024 AgentMem. All rights reserved.'
  }
};