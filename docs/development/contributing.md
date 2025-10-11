# Contributing to AgentDB

We welcome contributions to AgentDB! This document provides guidelines for contributing to the project.

## 🚀 Getting Started

### Prerequisites
- Rust 1.70+
- Zig 0.14.0
- Git

### Development Setup
```bash
# Fork and clone the repository
git clone https://github.com/louloulin/agent-db.git
cd agent-db

# Build the project
cargo build --release
zig build

# Run tests
cargo test --lib
zig build test-all

# Generate C bindings
cargo run --bin generate_bindings
```

## 📋 How to Contribute

### 1. Reporting Issues
- Use the [GitHub issue tracker](https://github.com/your-org/agent-db/issues)
- Provide detailed information about the bug or feature request
- Include steps to reproduce for bugs
- Use appropriate labels

### 2. Code Contributions

#### Pull Request Process
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Update documentation if needed
7. Commit with clear messages
8. Push to your fork
9. Create a pull request

#### Code Style
- **Rust**: Follow standard Rust formatting (`cargo fmt`)
- **Zig**: Follow Zig style guidelines
- **Comments**: Use clear, descriptive comments
- **Documentation**: Update docs for public APIs

#### Testing Requirements
- All new features must include tests
- Maintain 90%+ test coverage
- Run full test suite before submitting
- Include both unit and integration tests

### 3. Documentation
- Update README files for significant changes
- Add inline documentation for new APIs
- Update examples if APIs change
- Ensure documentation builds without warnings

## 🔧 Development Guidelines

### Code Quality
- Write clean, readable code
- Follow existing patterns and conventions
- Use meaningful variable and function names
- Keep functions focused and small

### Performance
- Consider performance implications of changes
- Run benchmarks for performance-critical code
- Profile code when making optimizations

### Security
- Follow secure coding practices
- Validate all inputs
- Handle errors gracefully
- Review security implications of changes

## 📝 Commit Guidelines

### Commit Message Format
```
type(scope): description

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `perf`: Performance improvements

### Examples
```
feat(rag): add hybrid search functionality

Add support for combining text and semantic search
with configurable weighting parameters.

Closes #123
```

## 🧪 Testing

### Running Tests
```bash
# Rust tests
cargo test --lib

# Zig tests
zig build test

# Stress tests
cargo test stress_test --lib

# Benchmarks
cargo test benchmark --lib
```

### Test Categories
- **Unit Tests**: Test individual functions/modules
- **Integration Tests**: Test component interactions
- **Stress Tests**: Test under high load
- **Benchmark Tests**: Performance measurements

## 📚 Resources

### Documentation
- [Architecture Overview](docs/architecture.md)
- [API Reference](docs/api.md)
- [Performance Guide](PERFORMANCE_REPORT.md)

### Communication
- GitHub Issues for bug reports and feature requests
- GitHub Discussions for questions and ideas
- Pull Request reviews for code discussions

## 🏆 Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes for significant contributions
- Project documentation

## 📄 License

By contributing to AgentDB, you agree that your contributions will be licensed under the MIT License.

## ❓ Questions?

If you have questions about contributing, please:
1. Check existing documentation
2. Search existing issues
3. Create a new issue with the "question" label
4. Join our community discussions

Thank you for contributing to AgentDB! 🚀
