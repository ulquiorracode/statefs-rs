# Contributing to StateFS (`statefs-rs`)

Thank you for your interest in contributing to **`statefs-rs`**! We welcome contributions of all kinds: virtual filesystem providers, performance optimizations, bug fixes, architecture improvements, documentation, and ecosystem adapters.

Please take a moment to review these guidelines before submitting code.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Branching Strategy](#branching-strategy)
- [Conventional Commits](#conventional-commits)
- [Development Setup](#development-setup)
- [Coding Standards & Core Invariants](#coding-standards--core-invariants)
- [Pull Request Workflow](#pull-request-workflow)

---

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

---

## Branching Strategy

Our repository strictly adheres to the standard GitFlow-inspired branching strategy:

- **`main`**: Production/stable releases only. Protected branch; direct commits are strictly prohibited.
- **`dev`**: Active development integration branch. Protected branch; all feature, fix, and dependency PRs must target `dev`.
- **`feature/<name>`**: New features and enhancements, branched from `dev` and merged back into `dev` via PR.
- **`fix/<name>`**: Bug fixes, branched from `dev` (or `main` for critical production hotfixes).

```text
main   ───────────────────────────● (v0.1.1)
                                 /
dev    ──────●──────────●───────●─── (Active Development)
              \        /
feature/vfs    ●──────●             (Feature Branch)
```

---

## Conventional Commits

Commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification:

```text
<type>(<optional-scope>): <description in imperative mood>

[optional body]

[optional footer(s)]
```

### Allowed Types

| Type | Description | Example |
| :--- | :--- | :--- |
| `feat` | A new feature or capability | `feat(arch): implement Port-Adapter-Provider triad for VFS` |
| `fix` | A bug fix | `fix(vfs): correct radix tree path traversal edge on trailing slash` |
| `perf` | A code change that improves performance | `perf(wal): zero-allocation drain wal ring buffer` |
| `refactor` | Code change that neither fixes a bug nor adds a feature | `refactor(config): separate statefs-config into standalone crate` |
| `docs` | Documentation changes | `docs(architecture): document virtual inode layout` |
| `test` | Adding or updating tests | `test(fuzz): add adversarial boundary and truncation tests` |
| `build` | Changes to build configuration or dependencies | `build(deps): update workspace dependencies` |
| `ci` | Changes to CI/CD pipeline | `ci: add matrix test runners` |
| `chore` | Routine maintenance tasks | `chore: update repository licenses` |

---

## Development Setup

### Prerequisites

- **Rust**: Rust 2024 Edition (MSRV: `1.85.0` or newer stable).
- **Cargo Tools**:

  ```bash
  rustup component add clippy rustfmt
  ```

### Local Verification Suite

Run the full local verification pipeline before submitting any PR:

```bash
# 1. Format check
cargo fmt --all -- --check

# 2. Strict Clippy check with all targets and features
cargo clippy --workspace --all-targets --all-features -- -D warnings

# 3. Comprehensive unit and integration tests
cargo test --workspace --all-targets --all-features

# 4. Strict #![no_std] verification for core engine
cargo check --no-default-features -p statefs-core

# 5. Documentation build check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
```

---

## Coding Standards & Core Invariants

1. **Nanokernel Purity (`#![no_std]`)**: `statefs-core` is strictly `#![no_std]` and zero-allocation by default.
2. **Dense Contiguous Memory**: Store virtual nodes and index maps in cacheline-aligned contiguous slices.
3. **Canonical Triad (Port - Adapter - Provider)**:
   - **Port**: Pure abstractions and interfaces defined by the engine.
   - **Adapter**: Integration glue for external libraries and network boundaries (Axum, Tower, etc.).
   - **Provider**: Storage, configuration, or telemetry backend implementations.
4. **Mechanical Sympathy**: Eliminate memory fragmentation; keep lock scopes narrow and poison-resilient.
5. **No Silent Error Suppression**: Explicitly return typed domain `Result<T, E>`. Never mask errors with lazy default fallbacks.

---

## Pull Request Workflow

1. Fork or branch from `dev`.
2. Implement changes surgically following the architectural patterns and coding invariants.
3. Verify all checks pass locally using the commands listed in [Local Verification Suite](#local-verification-suite).
4. Open a Pull Request targeting `dev` with structured summary, change details, and verification evidence.
