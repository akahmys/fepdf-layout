//! Right Parts Palette UI for fepdf-layout.

use eframe::egui;
use crate::app::{FepdfLayoutApp, ToolState};
use crate::icons::{lucide_button, LucideIcon};

pub fn render_parts_palette(app: &mut FepdfLayoutApp, ctx: &egui::Context) {
    egui::SidePanel::right("right_palette")
        .resizable(true)
        .default_width(180.0)
        .width_range(120.0..=400.0)
        .show(ctx, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
            ui.set_max_width(ui.available_width());

            let is_line_active = matches!(app.tool_state, ToolState::LineWaitStart | ToolState::LineWaitEnd { .. });
            if lucide_button(ui, LucideIcon::Line, "直線").clicked() || (is_line_active && ui.input(|i| i.pointer.primary_clicked())) {
                app.tool_state = ToolState::LineWaitStart;
                app.selected_ids.clear();
                app.status_msg = "キャンバス上をクリックして直線の【始点】を指定してください".to_string();
            }

            if lucide_button(ui, LucideIcon::Type, "テキストボックス").clicked() {
                app.add_element_preset("textbox");
            }

            if lucide_button(ui, LucideIcon::TextField, "テキスト枠").clicked() {
                app.add_element_preset("textfield");
            }
            if lucide_button(ui, LucideIcon::CheckBox, "チェックボックス").clicked() {
                app.add_element_preset("checkbox");
            }
            if lucide_button(ui, LucideIcon::RadioButton, "ラジオボタン").clicked() {
                app.add_element_preset("radio");
            }
            if lucide_button(ui, LucideIcon::ComboBox, "ドロップダウン").clicked() {
                app.add_element_preset("combo");
            }
        });
}
