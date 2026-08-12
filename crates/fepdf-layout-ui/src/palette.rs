//! Right Parts Palette UI for fepdf-layout.

use eframe::egui;
use crate::app::{FepdfLayoutApp, ToolState};

pub fn render_parts_palette(app: &mut FepdfLayoutApp, ctx: &egui::Context) {
    egui::SidePanel::right("right_palette")
        .resizable(true)
        .default_width(180.0)
        .width_range(120.0..=400.0)
        .show(ctx, |ui| {
            ui.set_max_width(ui.available_width());
            let is_line_active = matches!(app.tool_state, ToolState::LineWaitStart | ToolState::LineWaitEnd { .. });
            if ui.selectable_label(is_line_active, "直線").clicked() {
                app.tool_state = ToolState::LineWaitStart;
                app.selected_ids.clear();
                app.status_msg = "キャンバス上をクリックして直線の【始点】を指定してください".to_string();
            }

            if ui.button("テキストボックス").clicked() {
                app.add_element_preset("textbox");
            }

            if ui.button("テキスト枠").clicked() {
                app.add_element_preset("textfield");
            }
            if ui.button("チェックボックス").clicked() {
                app.add_element_preset("checkbox");
            }
            if ui.button("ラジオボタン").clicked() {
                app.add_element_preset("radio");
            }
            if ui.button("ドロップダウン").clicked() {
                app.add_element_preset("combo");
            }
        });
}
