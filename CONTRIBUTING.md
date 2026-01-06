# Contributing to AgentMem

Thank you for your interest in contributing to AgentMem! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

## Code of Conduct

Please be respectful and constructive. We aim to maintain a welcoming community.

- Be inclusive and respectful
- Use welcoming and inclusive language
- Be constructive in feedback
- Focus on what is best for the community

## Getting Started

### Prerequisites

- **Rust**: 1.75 or higher
- **Git**: For version control
- **GitHub account**: For PRs and issues

### Development Setup

```bash
# 1. Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/agentmem.git
cd agentmem

# 2. Add upstream remote
git remote add upstream https://github.com/agentmem/agentmem.git

# 3. Install dependencies
cargo build --workspace

# 4. Run tests to verify setup
cargo test --workspace

# 5. Install development tools
cargo install cargo-watch cargo-tarpaulin
```

### Project Structure

```
agentmem/
├── crates/              # Core crates (18 total)
│   ├── agent-mem-traits       # Core abstractions
│   ├── agent-mem-core         # Memory engine
│   ├── agent-mem              # Unified API
│   └── ...
├── examples/            # Example programs
├── tools/               # Development tools
├── docs/                # Documentation
└── tests/               # Integration tests
```

## Development Workflow

### 1. Choose an Issue

Check [GitHub Issues](https://github.com/agentmem/agentmem/issues) for open tasks.

Look for labels:
- `good first issue` - Beginner-friendly
- `help wanted` - Community contributions welcome
- `documentation` - Documentation improvements

### 2. Create a Branch

```bash
# Sync with upstream
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-123
```

#### Branch Naming

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions/changes
- `perf/` - Performance improvements

### 3. Make Changes

```bash
# Make your changes
# ...

# Watch for changes
cargo watch -x check -x test

# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace -- -D warnings

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p agent-mem-core

# Run documentation check
cargo doc --workspace --no-deps
```

### 4. Commit Changes

```bash
git add .
git commit -m "feat: add support for custom search engines"
```

#### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Test additions/changes
- `chore`: Build process or auxiliary tool changes
- `ci`: CI configuration changes

**Example**:
```
feat(llm): add support for DeepSeek API

- Implement DeepSeek client
- Add API key configuration
- Update documentation
- Add integration tests

Closes #123
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Coding Standards

### Rust Guidelines

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

#### Naming

```rust
// ✅ Good: Clear, descriptive names
pub struct MemoryManager { }
pub async fn add_memory(&self, content: &str) -> Result<String> { }

// ❌ Bad: Vague abbreviations
pub struct MemMgr { }
pub async fn add(&self, c: &str) -> Result<String> { }
```

#### Error Handling

```rust
// ✅ Good: Use Result and thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
}

pub async fn get_memory(&self, id: &str) -> Result<Memory, MemoryError> {
    // ...
}

// ❌ Bad: Panic on errors
pub fn get_memory(&self, id: &str) -> Memory {
    self.memories.get(id).unwrap()  // Don't do this!
}
```

#### Async/Await

```rust
// ✅ Good: Use async properly
pub async fn add_memory(&self, content: String) -> Result<Memory> {
    let embedding = self.embed(content).await?;
    self.store.save(embedding).await?;
    Ok(memory)
}

// ❌ Bad: Blocking in async function
pub async fn add_memory(&self, content: String) -> Result<Memory> {
    std::thread::sleep(std::time::Duration::from_secs(1));  // Don't block!
    Ok(memory)
}
```

#### Documentation Comments

```rust
//! Adds a new memory to the store.
//!
//! # Arguments
//!
//! * `content` - The memory content
//! * `scope` - The memory scope
//!
//! # Returns
//!
//! Returns the ID of the created memory.
//!
//! # Errors
//!
//! Returns an error if:
//! - The content is empty
//! - Storage fails
//!
//! # Examples
//!
/// ```
/// use agent_mem::Memory;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let memory = Memory::new().await?;
/// let id = memory.add("Hello, world!").await?;
/// # Ok(())
/// # }
/// ```
pub async fn add(&self, content: &str) -> Result<String> { }
```

## Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_memory() {
        let memory = Memory::new().await.unwrap();
        let id = memory.add("test content").await.unwrap();
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_add_empty_memory_fails() {
        let memory = Memory::new().await.unwrap();
        let result = memory.add("").await;
        assert!(result.is_err());
    }
}
```

