# AgentMem Documentation Standards

**Version**: 2.0.0  
**Last Updated**: 2025-01-05

---

## 📋 Overview

This document defines the standards and guidelines for all AgentMem documentation to ensure consistency, quality, and maintainability.

---

## 🎯 Principles

### 1. Clarity
- Use clear, concise language
- Avoid jargon when possible
- Explain technical terms on first use
- Use examples liberally

### 2. Completeness
- Cover all major topics
- Include code examples
- Provide troubleshooting guidance
- Link to related documentation

### 3. Consistency
- Follow naming conventions
- Use consistent formatting
- Maintain uniform structure
- Follow style guide

### 4. Currency
- Keep documentation up-to-date
- Update with code changes
- Remove outdated information
- Version documentation

---

## 📝 Document Structure

### Standard Document Template

```markdown
# Document Title

**Version**: X.Y.Z  
**Last Updated**: YYYY-MM-DD  
**Status**: ✅ Complete / ⚠️ In Progress / 🔜 Planned

---

## 📋 Table of Contents

1. [Section 1](#section-1)
2. [Section 2](#section-2)
3. [Section 3](#section-3)

---

## 🎯 Overview

Brief overview of the document's purpose and scope.

---

## Section 1

Content here...

### Subsection 1.1

More detailed content...

---

## Section 2

Content here...

---

## 🔗 See Also

- [Related Document 1](link)
- [Related Document 2](link)

---

**Last Updated**: YYYY-MM-DD  
**Version**: X.Y.Z
```

---

## 📐 Formatting Standards

### Headers

- **H1** (`#`): Document title only
- **H2** (`##`): Main sections
- **H3** (`###`): Subsections
- **H4** (`####`): Sub-subsections (use sparingly)

### Code Blocks

Always specify language:

````markdown
```rust
// Rust code
let memory = Memory::new().await?;
```

```bash
# Shell commands
cargo build --release
```

```json
{
  "key": "value"
}
```
````

### Inline Code

Use backticks for:
- Function names: `add_memory()`
- Variable names: `user_id`
- File paths: `config.toml`
- Commands: `cargo build`

### Lists

Use ordered lists for steps:
1. First step
2. Second step
3. Third step

Use unordered lists for features:
- Feature 1
- Feature 2
- Feature 3

### Tables

Use tables for structured data:

| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Value 1  | Value 2  | Value 3  |
| Value 4  | Value 5  | Value 6  |

### Emojis

Use emojis sparingly for visual clarity:
- ✅ Complete / Success
- ⚠️ Warning / In Progress
- ❌ Error / Missing
- 🔜 Planned / Coming Soon
- 📝 Note
- 💡 Tip
- 🚀 Quick Start
- 🔧 Configuration

---

## 📚 Content Guidelines

### Code Examples

1. **Always include working examples**
2. **Show complete, runnable code**
3. **Explain what the code does**
4. **Include expected output**
5. **Add error handling where relevant**

Example:

```rust
use agent_mem::Memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new memory instance
    let memory = Memory::new().await?;
    
    // Add a memory
    memory.add("User prefers dark mode").await?;
    
    // Search memories
    let results = memory.search("user preferences").await?;
    
    for result in results {
        println!("- {} (score: {:.2})", result.memory, result.score);
    }
    
    Ok(())
}
```

### Diagrams

Use ASCII art or link to images:

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Server    │
└─────────────┘
```

### Links

- Use descriptive link text
- Link to related documentation
- Use relative paths: `../README.md`
- Verify all links work

### Version Information

Always include:
- Version number
- Last updated date
- Status (if applicable)

---

## 🌐 Language Standards

### Primary Language: English

- Use American English spelling
- Use technical terms consistently
- Define acronyms on first use
- Use active voice when possible

### Secondary Language: Chinese

- Provide Chinese translations for key documents
- Use simplified Chinese
- Maintain consistency with English version

---

## 📂 File Organization

### Naming Conventions

- Use lowercase with hyphens: `api-reference.md`
- Be descriptive: `claude-code-quickstart.md`
- Use consistent prefixes:
  - `quickstart-*.md` for quick starts
  - `guide-*.md` for guides
  - `reference-*.md` for references

### Directory Structure

```
docs/
├── README.md                 # Documentation index
├── api/                      # API documentation
├── architecture/             # Architecture docs
├── deployment/               # Deployment guides
├── getting-started/          # Quick start guides
├── user-guide/               # User documentation
├── developer-guide/          # Developer docs
├── performance/              # Performance docs
└── troubleshooting/          # Troubleshooting
```

---

## ✅ Quality Checklist

Before publishing documentation, ensure:

- [ ] Clear title and overview
- [ ] Table of contents for long documents
- [ ] All code examples work
- [ ] All links are valid
- [ ] Version information included
- [ ] Consistent formatting
- [ ] No typos or grammar errors
- [ ] Related documentation linked
- [ ] Examples are relevant
- [ ] Troubleshooting section (if applicable)

---

## 🔄 Maintenance

### Update Frequency

- **API Documentation**: Update with each API change
- **Architecture**: Update with major changes
- **Guides**: Update quarterly or when needed
- **Examples**: Update when code changes

### Review Process

1. **Technical Review**: Verify accuracy
2. **Editorial Review**: Check grammar and style
3. **User Testing**: Get feedback from users
4. **Final Approval**: Maintainer approval

---

## 📊 Documentation Metrics

Track:
- Total documentation files
- Documentation coverage (%)
- Last update dates
- Broken links
- User feedback

---

## 🎓 Examples

### Good Documentation

✅ Clear title  
✅ Overview section  
✅ Table of contents  
✅ Code examples  
✅ Expected output  
✅ Related links  
✅ Version info  

### Poor Documentation

❌ Vague title  
❌ No overview  
❌ No structure  
❌ No examples  
❌ Outdated information  
❌ Broken links  

---

## 🔗 Related Documents

- [Contributing Guide](../CONTRIBUTING.md)
- [Documentation Index](README.md)
- [Security Documentation](SECURITY.md)

---

**Last Updated**: 2025-01-05  
**Version**: 2.0.0

