# Support Resources

获取帮助和支持资源

---

## 📚 Self-Service Resources

### Documentation

- **[Quick Start Guide](QUICKSTART.md)** - 5 minutes to get started
- **[User Guides](docs/guides/)** - Detailed feature guides
- **[API Reference](docs/api/)** - Complete API documentation
- **[FAQ](FAQ.md)** - Frequently asked questions
- **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and solutions

### Examples & Tutorials

- **[Examples](examples/)** - Code examples in multiple languages
- **[Tutorials](docs/tutorials/)** - Step-by-step tutorials
- **[Integration Guides](docs/integration/)** - Third-party integrations

---

## 🆘 Getting Help

### Community Support (Free)

**GitHub Discussions**
- 📝 [Discussions](https://github.com/agentmem/agentmem/discussions)
- 💬 Best for: Questions, ideas, discussions
- ⏰ Response time: 24-48 hours
- 💰 Cost: Free

**GitHub Issues**
- 🐛 [Issue Tracker](https://github.com/agentmem/agentmem/issues)
- 💬 Best for: Bug reports, feature requests
- ⏰ Response time: Varies
- 💰 Cost: Free

**Discord Community**
- 💬 [Join Discord](https://discord.gg/agentmem)
- 💬 Best for: Real-time chat, community help
- ⏰ Response time: Often immediate
- 💰 Cost: Free

**Stack Overflow**
- ❓ [Tag: agentmem](https://stackoverflow.com/questions/tagged/agentmem)
- 💬 Best for: Technical Q&A
- ⏰ Response time: Varies
- 💰 Cost: Free

### Professional Support (Paid)

**Enterprise Support**
- 📧 Email: enterprise@agentmem.dev
- 📞 SLA: 4-hour response time
- 🔧 Features: Dedicated support, custom patches
- 💰 Pricing: Contact sales

**Consulting Services**
- 📧 Email: consulting@agentmem.dev
- 🔧 Services: Architecture design, implementation, optimization
- 💰 Pricing: Project-based

**Training & Workshops**
- 📧 Email: training@agentmem.dev
- 📚 Options: Online training, on-site workshops
- 💰 Pricing: Per attendee

---

## 🐛 Reporting Issues

### Bug Reports

Before reporting a bug:

1. **Search existing issues**
   - Check [GitHub Issues](https://github.com/agentmem/agentmem/issues)
   - Search [FAQ](FAQ.md)
   - Search [Discussions](https://github.com/agentmem/agentmem/discussions)

2. **Gather information**
   - AgentMem version
   - Operating system and version
   - Rust version (if applicable)
   - Minimal reproduction code
   - Error messages and stack traces

3. **Submit a bug report**
   - Use the [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.md)
   - Include all gathered information
   - Be specific and concise

**Example Bug Report**:
```markdown
## Bug Description
Search returns empty results when using Chinese text.

## Reproduction Steps
1. Add memory: "我喜欢喝咖啡"
2. Search: "咖啡"
3. Result: Empty (expected: Memory found)

## Environment
- AgentMem: v2.0.0
- OS: Ubuntu 22.04
- Rust: 1.75.0
- Embedding: all-MiniLM-L6-v2

## Error Message
No errors, but unexpected empty results.
```

### Feature Requests

We welcome feature requests!

1. **Check existing requests**
   - Search [GitHub Issues](https://github.com/agentmem/agentmem/issues)
   - Label: `enhancement`

2. **Submit a feature request**
   - Use the [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.md)
   - Describe the use case
   - Explain why it's important
   - Suggest a solution (if you have ideas)

**Example Feature Request**:
```markdown
## Feature Description
Add support for Neo4j as a graph database backend.

## Use Case
I need to use AgentMem with an existing Neo4j database.

## Importance
Our enterprise already uses Neo4j for other services.

## Proposed Solution
Add a Neo4j storage backend similar to the PostgreSQL backend.
```

---

## 🚨 Security Issues

### Reporting Security Vulnerabilities

**DO NOT** open a public issue for security vulnerabilities.

**How to Report**:
1. **Email**: security@agentmem.dev
2. **PGP Key**: [Download](https://agentmem.dev/pgp-key.asc) (optional)
3. **Response Time**: 48 hours

**What to Include**:
- Vulnerability description
- Steps to reproduce (if applicable)
- Affected versions
- Potential impact
- Suggested mitigation (if known)

**What Happens Next**:
1. We acknowledge receipt (within 48 hours)
2. We investigate and assess severity
3. We develop a fix
4. We coordinate disclosure with you
5. We publish security update

**Security Updates**:
- Announced in [CHANGELOG.md](CHANGELOG.md)
- Published as patch releases
- Tagged with `security` label on GitHub

---

## 💬 Community Guidelines

### Code of Conduct

Please follow our [Code of Conduct](CODE_OF_CONDUCT.md):
- Be respectful and inclusive
- Welcome newcomers
- Focus on what is best for the community
- Show empathy towards other community members

### Getting the Best Help

**Do**:
- ✅ Search before asking
- ✅ Provide context and details
- ✅ Share minimal reproduction code
- ✅ Be patient and polite
- ✅ Share solutions when you find them

**Don't**:
- ❌ Demand immediate responses
- ❌ Post the same question in multiple places
- ❌ Use all caps or aggressive language
- ❌ Expect others to do your work for you
- ❌ Post sensitive information (API keys, passwords)

### Asking Good Questions

**Good Question**:
```markdown
Hi! I'm trying to use AgentMem with LangChain, but I'm getting
an authentication error when adding memories.

## What I've tried:
1. Followed the LangChain integration guide
2. Verified API keys are correct
3. Tested with the Python SDK directly (works)

## Code:
```python
from agentmem import Memory

memory = Memory(api_key="sk-...")
memory.add("Test")  # This works
```

## Error:
```
AuthenticationError: Invalid API key
```

## Environment:
- AgentMem: v2.0.0
- Python: 3.11
- OS: Ubuntu 22.04

Any suggestions on what I'm missing?
```

**Poor Question**:
```
It doesn't work. Help!
```

---

## 📅 Office Hours (Community)

### Weekly Office Hours

- **When**: Every Wednesday, 2:00 PM - 3:00 PM UTC
- **Where**: [Zoom Link](https://zoom.us/agentmem-office-hours)
- **What**: Live Q&A with maintainers
- **Cost**: Free

### Monthly Community Call

- **When**: First Thursday of every month, 5:00 PM - 6:00 PM UTC
- **Where**: [YouTube Live](https://youtube.com/@agentmem)
- **What**: Project updates, roadmap discussion, Q&A
- **Cost**: Free

---

## 📖 Additional Resources

### Learning Resources

- **Blog**: [https://blog.agentmem.dev](https://blog.agentmem.dev)
- **YouTube Channel**: [AgentMem TV](https://youtube.com/@agentmem)
- **Documentation**: [https://docs.agentmem.dev](https://docs.agentmem.dev)
- **Examples**: [github.com/agentmem/examples](https://github.com/agentmem/examples)

### Integration Examples

- **LangChain**: [Integration Guide](docs/integration/LANGCHAIN_INTEGRATION.md)
- **LlamaIndex**: [Integration Guide](docs/integration/LLAMAINDEX_INTEGRATION.md)
- **LumosAI**: [Integration Guide](docs/integration/LUMOSAI_INTEGRATION.md)
- **华为 MAAS**: [Integration Guide](docs/integration/HUAWEI_MAAS_INTEGRATION.md)

### Contributing

- **Contributing Guide**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Good First Issues**: [Label: good-first-issue](https://github.com/agentmem/agentmem/labels/good%20first%20issue)
- **Roadmap**: [ky0.1.md](ky0.1.md)

---

## 📞 Emergency Contacts

### Critical Production Issues

For **critical production issues** affecting your business:
- 📧 **Email**: emergency@agentmem.dev
- ⏰ **Response**: 2 hours (24/7)
- 💰 **Enterprise customers only**

### Press & Media Inquiries

- 📧 **Email**: press@agentmem.dev
- ⏰ **Response**: 24 hours

### Business Inquiries

- **Sales**: sales@agentmem.dev
- **Partnerships**: partners@agentmem.dev
- **Careers**: jobs@agentmem.dev

---

## 🙏 Acknowledgments

Thank you to our amazing community!
- All contributors who submit PRs and issues
- Everyone who answers questions on Discord and Stack Overflow
- Users who provide feedback and suggestions
- Organizations that use and support AgentMem

---

## 📊 Support Metrics

We aim to respond to all inquiries within:
- GitHub Issues: 48 hours
- GitHub Discussions: 48 hours
- Discord: 24 hours (often faster)
- Email: 24 hours (enterprise: 4 hours)

**Current average response times** (last 30 days):
- GitHub Issues: ~18 hours
- GitHub Discussions: ~12 hours
- Discord: ~2 hours
- Email: ~8 hours

---

*Need help? Start with the [FAQ](FAQ.md) or [Documentation](docs/).*

---

*Last updated: 2025-01-05*
