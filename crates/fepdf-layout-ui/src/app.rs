//! Main egui application UI for fepdf-layout.

use eframe::egui;
use fepdf_layout_core::{
    align_elements, Color, Command, DocumentManager, Element, ElementId, FormFieldElement,
    FormFieldKind, LineElement, Mm, PagePreset, StrokeStyle, TextAlign, TextBoxElement,
};
use std::collections::HashSet;

/// Active tool in the palette/toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Line,
    TextBox,
    TextField,
    CheckBox,
    RadioButton,
    ComboBox,
}

/// Main `egui` application state.
pub struct FepdfLayoutApp {
    pub mgr: DocumentManager,
    pub active_tool: ActiveTool,
    pub selected_ids: HashSet<ElementId>,
    pub zoom: f32,
    pub status_msg: String,
}

impl Default for FepdfLayoutApp {
    fn default() -> Self {
        Self {
            mgr: DocumentManager::new(PagePreset::A4),
            active_tool: ActiveTool::Select,
            selected_ids: HashSet::new(),
            zoom: 1.0,
            status_msg: "Ready (1mm Grid Snap Active)".to_string(),
        }
    }
}

impl FepdfLayoutApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_japanese_font(&cc.egui_ctx);
        Self::default()
    }

    /// Add a new element to the center of the active page.
    pub fn add_element_preset(&mut self, tool: ActiveTool) {
        let id = self.mgr.doc.next_id();
        let center_x = self.mgr.doc.page_spec.layout_width.0 / 2;
        let center_y = self.mgr.doc.page_spec.layout_height.0 / 2;

        let elem = match tool {
            ActiveTool::Line => Element::Line(LineElement {
                id,
                x1: Mm::new(center_x.saturating_sub(20)),
                y1: Mm::new(center_y),
                x2: Mm::new(center_x + 20),
                y2: Mm::new(center_y),
                stroke_width: Mm::new(1),
                stroke_color: Color::BLACK,
                stroke_style: StrokeStyle::Solid,
            }),
            ActiveTool::TextBox => Element::TextBox(TextBoxElement {
                id,
                x: Mm::new(center_x.saturating_sub(30)),
                y: Mm::new(center_y.saturating_sub(10)),
                width: Mm::new(60),
                height: Mm::new(20),
                text: "新規テキスト".to_string(),
                font_family: "Sans".to_string(),
                font_size_pt: 12.0,
                text_color: Color::BLACK,
                align: TextAlign::Left,
                horizontal_scaling: 100,
                auto_fit_horizontal: false,
            }),
            ActiveTool::TextField => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::TextField,
                field_tag: format!("field_{}", id.0),
                x: Mm::new(center_x.saturating_sub(25)),
                y: Mm::new(center_y.saturating_sub(6)),
                width: Mm::new(50),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            ActiveTool::CheckBox => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::CheckBox,
                field_tag: format!("check_{}", id.0),
                x: Mm::new(center_x.saturating_sub(6)),
                y: Mm::new(center_y.saturating_sub(6)),
                width: Mm::new(12),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            ActiveTool::RadioButton => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::RadioButton,
                field_tag: format!("radio_{}", id.0),
                x: Mm::new(center_x.saturating_sub(6)),
                y: Mm::new(center_y.saturating_sub(6)),
                width: Mm::new(12),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            ActiveTool::ComboBox => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::ComboBox,
                field_tag: format!("combo_{}", id.0),
                x: Mm::new(center_x.saturating_sub(25)),
                y: Mm::new(center_y.saturating_sub(6)),
                width: Mm::new(50),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            ActiveTool::Select => return,
        };

        self.mgr.execute(Command::AddElement(elem.clone()));
        self.selected_ids.clear();
        self.selected_ids.insert(elem.id());
        self.status_msg = format!("要素 #{} を追加しました", id.0);
    }
}

