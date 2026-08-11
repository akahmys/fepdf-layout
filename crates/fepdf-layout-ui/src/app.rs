//! Main egui application UI for fepdf-layout.

use eframe::egui;
use fepdf_layout_core::{
    align_elements, Color, Command, Document, DocumentManager, Element, ElementId, FormFieldElement,
    FormFieldKind, LineCap, LineElement, Mm, PagePreset, PdfExporter, StrokeStyle, TextAlign, TextBoxElement,
};
use std::collections::HashSet;

/// Interactive tool state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolState {
    /// Select & Move mode
    Select,
    /// Line tool waiting for 1st click (Start Point)
    LineWaitStart,
    /// Line tool waiting for 2nd click (End Point)
    LineWaitEnd { start_x: u32, start_y: u32 },
}

/// Active handle being dragged for a selected line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineHandleKind {
    StartPoint,
    EndPoint,
    Body,
}

/// Main `egui` application state.
pub struct FepdfLayoutApp {
    pub mgr: DocumentManager,
    pub tool_state: ToolState,
    pub selected_ids: HashSet<ElementId>,
    pub zoom: f32,
    pub status_msg: String,
    pub active_line_handle: Option<(ElementId, LineHandleKind)>,
    pub last_drag_mm: Option<(i32, i32)>,
}

impl Default for FepdfLayoutApp {
    fn default() -> Self {
        Self {
            mgr: DocumentManager::new(PagePreset::A4),
            tool_state: ToolState::Select,
            selected_ids: HashSet::new(),
            zoom: 1.0,
            status_msg: "準備完了 (1mm 格子スナップ有効)".to_string(),
            active_line_handle: None,
            last_drag_mm: None,
        }
    }
}

impl FepdfLayoutApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_japanese_font(&cc.egui_ctx);
        Self::default()
    }

    /// Add a non-line preset element (TextBox, FormFields) to the active page.
    pub fn add_element_preset(&mut self, kind_str: &str) {
        let id = self.mgr.doc.next_id();
        let count = self.mgr.doc.elements.len() as u32;
        let offset = (count * 10) % 100;
        let base_x = 20 + offset;
        let base_y = 30 + offset;

        let elem = match kind_str {
            "textbox" => Element::TextBox(TextBoxElement {
                id,
                x: Mm::new(base_x),
                y: Mm::new(base_y),
                width: Mm::new(60),
                height: Mm::new(20),
                text: "テキスト".to_string(),
                font_family: "Sans".to_string(),
                font_size_pt: 12.0,
                text_color: Color::BLACK,
                align: TextAlign::Left,
                horizontal_scaling: 100,
                auto_fit_horizontal: false,
            }),
            "textfield" => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::TextField,
                field_tag: format!("field_{}", id.0),
                x: Mm::new(base_x),
                y: Mm::new(base_y),
                width: Mm::new(50),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            "checkbox" => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::CheckBox,
                field_tag: format!("check_{}", id.0),
                x: Mm::new(base_x),
                y: Mm::new(base_y),
                width: Mm::new(12),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            "radio" => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::RadioButton,
                field_tag: format!("radio_{}", id.0),
                x: Mm::new(base_x),
                y: Mm::new(base_y),
                width: Mm::new(12),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            "combo" => Element::FormField(FormFieldElement {
                id,
                kind: FormFieldKind::ComboBox,
                field_tag: format!("combo_{}", id.0),
                x: Mm::new(base_x),
                y: Mm::new(base_y),
                width: Mm::new(50),
                height: Mm::new(12),
                border_color: Color::GRAY_LIGHT,
                border_width: Mm::new(1),
                bg_color: Color::WHITE,
                font_size_pt: 10.0,
                text_color: Color::BLACK,
            }),
            _ => return,
        };

        self.mgr.execute(Command::AddElement(elem.clone()));
        self.selected_ids.clear();
        self.selected_ids.insert(elem.id());
        self.tool_state = ToolState::Select;
        self.status_msg = format!("パーツ #{} を追加・選択しました", id.0);
    }
}

