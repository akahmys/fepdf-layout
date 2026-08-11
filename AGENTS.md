# 🤖 fepdf-layout Agentic Governance & System Architecture

Welcome to **fepdf-layout**, a high-fidelity PDF 2.0 based DTP & Page Layout application built with Rust and `fepdf`. This project operates under an AI-native autonomous engineering model.

---

## 🏛️ Governance Architecture & Document Structure

The project rules, architecture specs, and operational protocols are modularized into core documents:

| Document | Focus & Scope | Description |
| :--- | :--- | :--- |
| 📘 **[AGENTS.md](AGENTS.md)** | **Constitution & Governance** | System vision, truth hierarchy, decision framework, and entry point. |
| 🏛️ **[ARCHITECTURE.md](ARCHITECTURE.md)** | **System Design & Layering Rules** | UI/Canvas layer, DTP layout engine, `fepdf` integration, and state management. |
| 📋 **[PLANNING.md](PLANNING.md)** | **Planning & Roadmap** | Feature roadmap, UI/UX specifications, and task breakdown. |
| 💻 **[CODING.md](CODING.md)** | **Coding Rules** | Rust coding standards, state mutation rules, and error handling. |

---

## ⚖️ Hierarchy of Truth

When conflicting directives arise, agents and contributors MUST resolve ambiguities using the following strict hierarchy:

```
1. ISO 32000-2:2020 & fepdf Facade Specifications
   └── 2. fepdf-layout Core Architecture Specs (ARCHITECTURE.md)
        └── 3. Primary Governance Docs (AGENTS, PLANNING, CODING)
             └── 4. Codebase Implementation & Workspace Crates
```

---

## 🎯 Core Operating Principles

1. **Responsive & Fluid Interaction**: High frame-rate interactive canvas powered by `fepdf` GPU rendering backend.
2. **Deterministic Layout & Document State**: All page layout modifications must map to reversible operations or local component state.
3. **Safety & Zero Unsafe**: `unsafe_code = "forbid"` is enforced across all crates.
4. **Clean Abstraction Over Core**: `fepdf` storage abstractions (`PdfArena`, `Handle`) are isolated beneath the facade boundary.