impl eframe::App for FepdfLayoutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 1. Top Header Toolbar ---
        egui::TopBottomPanel::top("header_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("fepdf-layout (帳票エディタ)");
                ui.separator();

                if ui.button("↩ Undo").clicked() && self.mgr.history.can_undo() {
                    self.mgr.undo();
                    self.status_msg = "操作を取り消しました (Undo)".to_string();
                }
                if ui.button("↪ Redo").clicked() && self.mgr.history.can_redo() {
                    self.mgr.redo();
                    self.status_msg = "操作をやり直しました (Redo)".to_string();
                }

                ui.separator();
                ui.label("用紙:");
                let mut current_preset = self.mgr.doc.page_spec.preset;
                egui::ComboBox::from_id_salt("page_preset_combo")
                    .selected_text(format!("{:?}", current_preset))
                    .show_ui(ui, |ui| {
                        for preset in [
                            PagePreset::A4,
                            PagePreset::A3,
                            PagePreset::A5,
                            PagePreset::B4,
                            PagePreset::B5,
                            PagePreset::Letter,
                            PagePreset::Legal,
                        ] {
                            if ui.selectable_value(&mut current_preset, preset, format!("{:?}", preset)).clicked() {
                                self.mgr.execute(Command::SetPagePreset {
                                    old: self.mgr.doc.page_spec.preset,
                                    new: preset,
                                });
                            }
                        }
                    });

                ui.separator();
                ui.label("表示倍率:");
                ui.add(egui::Slider::new(&mut self.zoom, 0.5..=2.0).text("倍"));
            });
        });

        // --- 2. Bottom Status Bar ---
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("単位: mm | 1mm Grid Snap");
                });
            });
        });

        // --- 3. Left Panel: Property Inspector ---
        egui::SidePanel::left("left_inspector")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("プロパティ");
                ui.separator();

                if self.selected_ids.is_empty() {
                    ui.label("パーツが選択されていません");
                    ui.label("右パネルからパーツをドラッグ/追加するか、キャンバス上のパーツをクリックして選択してください。");
                } else if self.selected_ids.len() == 1 {
                    let id = *self.selected_ids.iter().next().unwrap();
                    if let Some(elem) = self.mgr.doc.get_element(id).cloned() {
                        ui.label(format!("ID: #{}", id.0));
                        let bounds = elem.bounds();
                        ui.label(format!("位置: X={} mm, Y={} mm (左下原点)", bounds.x.0, bounds.y.0));
                        ui.label(format!("サイズ: W={} mm, H={} mm", bounds.width.0, bounds.height.0));
                        ui.separator();

                        match elem {
                            Element::TextBox(mut tb) => {
                                ui.label("■ テキスト内容");
                                let old_elem = self.mgr.doc.get_element(id).unwrap().clone();
                                if ui.text_edit_singleline(&mut tb.text).changed() {
                                    self.mgr.execute(Command::UpdateElement {
                                        old: old_elem,
                                        new: Element::TextBox(tb.clone()),
                                    });
                                }
                                ui.horizontal(|ui| {
                                    ui.label("揃え:");
                                    ui.selectable_value(&mut tb.align, TextAlign::Left, "左");
                                    ui.selectable_value(&mut tb.align, TextAlign::Center, "中央");
                                    ui.selectable_value(&mut tb.align, TextAlign::Right, "右");
                                    ui.selectable_value(&mut tb.align, TextAlign::Justify, "均等割付");
                                });
                                ui.add(egui::Slider::new(&mut tb.horizontal_scaling, 50..=150).text("長体・平体%"));
                                ui.checkbox(&mut tb.auto_fit_horizontal, "自動枠合わせ (Auto-Fit)");
                            }
                            Element::FormField(mut ff) => {
                                ui.label("■ フォーム属性");
                                ui.label(format!("種別: {:?}", ff.kind));
                                ui.horizontal(|ui| {
                                    ui.label("集計タグ:");
                                    ui.text_edit_singleline(&mut ff.field_tag);
                                });
                            }
                            Element::Line(_) => {
                                ui.label("■ 直線属性");
                            }
                        }
                    }
                } else {
                    ui.label(format!("選択中パーツ: {} 個", self.selected_ids.len()));
                    ui.separator();
                    ui.label("■ 整列操作 (1mm Grid Snap)");
                    ui.horizontal(|ui| {
                        if ui.button("⬅ 左揃え").clicked() {
                            self.align_selected(fepdf_layout_core::AlignMode::Left);
                        }
                        if ui.button("↔ 中央揃え").clicked() {
                            self.align_selected(fepdf_layout_core::AlignMode::Center);
                        }
                        if ui.button("➡️ 右揃え").clicked() {
                            self.align_selected(fepdf_layout_core::AlignMode::Right);
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("⬇ 下揃え").clicked() {
                            self.align_selected(fepdf_layout_core::AlignMode::Bottom);
                        }
                        if ui.button("↕ 上下中央").clicked() {
                            self.align_selected(fepdf_layout_core::AlignMode::Middle);
                        }
                        if ui.button("⬆ 上揃え").clicked() {
                            self.align_selected(fepdf_layout_core::AlignMode::Top);
                        }
                    });
                }
            });

        // --- 4. Right Panel: Parts Palette ---
        egui::SidePanel::right("right_palette")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("パーツパレット");
                ui.separator();

                ui.label("■ 描画パーツ");
                if ui.button("─ 直線 (Line)").clicked() {
                    self.add_element_preset(ActiveTool::Line);
                }
                if ui.button("T テキストボックス").clicked() {
                    self.add_element_preset(ActiveTool::TextBox);
                }

                ui.separator();
                ui.label("■ PDF フォームパーツ");
                if ui.button("[ab] テキスト枠").clicked() {
                    self.add_element_preset(ActiveTool::TextField);
                }
                if ui.button("[☑] チェックボックス").clicked() {
                    self.add_element_preset(ActiveTool::CheckBox);
                }
                if ui.button("[○] ラジオボタン").clicked() {
                    self.add_element_preset(ActiveTool::RadioButton);
                }
                if ui.button("[▼] ドロップダウン").clicked() {
                    self.add_element_preset(ActiveTool::ComboBox);
                }
            });

        // --- 5. Center Workspace: 1mm Grid Canvas ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("作業スペース (1mm Grid Canvas / 左下原点)");

            let page_spec = self.mgr.doc.page_spec;
            ui.label(format!(
                "アクティブ領域: {} x {} mm (原点オフセット: {:.2}, {:.2} mm)",
                page_spec.layout_width.0,
                page_spec.layout_height.0,
                page_spec.offset_x.0,
                page_spec.offset_y.0
            ));

            // Canvas drawing frame
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (_response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
                let rect = painter.clip_rect();

                // Draw background page sheet
                painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0_f32, egui::Color32::GRAY));

                // Draw 10mm grid lines
                let step = 20.0 * self.zoom;
                let mut x = rect.min.x;
                while x < rect.max.x {
                    painter.line_segment(
                        [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                        egui::Stroke::new(0.5_f32, egui::Color32::from_gray(220)),
                    );
                    x += step;
                }

                let mut y = rect.min.y;
                while y < rect.max.y {
                    painter.line_segment(
                        [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                        egui::Stroke::new(0.5_f32, egui::Color32::from_gray(220)),
                    );
                    y += step;
                }
            });
        });
    }
}

impl FepdfLayoutApp {
    fn align_selected(&mut self, mode: fepdf_layout_core::AlignMode) {
        let mut selected_elems: Vec<_> = self
            .selected_ids
            .iter()
            .filter_map(|id| self.mgr.doc.get_element(*id).cloned())
            .collect();
        if selected_elems.len() >= 2 {
            let old_elems = selected_elems.clone();
            align_elements(&mut selected_elems, mode);
            self.mgr.execute(Command::AlignElements {
                old_elements: old_elems,
                new_elements: selected_elems,
            });
            self.status_msg = "整列を実行しました (1mm Grid Snap)".to_string();
        }
    }
}

/// Load system Japanese CJK font into egui FontDefinitions to resolve garbled text.
fn setup_japanese_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidate_paths = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/Supplemental/Songti.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\meiryo.ttc",
    ];

    for path in candidate_paths {
        if let Ok(font_bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "japanese_font".to_owned(),
                egui::FontData::from_owned(font_bytes),
            );

            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, "japanese_font".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                family.insert(0, "japanese_font".to_owned());
            }
            ctx.set_fonts(fonts);
            tracing::info!("Loaded Japanese font from {}", path);
            break;
        }
    }
}