impl eframe::App for FepdfLayoutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard Delete Handler
        if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
            let to_remove: Vec<_> = self.selected_ids.iter().cloned().collect();
            for id in to_remove {
                if let Some(elem) = self.mgr.doc.get_element(id).cloned() {
                    self.mgr.execute(Command::RemoveElement(elem));
                }
            }
            self.selected_ids.clear();
            self.status_msg = "選択パーツを削除しました".to_string();
        }

        // --- 1. Top Header Toolbar ---
        egui::TopBottomPanel::top("header_toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("fepdf-layout (帳票エディタ)");
                ui.separator();

                if ui.button("📂 開く").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("fepdf layout project", &["fepdf-layout", "json"])
                        .pick_file()
                    {
                        match Document::load_json(&path) {
                            Ok(doc) => {
                                self.mgr.doc = doc;
                                self.mgr.history = fepdf_layout_core::CommandHistory::new();
                                self.selected_ids.clear();
                                self.status_msg = format!("プロジェクトを読み込みました: {}", path.display());
                            }
                            Err(e) => {
                                self.status_msg = format!("読み込み失敗: {}", e);
                            }
                        }
                    }
                }

                if ui.button("💾 保存").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("fepdf layout project", &["fepdf-layout", "json"])
                        .set_file_name("layout_project.fepdf-layout")
                        .save_file()
                    {
                        match self.mgr.doc.save_json(&path) {
                            Ok(_) => {
                                self.status_msg = format!("プロジェクトを保存しました: {}", path.display());
                            }
                            Err(e) => {
                                self.status_msg = format!("保存失敗: {}", e);
                            }
                        }
                    }
                }

                if ui.button("📄 PDF出力").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PDF Document", &["pdf"])
                        .set_file_name("form_document.pdf")
                        .save_file()
                    {
                        match PdfExporter::export_to_file(&self.mgr.doc, &path) {
                            Ok(_) => {
                                self.status_msg = format!("PDFを出力しました: {}", path.display());
                            }
                            Err(e) => {
                                self.status_msg = format!("PDF出力失敗: {}", e);
                            }
                        }
                    }
                }

                ui.separator();

                if ui.button("↩ Undo").clicked() && self.mgr.history.can_undo() {
                    self.mgr.undo();
                    self.status_msg = "Undo 完了".to_string();
                }
                if ui.button("↪ Redo").clicked() && self.mgr.history.can_redo() {
                    self.mgr.redo();
                    self.status_msg = "Redo 完了".to_string();
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
                ui.add(egui::Slider::new(&mut self.zoom, 0.5..=20.0).text("倍"));
            });
        });

        // --- 2. Bottom Status Bar ---
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("単位: mm | 1mm Grid Snap | 左下原点 (0,0)");
                });
            });
        });

        // --- 3. Left Panel: Property Inspector ---
        egui::SidePanel::left("left_inspector")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                if self.selected_ids.is_empty() {
                    ui.label("パーツ未選択");
                } else if self.selected_ids.len() == 1 {
                    let id = *self.selected_ids.iter().next().unwrap();
                    if let Some(elem) = self.mgr.doc.get_element(id).cloned() {
                        ui.label(format!("ID: #{}", id.0));
                        let bounds = elem.bounds();
                        ui.label(format!("位置: X={} mm, Y={} mm", bounds.x.0, bounds.y.0));
                        ui.label(format!("サイズ: W={} mm, H={} mm", bounds.width.0, bounds.height.0));
                        ui.separator();

                        match elem {
                            Element::Line(mut l) => {
                                ui.label("■ 直線属性");
                                ui.label(format!("始点 (X1, Y1): ({}, {}) mm", l.x1.0, l.y1.0));
                                ui.label(format!("終点 (X2, Y2): ({}, {}) mm", l.x2.0, l.y2.0));
                                let old_elem = Element::Line(l.clone());

                                let mut stroke_w = l.stroke_width.0;
                                if ui.horizontal(|ui| {
                                    ui.label("線の太さ:");
                                    ui.add(egui::DragValue::new(&mut stroke_w).range(1..=20).suffix(" mm")).changed()
                                }).inner {
                                    l.stroke_width = Mm::new(stroke_w);
                                    self.mgr.execute(Command::UpdateElement {
                                        old: old_elem.clone(),
                                        new: Element::Line(l.clone()),
                                    });
                                }

                                ui.horizontal(|ui| {
                                    ui.label("端点形状:");
                                    let mut cap_changed = false;
                                    if ui.selectable_value(&mut l.line_cap, LineCap::Butt, "平頭 (Butt)").clicked() { cap_changed = true; }
                                    if ui.selectable_value(&mut l.line_cap, LineCap::Round, "丸頭 (Round)").clicked() { cap_changed = true; }
                                    if ui.selectable_value(&mut l.line_cap, LineCap::Square, "角頭 (Square)").clicked() { cap_changed = true; }
                                    if cap_changed {
                                        self.mgr.execute(Command::UpdateElement {
                                            old: old_elem.clone(),
                                            new: Element::Line(l.clone()),
                                        });
                                    }
                                });
                            }
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
                                    ui.label("文字揃え:");
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
                                    let old_elem = self.mgr.doc.get_element(id).unwrap().clone();
                                    if ui.text_edit_singleline(&mut ff.field_tag).changed() {
                                        self.mgr.execute(Command::UpdateElement {
                                            old: old_elem,
                                            new: Element::FormField(ff.clone()),
                                        });
                                    }
                                });
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
                let is_line_active = matches!(self.tool_state, ToolState::LineWaitStart | ToolState::LineWaitEnd { .. });
                if ui.selectable_label(is_line_active, "直線").clicked() {
                    self.tool_state = ToolState::LineWaitStart;
                    self.selected_ids.clear();
                    self.status_msg = "キャンバス上をクリックして直線の【始点】を指定してください".to_string();
                }

                if ui.button("テキストボックス").clicked() {
                    self.add_element_preset("textbox");
                }

                if ui.button("テキスト枠").clicked() {
                    self.add_element_preset("textfield");
                }
                if ui.button("チェックボックス").clicked() {
                    self.add_element_preset("checkbox");
                }
                if ui.button("ラジオボタン").clicked() {
                    self.add_element_preset("radio");
                }
                if ui.button("ドロップダウン").clicked() {
                    self.add_element_preset("combo");
                }
            });

        // --- 5. Center Workspace: 1mm Grid Canvas ---
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let page_spec = self.mgr.doc.page_spec;
                    let scale = 2.0_f32 * self.zoom; // 1mm = 2.0 * zoom pixels
                    let page_w_px = (page_spec.layout_width.0 as f32) * scale;
                    let page_h_px = (page_spec.layout_height.0 as f32) * scale;

            let (response, painter) = ui.allocate_painter(
                egui::vec2(page_w_px + 40.0, page_h_px + 40.0),
                egui::Sense::hover(),
            );

            let canvas_origin = response.rect.min + egui::vec2(20.0, 20.0);
            let page_rect = egui::Rect::from_min_size(canvas_origin, egui::vec2(page_w_px, page_h_px));

            // Bottom-Left origin coordinate converters
            let mm_to_screen = |x_mm: u32, y_mm: u32| -> egui::Pos2 {
                let px = page_rect.min.x + (x_mm as f32) * scale;
                let py = page_rect.max.y - (y_mm as f32) * scale;
                egui::pos2(px, py)
            };

            let screen_to_mm = |pos: egui::Pos2| -> (u32, u32) {
                let rel_x = (pos.x - page_rect.min.x) / scale;
                let rel_y = (page_rect.max.y - pos.y) / scale;
                let x_mm = rel_x.round().max(0.0) as u32;
                let y_mm = rel_y.round().max(0.0) as u32;
                (x_mm.min(page_spec.layout_width.0), y_mm.min(page_spec.layout_height.0))
            };

            // Helper to draw a dark coordinate hover badge tooltip near screen pos
            let draw_coord_badge = |painter: &egui::Painter, pos: egui::Pos2, text: &str| {
                let font = egui::FontId::proportional(11.0);
                let text_pos = pos + egui::vec2(12.0, -22.0);
                let text_rect = painter.text(
                    text_pos,
                    egui::Align2::LEFT_TOP,
                    text,
                    font.clone(),
                    egui::Color32::WHITE,
                );
                let bg_rect = text_rect.expand(4.0);
                painter.rect_filled(bg_rect, 4.0, egui::Color32::from_black_alpha(220));
                painter.rect_stroke(bg_rect, 4.0, egui::Stroke::new(1.0_f32, egui::Color32::from_gray(100)));
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_TOP,
                    text,
                    font,
                    egui::Color32::WHITE,
                );
            };

            // 1. Draw Page Sheet Background
            painter.rect_filled(page_rect, 0.0, egui::Color32::WHITE);
            painter.rect_stroke(page_rect, 0.0, egui::Stroke::new(2.0_f32, egui::Color32::DARK_GRAY));

            // 2. Draw Grid Lines (1mm fine lines & 5mm major lines)
            let color_1mm = egui::Color32::from_gray(215);
            let color_5mm = egui::Color32::from_gray(195);

            for gx in 0..=page_spec.layout_width.0 {
                let p1 = mm_to_screen(gx, 0);
                let p2 = mm_to_screen(gx, page_spec.layout_height.0);
                let color = if gx % 5 == 0 { color_5mm } else { color_1mm };
                let stroke_w = if gx % 5 == 0 { 0.8_f32 } else { 0.4_f32 };
                painter.line_segment([p1, p2], egui::Stroke::new(stroke_w, color));
            }

            for gy in 0..=page_spec.layout_height.0 {
                let p1 = mm_to_screen(0, gy);
                let p2 = mm_to_screen(page_spec.layout_width.0, gy);
                let color = if gy % 5 == 0 { color_5mm } else { color_1mm };
                let stroke_w = if gy % 5 == 0 { 0.8_f32 } else { 0.4_f32 };
                painter.line_segment([p1, p2], egui::Stroke::new(stroke_w, color));
            }

            // 3. Render All Document Elements & Clean Round Line Handles
            for elem in &self.mgr.doc.elements {
                let bounds = elem.bounds();
                let is_selected = self.selected_ids.contains(&elem.id());

                match elem {
                    Element::Line(l) => {
                        let p1 = mm_to_screen(l.x1.0, l.y1.0);
                        let p2 = mm_to_screen(l.x2.0, l.y2.0);
                        let color = egui::Color32::from_rgba_unmultiplied(
                            l.stroke_color.r,
                            l.stroke_color.g,
                            l.stroke_color.b,
                            l.stroke_color.a,
                        );
                        let stroke_w = (l.stroke_width.0 as f32 * scale).max(1.5);

                        match l.line_cap {
                            LineCap::Butt => {
                                painter.line_segment([p1, p2], egui::Stroke::new(stroke_w, color));
                            }
                            LineCap::Round => {
                                painter.line_segment([p1, p2], egui::Stroke::new(stroke_w, color));
                                painter.circle_filled(p1, stroke_w / 2.0, color);
                                painter.circle_filled(p2, stroke_w / 2.0, color);
                            }
                            LineCap::Square => {
                                let dir = (p2 - p1).normalized();
                                let ext = dir * (stroke_w / 2.0);
                                painter.line_segment([p1 - ext, p2 + ext], egui::Stroke::new(stroke_w, color));
                            }
                        }

                        if is_selected {
                            painter.circle_filled(p1, 5.0, egui::Color32::BLUE);
                            painter.circle_stroke(p1, 5.0, egui::Stroke::new(1.5_f32, egui::Color32::WHITE));

                            painter.circle_filled(p2, 5.0, egui::Color32::BLUE);
                            painter.circle_stroke(p2, 5.0, egui::Stroke::new(1.5_f32, egui::Color32::WHITE));
                        }
                    }
                    Element::TextBox(t) => {
                        let p_left_top = mm_to_screen(bounds.x.0, bounds.y.0 + bounds.height.0);
                        let p_right_bottom = mm_to_screen(bounds.x.0 + bounds.width.0, bounds.y.0);
                        let elem_rect = egui::Rect::from_two_pos(p_left_top, p_right_bottom);

                        let bg_color = egui::Color32::from_rgba_unmultiplied(245, 245, 255, 255);
                        let text_color = egui::Color32::from_rgba_unmultiplied(
                            t.text_color.r,
                            t.text_color.g,
                            t.text_color.b,
                            t.text_color.a,
                        );
                        painter.rect_filled(elem_rect, 2.0, bg_color);
                        painter.rect_stroke(
                            elem_rect,
                            2.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_gray(180)),
                        );

                        painter.text(
                            elem_rect.min + egui::vec2(4.0, 4.0),
                            egui::Align2::LEFT_TOP,
                            &t.text,
                            egui::FontId::proportional(t.font_size_pt as f32),
                            text_color,
                        );

                        if is_selected {
                            painter.rect_stroke(
                                elem_rect.expand(2.0),
                                2.0,
                                egui::Stroke::new(2.0_f32, egui::Color32::BLUE),
                            );
                        }
                    }
                    Element::FormField(f) => {
                        let p_left_top = mm_to_screen(bounds.x.0, bounds.y.0 + bounds.height.0);
                        let p_right_bottom = mm_to_screen(bounds.x.0 + bounds.width.0, bounds.y.0);
                        let elem_rect = egui::Rect::from_two_pos(p_left_top, p_right_bottom);

                        let bg_color = egui::Color32::from_rgba_unmultiplied(
                            f.bg_color.r,
                            f.bg_color.g,
                            f.bg_color.b,
                            f.bg_color.a,
                        );
                        let border_color = egui::Color32::from_rgba_unmultiplied(
                            f.border_color.r,
                            f.border_color.g,
                            f.border_color.b,
                            f.border_color.a,
                        );
                        painter.rect_filled(elem_rect, 1.0, bg_color);
                        painter.rect_stroke(
                            elem_rect,
                            1.0,
                            egui::Stroke::new(1.5_f32, border_color),
                        );

                        let label = format!("[{:?}] {}", f.kind, f.field_tag);
                        painter.text(
                            elem_rect.min + egui::vec2(4.0, 2.0),
                            egui::Align2::LEFT_TOP,
                            &label,
                            egui::FontId::proportional(10.0),
                            egui::Color32::DARK_BLUE,
                        );

                        if is_selected {
                            painter.rect_stroke(
                                elem_rect.expand(2.0),
                                2.0,
                                egui::Stroke::new(2.0_f32, egui::Color32::BLUE),
                            );
                        }
                    }
                }
            }

            // 4. Live Rubber-Band Line Creation Preview & Hover Coordinates
            match self.tool_state {
                ToolState::LineWaitStart => {
                    if let Some(pointer_pos) = ctx.pointer_interact_pos() {
                        if page_rect.contains(pointer_pos) {
                            let (curr_x, curr_y) = screen_to_mm(pointer_pos);
                            draw_coord_badge(&painter, pointer_pos, &format!("始点: {}, {}", curr_x, curr_y));
                        }
                    }
                }
                ToolState::LineWaitEnd { start_x, start_y } => {
                    if let Some(pointer_pos) = ctx.pointer_interact_pos() {
                        if page_rect.contains(pointer_pos) {
                            let (curr_x, curr_y) = screen_to_mm(pointer_pos);
                            let p1 = mm_to_screen(start_x, start_y);
                            let p2 = mm_to_screen(curr_x, curr_y);
                            painter.line_segment([p1, p2], egui::Stroke::new(2.0_f32, egui::Color32::RED));
                            painter.circle_filled(p1, 5.0, egui::Color32::RED);
                            painter.circle_filled(p2, 5.0, egui::Color32::RED);

                            // Draw active coordinate hover badges during creation
                            draw_coord_badge(&painter, p1, &format!("始点: {}, {}", start_x, start_y));
                            draw_coord_badge(&painter, p2, &format!("終点: {}, {}", curr_x, curr_y));
                        }
                    }
                }
                ToolState::Select => {}
            }

            // 5. Mouse Press & Drag Handling with Always-On Live Hover Coordinates
            if let Some(pointer_pos) = ctx.pointer_interact_pos() {
                if page_rect.contains(pointer_pos) {
                    let (mouse_x, mouse_y) = screen_to_mm(pointer_pos);

                    // Always-on live coordinate hover badge near cursor inside page area
                    draw_coord_badge(&painter, pointer_pos, &format!("{}, {}", mouse_x, mouse_y));

                    // MOUSE PRESS DOWN EVENT
                    if ctx.input(|i| i.pointer.primary_pressed()) {
                        match self.tool_state {
                            ToolState::LineWaitStart => {
                                self.tool_state = ToolState::LineWaitEnd {
                                    start_x: mouse_x,
                                    start_y: mouse_y,
                                };
                                self.status_msg = format!("始点を指定しました ({}, {}) mm。終点をクリックしてください", mouse_x, mouse_y);
                            }
                            ToolState::LineWaitEnd { start_x, start_y } => {
                                let id = self.mgr.doc.next_id();
                                let line = Element::Line(LineElement {
                                    id,
                                    x1: Mm::new(start_x),
                                    y1: Mm::new(start_y),
                                    x2: Mm::new(mouse_x),
                                    y2: Mm::new(mouse_y),
                                    stroke_width: Mm::new(1),
                                    stroke_color: Color::BLACK,
                                    stroke_style: StrokeStyle::Solid,
                                    line_cap: LineCap::Butt,
                                });
                                self.mgr.execute(Command::AddElement(line.clone()));
                                self.selected_ids.clear();
                                self.selected_ids.insert(id);
                                self.tool_state = ToolState::Select;
                                self.status_msg = format!("直線 #{} を作成しました", id.0);
                            }
                            ToolState::Select => {
                                let mut hit_handle = None;
                                let mut hit_id = None;

                                for elem in self.mgr.doc.elements.iter().rev() {
                                    match elem {
                                        Element::Line(l) => {
                                            let p1 = mm_to_screen(l.x1.0, l.y1.0);
                                            let p2 = mm_to_screen(l.x2.0, l.y2.0);

                                            if p1.distance(pointer_pos) <= 16.0 {
                                                hit_handle = Some((l.id, LineHandleKind::StartPoint));
                                                hit_id = Some(l.id);
                                                break;
                                            }
                                            if p2.distance(pointer_pos) <= 16.0 {
                                                hit_handle = Some((l.id, LineHandleKind::EndPoint));
                                                hit_id = Some(l.id);
                                                break;
                                            }

                                            let b = elem.bounds();
                                            let in_x = mouse_x >= b.x.0.saturating_sub(1) && mouse_x <= (b.x.0 + b.width.0 + 1);
                                            let in_y = mouse_y >= b.y.0.saturating_sub(1) && mouse_y <= (b.y.0 + b.height.0 + 1);
                                            if in_x && in_y {
                                                hit_handle = Some((l.id, LineHandleKind::Body));
                                                hit_id = Some(l.id);
                                                break;
                                            }
                                        }
                                        _ => {
                                            let b = elem.bounds();
                                            let in_x = mouse_x >= b.x.0 && mouse_x <= (b.x.0 + b.width.0);
                                            let in_y = mouse_y >= b.y.0 && mouse_y <= (b.y.0 + b.height.0);
                                            if in_x && in_y {
                                                hit_id = Some(elem.id());
                                                break;
                                            }
                                        }
                                    }
                                }

                                if let Some(id) = hit_id {
                                    self.selected_ids.clear();
                                    self.selected_ids.insert(id);
                                    self.active_line_handle = hit_handle;
                                    self.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));

                                    if let Some((_, kind)) = hit_handle {
                                        match kind {
                                            LineHandleKind::StartPoint => self.status_msg = "直線の【始点】を掴みました".to_string(),
                                            LineHandleKind::EndPoint => self.status_msg = "直線の【終点】を掴みました".to_string(),
                                            LineHandleKind::Body => self.status_msg = "直線全体を掴みました".to_string(),
                                        }
                                    } else {
                                        self.status_msg = format!("パーツ #{} を選択しました", id.0);
                                    }
                                } else {
                                    self.selected_ids.clear();
                                    self.active_line_handle = None;
                                    self.last_drag_mm = None;
                                }
                            }
                        }
                    }

                    // MOUSE DRAG EVENT & LIVE HOVER TOOLTIP DISPLAY
                    if ctx.input(|i| i.pointer.primary_down()) && !self.selected_ids.is_empty() {
                        if let Some((last_x, last_y)) = self.last_drag_mm {
                            let dx = (mouse_x as i32) - last_x;
                            let dy = (mouse_y as i32) - last_y;

                            if dx != 0 || dy != 0 {
                                if let Some((line_id, handle_kind)) = self.active_line_handle {
                                    if let Some(Element::Line(l)) = self.mgr.doc.get_element(line_id).cloned() {
                                        let old_elem = Element::Line(l.clone());
                                        let mut new_line = l.clone();

                                        match handle_kind {
                                            LineHandleKind::StartPoint => {
                                                let new_x1 = ((l.x1.0 as i32) + dx).max(0) as u32;
                                                let new_y1 = ((l.y1.0 as i32) + dy).max(0) as u32;
                                                new_line.x1 = Mm::new(new_x1);
                                                new_line.y1 = Mm::new(new_y1);
                                                self.status_msg = format!("始点移動 ({}, {}) mm", new_x1, new_y1);
                                            }
                                            LineHandleKind::EndPoint => {
                                                let new_x2 = ((l.x2.0 as i32) + dx).max(0) as u32;
                                                let new_y2 = ((l.y2.0 as i32) + dy).max(0) as u32;
                                                new_line.x2 = Mm::new(new_x2);
                                                new_line.y2 = Mm::new(new_y2);
                                                self.status_msg = format!("終点移動 ({}, {}) mm", new_x2, new_y2);
                                            }
                                            LineHandleKind::Body => {
                                                let new_x1 = ((l.x1.0 as i32) + dx).max(0) as u32;
                                                let new_y1 = ((l.y1.0 as i32) + dy).max(0) as u32;
                                                let new_x2 = ((l.x2.0 as i32) + dx).max(0) as u32;
                                                let new_y2 = ((l.y2.0 as i32) + dy).max(0) as u32;
                                                new_line.x1 = Mm::new(new_x1);
                                                new_line.y1 = Mm::new(new_y1);
                                                new_line.x2 = Mm::new(new_x2);
                                                new_line.y2 = Mm::new(new_y2);
                                                self.status_msg = format!("直線全体移動 ({}, {}) -> ({}, {})", new_x1, new_y1, new_x2, new_y2);
                                            }
                                        }

                                        self.mgr.execute(Command::UpdateElement {
                                            old: old_elem,
                                            new: Element::Line(new_line),
                                        });
                                        self.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                                    }
                                } else {
                                    // Move non-line elements
                                    for id in self.selected_ids.clone() {
                                        self.mgr.execute(Command::MoveElement { id, dx, dy });
                                    }
                                    self.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                                    self.status_msg = format!("移動: dx={}mm, dy={}mm", dx, dy);
                                }
                            }
                        } else {
                            self.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                        }

                        // Display live coordinate tooltip badge while dragging line handles or elements!
                        if let Some((line_id, handle_kind)) = self.active_line_handle {
                            if let Some(Element::Line(l)) = self.mgr.doc.get_element(line_id) {
                                match handle_kind {
                                    LineHandleKind::StartPoint => {
                                        let p = mm_to_screen(l.x1.0, l.y1.0);
                                        draw_coord_badge(&painter, p, &format!("始点: {}, {}", l.x1.0, l.y1.0));
                                    }
                                    LineHandleKind::EndPoint => {
                                        let p = mm_to_screen(l.x2.0, l.y2.0);
                                        draw_coord_badge(&painter, p, &format!("終点: {}, {}", l.x2.0, l.y2.0));
                                    }
                                    LineHandleKind::Body => {
                                        let p1 = mm_to_screen(l.x1.0, l.y1.0);
                                        draw_coord_badge(&painter, p1, &format!("始点: {}, {} | 終点: {}, {}", l.x1.0, l.y1.0, l.x2.0, l.y2.0));
                                    }
                                }
                            }
                        }
                    } else if !ctx.input(|i| i.pointer.primary_down()) {
                        self.last_drag_mm = None;
                        self.active_line_handle = None;
                    }
                }
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
