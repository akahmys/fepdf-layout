# 📋 `fepdf-layout` Planning & Discovery Protocol

This document governs task planning, codebase exploration, feature design, and decision-making workflows within the `fepdf-layout` project.

---

## 🎯 1. Planning Workflow

Before embarking on significant structural changes, new features, or refactoring, create or update an **Implementation Plan** (`implementation_plan.md`).

### Implementation Plan Structure
1. **Goal Description**: Clear scope, rationale, and target outcomes for the feature or refactor.
2. **User Review Required**: Document breaking changes, architectural choices, or design trade-offs requiring user confirmation.
3. **Open Questions**: Unresolved requirements, layout calculation edge cases, or UX design decisions.
4. **Proposed Changes**: Grouped logically by crate/component with `[NEW]`, `[MODIFY]`, or `[DELETE]` annotations.
5. **Verification Plan**: Automated tests (`cargo test`), visual regression/rendering checks, and manual verification steps.

---

## 🔍 2. Codebase Discovery Protocol

Never guess implementation details, data schemas, or file locations. Follow this exploration protocol:

1. **Log-First Diagnostics**: Inspect full error tracebacks and empirical logs before forming diagnostic hypotheses.
2. **Complete Symbol Inspection**: View full struct, enum, and trait definitions in `fepdf` or `fepdf-layout` rather than truncated code snippets.
3. **Registry & Dependency Audit**: Check crate manifests (`Cargo.toml`), workspace dependencies, and public API exports (`lib.rs`).

---

## 🔄 3. Workflows & Session Management

- Task states and progress are tracked in `.agents/session/` artifacts (`task.md`, `walkthrough.md`).
- After completing work, update `walkthrough.md` with:
  - Concise summary of changes made.
  - Verification results (test outputs, UI/canvas render checks, compliance logs).
