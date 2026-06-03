# Contributing to AI Disk Doctor

Thank you for your interest in contributing to AI Disk Doctor! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, constructive, and inclusive in all interactions.

## How to Contribute

### Reporting Issues

Before creating an issue, please:

1. Search existing issues to avoid duplicates
2. Use the appropriate issue template if available
3. Provide as much context as possible:
   - Windows version
   - Rust version (`rustc --version`)
   - Command used and full output
   - Expected vs actual behavior

### Suggesting Features

Feature suggestions are welcome! Please:

1. Open an issue with the `enhancement` label
2. Describe the use case and expected behavior
3. Discuss implementation approach if you have one

### Contributing Code

#### Development Setup

```bash
# Clone your fork
git clone git@github.com:YOUR_USERNAME/ai-disk-doctor.git
cd ai-disk-doctor

# Build
cd aidisk
cargo build

# Run tests
cargo test

# Run linting (if available)
cargo clippy
```

#### Project Structure

```
.
├── aidisk/              # Rust CLI crate
│   ├── src/             # Source code
│   ├── tests/           # Integration tests
│   ├── config/          # Default rules and policy
│   └── fixtures/        # Test fixtures
├── docs/                # Documentation
├── scripts/             # Utility scripts
└── skills/              # Agent skill definitions
```

#### Coding Standards

- **Rust**: Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- **Tests**: All new features must include tests
- **Documentation**: Update README.md and relevant docs for user-facing changes
- **Safety**: New cleanup rules default to `careful` or `dangerous`; `safe` requires strong justification

#### Commit Message Convention

We follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

Examples:
```
feat(scanner): add node_modules cache detection
fix(cleaner): handle cross-disk quarantine fallback
docs(readme): update installation instructions
test(rules): add playground for custom rules
```

#### Pull Request Process

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make your changes with tests
3. Ensure all tests pass: `cargo test`
4. Commit with clear messages
5. Push to your fork
6. Open a Pull Request against `master`
7. Fill out the PR template completely
8. Address review feedback promptly

#### PR Checklist

- [ ] Tests added/updated and passing
- [ ] Documentation updated (README, docs/, comments)
- [ ] No breaking changes (or clearly documented)
- [ ] Code follows project style
- [ ] Commit messages follow convention

### Adding New Rules

Rules are defined in YAML. To add a new detection rule:

1. Identify the category (browser-cache, ai-model, dev-tool, etc.)
2. Determine risk level:
   - `safe`: Known temp/cache, easily recreated
   - `careful`: User data that might be needed
   - `dangerous`: System-critical, never auto-clean
3. Add rule to `aidisk/config/rules/` or submit to community rules repo
4. Include test fixture demonstrating the path pattern

Example rule:
```yaml
name: "My Tool Cache"
category: "dev-tool"
risk: "safe"
patterns:
  - "%USERPROFILE%\\.mytool\\cache"
```

### Adding New Doctor Topics

1. Add topic flag to CLI (in `src/main.rs`)
2. Implement analyzer in `src/doctor.rs`
3. Add rules for detection paths
4. Update README with example command
5. Add test for topic output

## Release Process

Maintainers only:

1. Update `CHANGELOG.md`
2. Update version in `aidisk/Cargo.toml`
3. Create release notes in `docs/release-notes/`
4. Tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
5. Push: `git push origin vX.Y.Z`
6. Create GitHub Release with notes

## Questions?

Open a Discussion or reach out in issues. We're happy to help!

## License

By contributing, you agree that your contributions will be licensed under the same dual license as the project (MIT/Apache-2.0).
