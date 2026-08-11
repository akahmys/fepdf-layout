# 📋 `fepdf-layout` 段階的実装計画 (PLANS.md)

本ドキュメントは、[`PLANNING.md`](file:///Users/akahmys/projects/fepdf-layout/PLANNING.md) の策定プロトコルおよび [`ARCHITECTURE.md`](file:///Users/akahmys/projects/fepdf-layout/ARCHITECTURE.md) の確定仕様に厳格に準拠した段階的実装計画書です。

---

## 🎯 1. Goal Description (開発目標)

帳票（請求書・申請書・証明書等）の作成に特化した **1ページ完結型・1mm格子単位スナップ DTP レイアウトエディタ** を構築します。
`fepdf-layout-core`（ロジック層）の完全 GUI 非依存設計と Command パターンによる Undo/Redo を確立し、`fepdf-layout-ui`（`egui` + `wgpu`）による左下原点キャンバス操作および 3 パネルインタフェースを段階的に実装します。

---

## ⚠️ 2. User Review Required (要確認・合意事項)

> [!NOTE]
> 1. **段階的開発**: コアロジック (`core` の 1mm スナップ・コマンド・Undo/Redo・ユニットテスト) を先行して完成・検証した後に、UI 層 (`ui` の `egui` キャンバス・パレット) の構築へ進みます。
> 2. **完全安全コード**: `unsafe_code = "forbid"` を全フェーズで遵守します。

---

## ❓ 3. Open Questions (検討・確認事項)

> [!NOTE]
> 現時点で未解決の検討事項はありません。すべての仕様（1mm 格子、左下原点、帳票ターゲット、全 6 種パーツ、均等割付 `Justify`、フォームデータタグ `field_tag`）は [`ARCHITECTURE.md`](file:///Users/akahmys/projects/fepdf-layout/ARCHITECTURE.md) に集約・確定済みです。

---

## 🛠️ 4. Proposed Changes (実装タスク分解)

---

### Phase 1: コアレイアウトモデル ＆ 1mm 単位系 (`fepdf-layout-core`)

#### [NEW] [units.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/units.rs)
- `Mm(u32)` 型および `MmF64(f64)` 型の定義。
- PDF PostScript Point ($1\text{ mm} = 72/25.4\text{ pt}$) への変換ロジック。

#### [NEW] [page.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/page.rs)
- `PagePreset` 列挙型 (A3, A4, A5, B4, B5, Letter, Legal, Custom)。
- 物理用紙寸法、1mm 整数アクティブ領域、原点オフセット ($\text{offset}_x, \text{offset}_y$) の算出構造体 `PageSpec`。

#### [NEW] [element.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/element.rs)
- ユニーク要素 ID `ElementId(u64)`。
- `LineElement`: 始点・終点 $(X_1, Y_1) \to (X_2, Y_2)$ (1mm 単位), 太さ, 色, スタイル。
- `TextBoxElement`: 位置 $(X, Y)$、サイズ $(W, H)$ (1mm 単位), テキスト内容, 文字揃え (`Left`, `Center`, `Right`, `Justify`), `horizontal_scaling` (%), `auto_fit_horizontal` (bool)。
- `FormFieldElement`: 種別 (`TextField`, `CheckBox`, `RadioButton`, `ComboBox`), データ集計用タグ `field_tag`, 位置・サイズ $(X, Y, W, H)$, 枠線・背景・フォントアピアランス属性。
- 列挙型 `Element`（上記パーツの統合カプセル化）。

#### [NEW] [align.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/align.rs)
- 複数選択要素の整列アルゴリズム (`AlignMode`: Left, Center, Right, Bottom, Middle, Top)。
- 全整列結果の 1mm 整数スナップ演算。

#### [MODIFY] [command.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/command.rs)
- 可逆コマンド構造体 `Command` (`AddElement`, `RemoveElement`, `MoveElement`, `ResizeElement`, `UpdateProperty`, `AlignElements`, `SetPagePreset`)。

#### [MODIFY] [history.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/history.rs)
- Command パターンに基づく Undo / Redo 履歴マネージャ `CommandHistory`。

#### [NEW] [document.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/document.rs)
- 単一ページ帳票ドキュメント構造体 `Document` の統合。1mm 格子バリデーション。

---

### Phase 2: PDF レンダリング・エクスポート統合 (`fepdf-layout-core` ➔ `fepdf`)

#### [NEW] [export.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-core/src/export.rs)
- `Document` を `fepdf` の PDF 2.0 ファサードデータ（PDF ページ、テキスト演算子、フォームアピアランス Stream）へ変換・出力するエクスポータ。
- 原点オフセットおよび左下原点 $Y$ 軸の変換計算処理。

---

### Phase 3: GUI インタフェース ＆ キャンバス描画 (`fepdf-layout-ui`)

#### [MODIFY] [Cargo.toml](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-ui/Cargo.toml)
- `egui`, `eframe` (または `egui-wgpu` + `winit`) の依存追加。

#### [NEW] [app.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-ui/src/app.rs)
- `egui::App` メインアプリケーションループ。
- 左: プロパティ, 中央: 作業キャンバス, 右: パーツパレットの 3 パネルレイアウトの構築。

#### [NEW] [canvas.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-ui/src/views/canvas.rs)
- 1mm 格子グリッド描画（10mm 主線強調）。
- 左下原点に基づくキャンバス座標レンダリングおよびルーラー（定規）表示。
- ドラッグ＆ドロップ配置、マウスドラッグ移動・四隅ハンドルリサイズの 1mm スナップ処理。

#### [NEW] [inspector.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-ui/src/views/inspector.rs)
- 左パネルのプロパティインスペクター。
- 1mm 単位の数値入力フィールド、文字揃え (`Justify`含む)、`field_tag` 入力欄、整列ボタン群。

#### [NEW] [palette.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-ui/src/views/palette.rs)
- 右パネルのパーツパレット。全 6 種のドラッグ用アイコン UI。

#### [NEW] [toolbar.rs](file:///Users/akahmys/projects/fepdf-layout/crates/fepdf-layout-ui/src/views/toolbar.rs)
- 上部ヘッダー（新規、保存、PDF出力、Undo/Redo、用紙サイズ選択、ズーム率）。

---

## 🧪 5. Verification Plan (検証計画)

### 自動テスト (`cargo test`)
1. **単位・座標変換テスト**: `Mm` $\leftrightarrow$ `Pt` 変換および Letter/Legal オフセット計算の精度検証。
2. **要素スナップテスト**: 浮動小数点座標が必ず 1mm 整数にスナップされることのテスト。
3. **整列アルゴリズムテスト**: 左・右・中央・上・下・垂直中央整列結果の 1mm 格子一致テスト。
4. **Command & Undo/Redo テスト**: コマンドの実行・Undo・Redo 後のドキュメント状態完全完全一致テスト。

### ガバナンス・セキュリティ検証
- `cargo deny check licenses` によるライセンスチェック。
- `betterleaks git --pre-commit --staged` によるセキュリティ・個人名漏洩チェック。