### Integration Tests

Create files in `tests/` directory:

```rust
// tests/integration_test.rs
use agent_mem::Memory;

#[tokio::test]
async fn test_full_workflow() {
    let memory = Memory::new().await.unwrap();

    // Add
    let id = memory.add("test").await.unwrap();

    // Get
    let retrieved = memory.get(&id).await.unwrap();
    assert_eq!(retrieved.content, "test");

    // Search
    let results = memory.search("test").await.unwrap();
    assert!(!results.is_empty());
}
```

### Test Coverage

We aim for >95% code coverage:

```bash
# Run tests with coverage
cargo tarpaulin --workspace --out Html

# View coverage report
open tarpaulin-report.html
```

### Performance Tests

Use `criterion` for benchmarks:

```rust
// benches/memory_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use agent_mem::Memory;

fn bench_add_memory(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let memory = rt.block_on(Memory::new()).unwrap();

    c.bench_function("add_memory", |b| {
        b.iter(|| {
            rt.block_on(async {
                memory.add(black_box("test content")).await
            })
        })
    });
}

criterion_group!(benches, bench_add_memory);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench --workspace
```

## Documentation

### API Documentation

Run `cargo doc` to generate API docs:

```bash
# Generate and open docs
cargo doc --workspace --no-deps --open

# Check for broken links
cargo doc --workspace --document-private-items
```

### User Documentation

Update guides in `docs/user-guide/`:
- [Getting Started](docs/user-guide/getting-started.md)
- [Core Concepts](docs/user-guide/core-concepts.md)
- Advanced Features
- Troubleshooting

### Developer Documentation

Update guides in `docs/developer-guide/`:
- [Architecture](docs/developer-guide/architecture.md)
- Development Setup
- Plugin Development

## Pull Request Process

### PR Checklist

Before submitting a PR, ensure:

- [ ] Code follows style guidelines (`cargo fmt`)
- [ ] Code passes clippy (`cargo clippy`)
- [ ] Tests pass (`cargo test`)
- [ ] Documentation updated (if needed)
- [ ] Commit messages follow format
- [ ] PR description clearly explains changes

### PR Description Template

```markdown
## Description
Brief description of the changes and why they're needed.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] All tests pass: `cargo test --workspace`

## Checklist
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have tested my changes locally
- [ ] I have added tests that prove my fix is effective or that my feature works

## Related Issues
Fixes #123
Related to #456
```

### Review Process

1. **Automated Checks**: CI runs tests, linters, and formatting checks
2. **Code Review**: Maintainers review your code (may take 1-3 days)
3. **Address Feedback**: Make requested changes
4. **Approval**: At least one maintainer approval required
5. **Merge**: Squash and merge to main branch

## Release Process

Releases are managed by maintainers following semantic versioning:

1. Update version in `Cargo.toml` (root workspace)
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v2.0.0`
4. Push tag: `git push origin v2.0.0`
5. GitHub Actions creates release automatically
6. Publish to crates.io: `cargo publish`

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time chat (see README for link)

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://doc.rust-lang.org/book/ch10-00-generics.html)

### Asking Questions

1. Search existing issues and discussions first
2. Check the documentation
3. Create a new issue with the "question" label
4. Be specific and provide context

## Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file in the repository
- Release notes for significant contributions
- GitHub contributor list

All contributors retain copyright of their contributions.

## License

By contributing to AgentMem, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

---

Thank you for contributing to AgentMem! 🎉

Your contributions help make AgentMem better for everyone.
