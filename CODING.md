# 💻 `fepdf-layout` Coding Standards & Hardening Protocol

This document defines the coding conventions, safety standards (**RR-15 Protocol**), and architectural patterns required across all crates in the `fepdf-layout` workspace.

---

## 🛡️ 1. The RR-15 Hardening Rules

Derived from aerospace safety principles, the **RR-15 (Reliable Rust-15)** rules guarantee determinism, memory safety, and absolute runtime reliability.

### Rule Summary Matrix

| Rule | Area | Requirement | Enforcement |
| :--- | :--- | :--- | :--- |
| **Rule 1** | Function Length | Max 50 lines for standard functions.<br>Max 200 lines for GUI component handlers.<br>Max 500 lines for Canvas dispatchers. | Static analysis / Linter |
| **Rule 2** | Panic Prevention | `unwrap()` and `expect()` are forbidden in production code. Use `?` or `unwrap_or()`. | Automated grep check |
| **Rule 3** | Unsafe Ban | `unsafe` blocks are forbidden (`workspace.lints.rust.unsafe_code = "forbid"`). | Rustc lint |
| **Rule 4** | Control Flow | Avoid deep nesting (`if let` / `match`). Prefer early return with `?`. | Code review / Clippy |
| **Rule 5** | Match Exhaustiveness | Wildcard arms (`_ =>`) are forbidden when matching a **domain enum** (e.g., ToolMode, LayoutUnit, FrameKind). | Clippy / Compiler |
| **Rule 6** | Stack Safety | Unbounded recursion is forbidden. Use heap-based loops with `Vec`. | Code review |
| **Rule 7** | Global State | `static mut` and global mutable state are forbidden. | Automated grep check |
| **Rule 8** | Invalid State | Use type-safe `enum` states instead of boolean flags or nested `Option`s. | Architecture review |
| **Rule 10** | Determinism | `HashMap` and `HashSet` are forbidden in core layout & document models. Use `BTreeMap` or `BTreeSet`. | Code review / Grep check |
| **Rule 11** | Error Transparency | Return typed `thiserror` enums. String-based errors (`Result<T, String>`) are forbidden in core APIs. | Compiler / Code review |
| **Rule 13** | Error Swallowing | `filter_map(Result::ok)` and silent error swallowing are forbidden. | Code review |
| **Rule 14** | Test Code Separation | Standalone/Integration tests MUST be placed in `crates/*/tests/`. Do NOT pollute `src/` with dedicated test files. | Directory structure check |
| **Rule 15** | Clone Optimization | Avoid excessive `.clone()`. Use `Arc` or handle references where appropriate. | Code review |
| **Rule 17** | Type Explicitly | Explicitly specify floating-point types (`1.0_f32`, `2.5_f64`) to prevent Edition 2024 inference fallbacks. | Clippy / Compiler |

---

## 🏛️ 2. DTP & Canvas Architecture

### State & Canvas Interactivity
1. **Interactive Canvas State**: Selection handles, bounding box transformations, alignment guides, and zoom levels are kept in local GUI/view state.
2. **Command Reversibility**: Any modification affecting the page content (e.g., frame creation, resize, text edits) must generate an Undoable `Command`.
3. **Precision**: Use double-precision `f64` for layout coordinates, snapping, and grid calculations.

---

## 🎨 3. UI Framework & `fepdf` Integration

- **Integration Boundary**: Never access `fepdf` internal pools or arenas directly; use `fepdf` public facade types (`Document`, `Page`, `Operation`).
- **Render Cache**: Cache rendered PDF page textures for smooth panning/zooming on the interactive canvas.
