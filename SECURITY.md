# Security Policy

## Supported Versions

Currently, only the latest version of AgentMem is receiving security updates.

| Version | Supported          |
|---------|--------------------|
| 2.x.x   | :white_check_mark: Yes |
| 1.x.x   | :x: No              |

## Reporting a Vulnerability

The AgentMem team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report

**DO NOT** open a public issue for security vulnerabilities.

Instead, please send an email to: **security@agentmem.dev**

Please include:
- A description of the vulnerability
- Steps to reproduce the issue
- Affected versions (if known)
- Potential impact
- Suggested mitigation (if any)

### What Happens Next?

1. **Acknowledgment** (within 48 hours)
   - You will receive an email acknowledging receipt of your report
   - We may ask for additional information

2. **Investigation** (within 7 days)
   - We will investigate the vulnerability
   - We will determine the severity and impact

3. **Resolution** (varies by severity)
   - We will develop a fix
   - We will coordinate release with you

4. **Disclosure**
   - You will be notified when the fix is released
   - We will credit you in the release notes (unless you prefer anonymity)

### Security Updates

Security updates will be:
- Released as patch versions (e.g., 2.0.1 → 2.0.2)
- Announced in the [CHANGELOG.md](CHANGELOG.md)
- Published on GitHub Releases with the "security" label

## Security Best Practices

### For Users

1. **Keep Updated**
   - Always use the latest version
   - Subscribe to [GitHub Releases](https://github.com/louloulin/agentmem/releases)
   - Monitor the [CHANGELOG.md](CHANGELOG.md)

2. **Secure Configuration**
   ```rust
   use agent_mem::{Memory, Config};

   let config = Config::builder()
       .with_encryption(true)  // Enable encryption at rest
       .with_authentication(AuthConfig::default())
       .build();

   let memory = Memory::new(config).await?;
   ```

3. **Input Validation**
   - Always validate user input before storing
   - Sanitize data from untrusted sources
   - Use parameterized queries to prevent injection

4. **Access Control**
   - Implement proper RBAC (Role-Based Access Control)
   - Use API keys for authentication
   - Rotate credentials regularly

### For Developers

1. **Code Review**
   - All security-related changes require 2 approvers
   - Security-sensitive code must be documented
   - Use `unsafe` sparingly and document why it's necessary

2. **Dependency Management**
   - Keep dependencies up-to-date
   - Review security advisories for dependencies
   - Use `cargo audit` regularly

3. **Testing**
   - Include security tests in PRs
   - Test for common vulnerabilities (injection, XSS, etc.)
   - Use fuzzing for security-critical code

4. **Secrets Management**
   - Never commit secrets to git
   - Use environment variables for configuration
   - Rotate secrets regularly

## Security Features

AgentMem includes several security features:

### Encryption at Rest
- SQLite database encryption support
- Configurable encryption keys
- Secure key storage via OS keychain

### Authentication
- API key authentication
- JWT token support
- OAuth 2.0 integration (enterprise)

### Authorization
- Role-based access control (RBAC)
- User/Agent isolation
- Multi-tenancy support

### Audit Logging
- Comprehensive audit trail
- Access logging
- Change tracking

## Common Vulnerabilities and Mitigations

### SQL Injection

**Vulnerable**:
```rust
// DON'T DO THIS
let query = format!("SELECT * FROM memories WHERE content = '{}'", user_input);
```

**Secure**:
```rust
// DO THIS
let query = sqlx::query("SELECT * FROM memories WHERE content = $1")
    .bind(user_input)
    .fetch_all(pool)
    .await?;
```

### Path Traversal

**Vulnerable**:
```rust
// DON'T DO THIS
let path = format!("{}/{}", base_dir, user_input);
```

**Secure**:
```rust
// DO THIS
use std::path::Path;

let path = Path::new(base_dir).join(user_input);
if !path.starts_with(base_dir) {
    return Err("Invalid path".into());
}
```

### Denial of Service

**Mitigations**:
- Rate limiting
- Request size limits
- Timeout enforcement
- Resource quotas

```rust
let config = Config::builder()
    .with_max_request_size(10_000_000)  // 10 MB
    .with_rate_limit(RateLimit::per_minute(100))
    .build();
```

## Dependency Vulnerabilities

We use GitHub Dependabot and GitHub Actions to monitor dependencies:

- **Dependabot**: Automatically opens PRs for vulnerable dependencies
- **Security Workflow**: Runs `cargo audit` on every PR
- **Monthly Reviews**: Manual review of dependency updates

### Audit Your Installation

```bash
# Check for vulnerabilities
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Security Audits

| Date | Auditor | Scope | Report |
|------|---------|-------|--------|
| TBD | TBD | Core library, plugins, server | TBD |

*If you're interested in sponsoring a security audit, please contact security@agentmem.dev*

## Security Contact Information

- **Email**: security@agentmem.dev
- **PGP Key**: [To be added]
- **Security Policy**: This document

## Related Resources

- [SECURITY.md on GitHub](https://github.com/louloulin/agentmem/blob/main/SECURITY.md)
- [Reporting vulnerabilities securely](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities)
- [CVE Mitigation](https://cve.mitre.org/)

---

*Last updated: 2025-01-05*
