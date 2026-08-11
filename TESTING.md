# 🧪 `fepdf-layout` Testing & Validation Strategy

This document details the testing methodology, test suites, and quality assurance processes for `fepdf-layout`.

---

## 🎯 1. Test Pyramid & Separation Policy

`fepdf-layout` employs a 3-tier testing hierarchy with strict **Test Code Separation**:

```
┌──────────────────────────────────────────────┐
│ 1. Canvas Interactive & Integration Tests    │ (crates/*/tests/*.rs)
├──────────────────────────────────────────────┤
│ 2. Core Layout & Command Unit Tests          │ (#[cfg(test)] in src/)
└──────────────────────────────────────────────┘
```

### 📁 Test Code Separation Guidelines
1. **Integration & Large Test Suites (`crates/*/tests/`)**:
   - Multi-frame scenarios, document import/export end-to-end tests MUST be located in the crate's root `tests/` directory.
   - Do NOT place standalone test files inside `src/`.
2. **Inline Unit Tests (`src/`)**:
   - Small, private helper unit tests reside alongside production code inside `#[cfg(test)] mod tests { ... }` blocks at the bottom of `src/` files.

---

## 🧪 2. Workspace Unit & Integration Tests

Run all unit & integration tests across the workspace:

```bash
cargo test --workspace
```

---

## 🦀 3. Minimum Supported Rust Version (MSRV) Verification

The project requires **Rust 1.94+ (Edition 2024)**. MSRV compatibility is verified via:

```bash
cargo check --workspace
```

---

## 🛠️ 4. Pre-Merge Quality Checklist

Before submitting a Pull Request or completing a major task:

- [ ] `cargo test --workspace` passes with 0 failures.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean.
- [ ] `cargo fmt --all --check` reports no diff.
- [ ] All integration tests are placed in `crates/*/tests/` following the Test Separation Policy.
- [ ] `cargo deny check licenses` passes without license violations.
