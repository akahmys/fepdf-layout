//! Main egui application UI for fepdf-layout.

use eframe::egui;
use fepdf_layout_core::{
    Color, Command, DocumentManager, Element, ElementId, FormFieldElement,
    FormFieldKind, Mm, PagePreset, TextAlign, TextBoxElement,
};
use std::collections::HashSet;

use crate::canvas::render_workspace_canvas;
use crate::inspector::render_property_inspector;
use crate::palette::render_parts_palette;
use crate::toolbar::render_top_toolbar;

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
                width: Mm::new(10),
                height: Mm::new(10),
                border_color: Color::BLACK,
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
                width: Mm::new(10),
                height: Mm::new(10),
                border_color: Color::BLACK,
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
                width: Mm::new(40),
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

    pub fn align_selected(&mut self, mode: fepdf_layout_core::AlignMode) {
        let old_elems: Vec<_> = self
            .selected_ids
            .iter()
            .filter_map(|id| self.mgr.doc.get_element(*id).cloned())
            .collect();

        if old_elems.len() < 2 {
            return;
        }

        let mut new_elems = old_elems.clone();
        fepdf_layout_core::align_elements(&mut new_elems, mode);

        self.mgr.execute(Command::AlignElements {
            old_elements: old_elems,
            new_elements: new_elems,
        });
        self.status_msg = format!("選択パーツ {} 個を整列しました", self.selected_ids.len());
    }
}

impl eframe::App for FepdfLayoutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keyboard Handlers (Escape to cancel creation, Delete/Backspace to remove selected)
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.tool_state != ToolState::Select {
                self.tool_state = ToolState::Select;
                self.status_msg = "パーツ作成をキャンセルしました".to_string();
            }
        }

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
        render_top_toolbar(self, ctx);

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
        render_property_inspector(self, ctx);

        // --- 4. Right Panel: Parts Palette ---
        render_parts_palette(self, ctx);

        // --- 5. Center Workspace: 1mm Grid Canvas ---
        render_workspace_canvas(self, ctx);
    }
}

/// Setup Japanese CJK System Fonts into `egui::FontDefinitions`.
fn setup_japanese_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let font_paths = [
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
    ];

    let mut loaded = false;
    for path in &font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "cjk_font".to_owned(),
                egui::FontData::from_owned(font_data),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "cjk_font".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("cjk_font".to_owned());

            tracing::info!("Loaded Japanese font from {}", path);
            loaded = true;
            break;
        }
    }

    if !loaded {
        tracing::warn!("No CJK system fonts found. Fallback to default fonts.");
    }

    ctx.set_fonts(fonts);
}
