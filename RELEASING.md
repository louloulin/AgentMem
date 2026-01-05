# AgentMem Release Process

This document describes the process for releasing AgentMem.

## Table of Contents

- [Version Numbers](#version-numbers)
- [Release Types](#release-types)
- [Pre-Release Checklist](#pre-release-checklist)
- [Release Process](#release-process)
- [Post-Release Tasks](#post-release-tasks)
- [Emergency Releases](#emergency-releases)

---

## Version Numbers

AgentMem follows [Semantic Versioning 2.0.0](https://semver.org/).

### Version Format

```
MAJOR.MINOR.PATCH (e.g., 2.1.0)
```

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

### Pre-Release Versions

```
VERSION-pre-release.N (e.g., 2.1.0-rc.1)
```

Pre-release identifiers:
- `alpha`: Early development, not feature-complete
- `beta`: Feature-complete, testing in progress
- `rc`: Release candidate, final testing phase

---

## Release Types

### Major Release (e.g., 2.0 → 3.0)

**Frequency**: Every 6-12 months
**Examples**:
- New architecture version
- Breaking API changes
- Major feature additions

**Process**:
1. Create `release-x.x` branch from `main`
2. Update `CHANGELOG.md` with new features
2. Update all documentation
4. Beta testing period (4-6 weeks)
5. Release candidates (2-3 RC versions)
6. Final release

### Minor Release (e.g., 2.0 → 2.1)

**Frequency**: Every 2-3 months
**Examples**:
- New features (backwards compatible)
- New storage backends
- New embedding models

**Process**:
1. Create `release-x.x.x` branch from `main`
2. Feature freeze
3. Testing period (2 weeks)
4. Release candidate (optional)
5. Final release

### Patch Release (e.g., 2.1.0 → 2.1.1)

**Frequency**: As needed
**Examples**:
- Bug fixes
- Security patches
- Documentation updates

**Process**:
1. Create `release-x.x.x` branch from `main`
2. Apply fixes
3. Quick testing (3-5 days)
4. Release

---

## Pre-Release Checklist

### Code Quality

- [ ] All tests passing: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] Code formatted: `cargo fmt --all -- --check`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] Audit passes: `cargo audit`

### Test Coverage

- [ ] Coverage ≥ 60%: `cargo tarpaulin --fail-under 60`
- [ ] Integration tests passing
- [ ] E2E tests passing
- [ ] Performance benchmarks passing

### Documentation

- [ ] CHANGELOG.md updated
- [ ] README.md updated (if needed)
- [ ] API documentation complete
- [ ] Migration guide (for breaking changes)
- [ ] Release notes drafted

### CI/CD

- [ ] All workflows passing on `main`
- [ ] Release workflow configured
- [ ] Artifacts building correctly
- [ ] Deployment scripts tested

### Security

- [ ] Dependency audit clean: `cargo audit`
- [ ] No high-severity vulnerabilities
- [ ] Security policy reviewed
- [ ] Breaking changes documented

---

## Release Process

### 1. Create Release Branch

```bash
# Checkout main
git checkout main
git pull upstream main

# Create release branch
git checkout -b release-x.x.x

# Example:
# git checkout -b release-2.1.0
```

### 2. Update Version Numbers

```bash
# Use cargo-set-version to update workspace
cargo install cargo-set-version

# Update all workspace members
cargo set-version --workspace 2.1.0

# Verify versions updated
git diff
```

### 3. Update CHANGELOG

```markdown
## [2.1.0] - 2025-01-15

### Added
- WASM plugin system for custom memory processors
- Multimodal memory support (images, audio, video)
- Graph memory network for entity relationships

### Changed
- Improved search performance by 40%
- Updated memory deduplication algorithm
- Enhanced embedding model integration

### Fixed
- Fixed memory leak in long-running processes
- Fixed concurrent access issues
- Fixed PostgreSQL connection pooling

### Deprecated
- `MemoryItem` type (use `MemoryV4` instead)

### Removed
- Legacy V1 API (removed in 2.0)
- Old embedding model interfaces

### Security
- Updated dependencies to fix vulnerabilities
```

### 4. Update Documentation

- [ ] Update README.md with new features
- [ ] Update API documentation
- [ ] Create migration guide for breaking changes
- [ ] Update examples
- [ ] Update architecture docs

### 5. Final Testing

```bash
# Run full test suite
cargo test --workspace --all-features

# Run benchmarks
cargo bench

# Test release build
cargo build --release

# Test installation
cargo install --path .
```

### 6. Commit Changes

```bash
# Stage version changes
git add Cargo.toml **/Cargo.toml
git add CHANGELOG.md
git add README.md
git add docs/

# Commit
git commit -m "chore: prepare release 2.1.0"
```

### 7. Create Git Tag

```bash
# Create annotated tag
git tag -a v2.1.0 -m "Release v2.1.0

Features:
- WASM plugin system
- Multimodal memory support
- Graph memory networks
- Performance improvements

See CHANGELOG.md for details"
```

### 8. Push to GitHub

```bash
# Push release branch
git push upstream release-2.1.0

# Push tag
git push upstream v2.1.0
```

### 9. GitHub Release

1. Go to GitHub Releases page
2. Click "Draft a new release"
3. Choose tag `v2.1.0`
4. Write release notes (copy from CHANGELOG)
5. Attach binaries (built by CI)
6. Publish release

### 10. Publish to Crates.io

```bash
# Login to crates.io (if needed)
cargo login

# Publish in dependency order
cargo publish -p agent-mem-traits
cargo publish -p agent-mem-utils
cargo publish -p agent-mem-config
cargo publish -p agent-mem-core
cargo publish -p agent-mem
# ... publish other crates

# Verify publishing
cargo search agent-mem
```

---

## Post-Release Tasks

### 1. Merge Back to Main

```bash
# Merge release branch to main
git checkout main
git merge release-2.1.0
git push upstream main
```

### 2. Create Maintenance Branch

For major/minor releases, create a maintenance branch:

```bash
# Create branch from release tag
git checkout -b maintain-2.1 v2.1.0
git push upstream maintain-2.1
```

### 3. Update Main Branch

```bash
# Bump version to next development version
cargo set-version --workspace 2.2.0-alpha.0

# Commit version bump
git commit -m "chore: bump version to 2.2.0-alpha.0"
git push upstream main
```

### 4. Announce Release

- [ ] Post on GitHub Discussions
- [ ] Update Discord
- [ ] Tweet release notes
- [ ] Update website
- [ ] Send newsletter (if applicable)

### 5. Monitor Issues

- [ ] Watch for regression issues
- [ ] Respond to release feedback
- [ ] Track post-release metrics

---

## Emergency Releases

For critical security issues or severe bugs:

### Emergency Release Process

1. **Quick Assessment** (1 hour)
   - Verify severity
   - Identify affected versions
   - Determine fix approach

2. **Prepare Fix** (4-8 hours)
   - Create `hotfix-x.x.x` branch
   - Implement minimal fix
   - Add tests for the bug

3. **Test Thoroughly** (2-4 hours)
   - Run full test suite
   - Test specific scenario
   - Verify no regressions

4. **Quick Release** (1-2 hours)
   - Update version (PATCH increment)
   - Create git tag
   - Publish to crates.io
   - Create GitHub release

5. **Announcement** (1 hour)
   - Security advisory (if applicable)
   - Migration guide (if needed)
   - Update all channels

### Example Timeline

```
09:00 - Security issue reported
10:00 - Severity confirmed, hotfix branch created
14:00 - Fix implemented and tested
16:00 - Fix verified, tests passing
17:00 - Release published
18:00 - Announcement sent
```

---

## Release Cadence

### Planned Releases

| Type | Frequency | Next Scheduled |
|------|-----------|----------------|
| Major | 6-12 months | Q3 2025 (v3.0) |
| Minor | 2-3 months | Q1 2025 (v2.2) |
| Patch | As needed | As needed |

### Release Calendar

```plaintext
Q4 2024:
  - v2.1.0 (October)
  - v2.1.1 (November - patch)

Q1 2025:
  - v2.2.0-beta.1 (January)
  - v2.2.0 (February)

Q2 2025:
  - v3.0.0-alpha.1 (April)
  - v3.0.0-beta.1 (May)
  - v3.0.0 (June)
```

---

## Release Communication

### Release Notes Template

```markdown
# AgentMem vX.X.X Release

## 🎉 What's New

[Highlight major features]

## ✨ Features

- [Feature 1]
- [Feature 2]

## 🐛 Bug Fixes

- [Fix 1]
- [Fix 2]

## 🔧 Breaking Changes

[If applicable]

## 📚 Migration Guide

[Link to migration guide]

## 📦 Installation

```bash
cargo add agent-mem
```

## 🙏 Thanks

[Thank contributors]

## 🔗 Links

- [Documentation](https://docs.agentmem.dev)
- [GitHub](https://github.com/agentmem/agentmem)
- [Discord](https://discord.gg/agentmem)
```

### Announcement Channels

1. **GitHub Release** - Primary announcement
2. **Discord** - Community discussion
3. **Twitter** - Public announcement
4. **GitHub Discussions** - Feature discussion
5. **Blog** (if applicable) - Detailed writeup

---

## Release Metrics

Track these metrics for each release:

- [ ] Release date vs. planned date
- [ ] Number of issues fixed
- [ ] Number of new features
- [ ] Test coverage percentage
- [ ] Binary size
- [ ] Performance benchmarks
- [ ] Download stats (after 1 week, 1 month)
- [ ] Issue reports (regressions)

---

## Troubleshooting

### Common Issues

**Issue**: Tests failing on release branch
**Solution**: Fix on release branch, do not merge from main

**Issue**: Version conflict in dependencies
**Solution**: Update `Cargo.toml` locking file

**Issue**: Crates.io publish fails
**Solution**: Check for already published version, clean registry cache

**Issue**: GitHub Actions timeout
**Solution**: Increase timeout or split workflow

---

## Additional Resources

- [Semantic Versioning](https://semver.org/)
- [Cargo Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [CHANGELOG.md](CHANGELOG.md) - Example format

---

*For questions about releases, please open a GitHub discussion or contact the maintainers.*
