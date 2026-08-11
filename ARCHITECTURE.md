# 🏛️ `fepdf-layout` Architecture & System Design Specification

The authoritative architectural and functional blueprint for **fepdf-layout**: page layout engine, interactive canvas, and `fepdf` integration.

> **Target Concept Notice.** This application specifically targets **帳票作成 (Business & Standardized Forms Creation)** (invoices, receipts, application forms, certificates). It prioritizes clean, deterministic 1mm grid-based layout precision over complex or bloated layout wizard automation.

---

## 📐 1. Layer Topology & Design Principles

```
┌──────────────────────────────────────────────────────────────────────┐
│  【View 層】 fepdf-layout-ui  (egui + wgpu / GUI / Panels / Canvas)   │
│  - パネル構成 (左: プロパティ, 中央: 作業スペース, 右: パーツパレット)      │
│  - UI 一時状態 (CanvasViewState: ドラッグ中プレビュー, 選択枠, ズーム)     │
└──────────────────────────────────┬───────────────────────────────────┘
                                   │ Translates UI events into Commands
┌──────────────────────────────────▼───────────────────────────────────┐
│  【Logic 層】 fepdf-layout-core (Layout Engine / Document Model)     │
│  - ドメインモデル (Single Page, 1mm-Grid Snapped Items, Bottom-Left)  │
│  - コマンド実行機 ＆ Undo/Redo スタック (Command History)              │
│  - 完全な egui 非依存 (純粋な Rust ユニットテスト可能)                 │
└──────────────────────────────────┬───────────────────────────────────┘
                                   │ Maps to fepdf Operations & Render calls
┌──────────────────────────────────▼───────────────────────────────────┐
│  【Engine 層】 fepdf Facade  (Document, Page, PDF 2.0 Output)         │
└──────────────────────────────────────────────────────────────────────┘
```

### Rule 1 — Strict Logic & View Separation
`fepdf-layout-core` contains **zero GUI (`egui`) dependencies**. All document state and calculations are pure Rust data structures and deterministic functions.

### Rule 2 — Command-Driven Document Mutations
All layout modifications are executed via serializable `Command` objects, enabling robust Undo/Redo (`CommandHistory`) and macro execution.

### Rule 3 — Facade-Only Engine Integration
`fepdf-layout` communicates with `fepdf` exclusively through its public facade (`fepdf` crate) and `Operation` vocabulary.

---

## 📄 2. Document & Page Specifications

1. **ドキュメント構造**: 初期フェーズは **1ページ作成（帳票フォーマット）** に限定。
2. **座標系**: 全レイヤーで **左下原点 (Bottom-Left Origin $(0, 0)$)** に統一。
   - $X$ 軸: 右方向へ増加 ($X > 0$)
   - $Y$ 軸: 上方向へ増加 ($Y > 0$)
3. **単位系**: 全 UI 表示・入力・データ管理を **`mm`（ミリメートル）のみ** に統一。
4. **グリッド ＆ スナップ**:
   - キャンバス上に 1mm 格子グリッドを表示。
   - すべてのパーツの位置 $(X, Y)$ およびサイズ $(W, H)$ は **1mm 単位の整数に強制スナップ**。
5. **用紙規格 ＆ アクティブ領域**:

| 規格名 | 物理用紙サイズ ($W_{\text{paper}} \times H_{\text{paper}}$) | アクティブレイアウト領域 ($W_{\text{layout}} \times H_{\text{layout}}$) | 左下原点オフセット ($\text{offset}_x, \text{offset}_y$) |
| :--- | :--- | :--- | :--- |
| **A4 (デフォルト)** | $210 \times 297 \text{ mm}$ | **$210 \times 297 \text{ mm}$** | $(0.0, 0.0) \text{ mm}$ |
| **A3** | $297 \times 420 \text{ mm}$ | **$297 \times 420 \text{ mm}$** | $(0.0, 0.0) \text{ mm}$ |
| **A5** | $148 \times 210 \text{ mm}$ | **$148 \times 210 \text{ mm}$** | $(0.0, 0.0) \text{ mm}$ |
| **B5 (JIS)** | $182 \times 257 \text{ mm}$ | **$182 \times 257 \text{ mm}$** | $(0.0, 0.0) \text{ mm}$ |
| **B4 (JIS)** | $257 \times 364 \text{ mm}$ | **$257 \times 364 \text{ mm}$** | $(0.0, 0.0) \text{ mm}$ |
| **Letter** | $215.9 \times 279.4 \text{ mm}$ | **$215 \times 279 \text{ mm}$** | $(0.45, 0.20) \text{ mm}$ |
| **Legal** | $215.9 \times 355.6 \text{ mm}$ | **$215 \times 355 \text{ mm}$** | $(0.45, 0.30) \text{ mm}$ |
| **Custom** | 入力値 $W \times H \text{ mm}$ | **$W \times H \text{ mm}$** | $(0.0, 0.0) \text{ mm}$ |

---

## 🧩 3. Element / Component Specifications

配置可能なパーツは以下の 6 種に限定（無駄な複雑化を排し、1mm 格子による直接配置に特化）：

1. **直線 (`Line`)**: 始点 $(X_1, Y_1) \to$ 終点 $(X_2, Y_2)$（線の太さ・色・スタイル）
2. **テキストボックス (`TextBox`)**:
   - 配置 $(X, Y)$、サイズ $(W, H)$、文章・フォント・サイズ・文字色
   - **文字揃え (`Text Alignment`)**: 左揃え / 中央揃え / 右揃え / **均等割付 (`Justify`)**
   - **文字横比率 (`Horizontal Scaling`)**: 初期値 `100%`（長体・平体設定）
   - **自動枠合わせ (`Auto-Fit Horizontal`)**: 枠幅に合わせた自動長体調整
3. **PDF フォームフィールド (`FormField`)**:
   - **テキストフィールド (`TextField`)**: 文字・数値入力枠
   - **チェックボックス (`CheckBox`)**: ON/OFF チェック
   - **ラジオボタン (`RadioButton`)**: グループ内単一選択
   - **ドロップダウン (`ComboBox`)**: リスト選択
   - **フォームデータタグ (`field_tag`)**: **自動集計・データ抽出用の識別子タグ（データキー）**を設定可能
   - **フォームデザイン属性**: 枠線色、枠線幅、背景色、フォントサイズ・文字色を編集可能

---

## 🖥️ 4. UI Panel Layout & Interactivity

- **画面配置**:
  - **左パネル**: プロパティインスペクター（選択パーツの属性数値編集 ＆ 複数選択時の整列ボタン）
  - **中央スペース**: 作業キャンバス（1mm グリッド、定規、ズーム・パン、アクティブページ描画）
  - **右パネル**: パーツパレット（パーツ一覧からのドラッグ＆ドロップ配置）
- **複数選択 ＆ 整列機能**:
  - 囲み選択 / `Shift` + クリック
  - 左揃え、左右中央揃え、右揃え、下揃え、上下中央揃え、上揃え（すべて 1mm 整数座標へスナップ）

---

## 🛡️ 5. Safety & Quality Control

- **Zero Unsafe**: `unsafe_code = "forbid"` across all workspace crates.
- **Licensing**: MIT License ([`LICENSE`](file:///Users/akahmys/projects/fepdf-layout/LICENSE)).
- **Security Audit**: Automated pre-commit hook via `betterleaks` (detecting leaks including personal names) & `cargo-deny` license checks.
