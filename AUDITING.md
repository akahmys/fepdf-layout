# 🛡️ `fepdf-layout` Security, License & Compliance Auditing Protocol

This document defines the automated audit checks, license policy, security vulnerability management, and secret protection standards for `fepdf-layout`.

---

## 🔍 1. Audit Framework Overview

`fepdf-layout` enforces an automated compliance pipeline:

```
1. RR-15 Rules          2. Clippy Lints           3. License Audit       4. Secret Scan
(Line limits, panic,    (-D warnings,             (cargo-deny via        (betterleaks /
 unsafe, BTreeMap)       pedantic/nursery)         deny.toml)             pre-commit)
```

---

## 📜 2. License Compliance Protocol (`cargo-deny`)

All workspace crates and third-party dependencies are audited using **`cargo-deny`** against the project's license policy.

### Allowed License List (Permissive & Weak Copyleft)
- **Primary**: `MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`
- **BSD Family**: `BSD-3-Clause`, `BSD-2-Clause`, `BSD-1-Clause`, `0BSD`
- **Public Domain / Permissive**: `CC0-1.0`, `Unlicense`, `ISC`, `BSL-1.0`, `Zlib`, `MIT-0`
- **Fonts & Special**: `OFL-1.1`, `Ubuntu-font-1.0`, `Unicode-3.0`, `MPL-2.0`

### Forbidden Licenses
- Strong copyleft licenses (e.g., `GPL-2.0`, `GPL-3.0`, `AGPL-3.0`) are strictly **denied** (`copyleft = "deny"`).

---

## 🔐 3. Secret & PII Protection Protocol

To prevent accidental leaks of credentials, private keys, API tokens, and Personally Identifiable Information (PII):

- **Git Pre-commit Hook**: Automated scanning for high-entropy tokens and private keys before commit.
- **Directories Scanned**: Codebase, docs, and test assets.

---

## 🛠️ 4. Static Compliance Audit

Checks enforced prior to merging code:
1. Absence of `unwrap`/`expect` in production crates (Rule 2).
2. Zero `unsafe` blocks (Rule 3).
3. Zero `static mut` (Rule 7).
4. No `HashMap`/`HashSet` in core layout pipeline (Rule 10).
5. Zero Clippy warnings (`cargo clippy --workspace -- -D warnings`).
6. Pass `cargo deny check licenses`.
