//! Lucide Icons vector rendering system for fepdf-layout-ui.
//!
//! Provides MIT-licensed, high-precision vector icons (Lucide Icons design system)
//! rendered cleanly using egui Painter vector paths.

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LucideIcon {
    FolderOpen,
    Save,
    FileText,
    Undo,
    Redo,
    Line,
    Type,
    TextField,
    CheckBox,
    RadioButton,
    ComboBox,
}

/// Paint a Lucide vector icon inside the given bounding rectangle.
pub fn paint_lucide_icon(painter: &egui::Painter, rect: egui::Rect, icon: LucideIcon, color: egui::Color32) {
    let stroke = egui::Stroke::new(1.8_f32, color);
    let center = rect.center();
    let size = rect.width().min(rect.height());
    let r = size * 0.45;

    match icon {
        LucideIcon::FolderOpen => {
            let min = center - egui::vec2(r, r * 0.7);
            let max = center + egui::vec2(r, r * 0.7);
            let flap = egui::pos2(min.x + r * 0.6, min.y);
            painter.line_segment([min, flap], stroke);
            painter.rect_stroke(egui::Rect::from_min_max(min, max), 2.0_f32, stroke);
        }
        LucideIcon::Save => {
            let box_rect = egui::Rect::from_center_size(center, egui::vec2(r * 1.8, r * 1.8));
            painter.rect_stroke(box_rect, 2.0_f32, stroke);
            let notch = egui::Rect::from_min_max(
                center - egui::vec2(r * 0.5, r * 0.8),
                center + egui::vec2(r * 0.5, -r * 0.2),
            );
            painter.rect_stroke(notch, 1.0_f32, stroke);
        }
        LucideIcon::FileText => {
            let page = egui::Rect::from_center_size(center, egui::vec2(r * 1.5, r * 1.8));
            painter.rect_stroke(page, 2.0_f32, stroke);
            let l1_y = center.y - r * 0.3;
            let l2_y = center.y + r * 0.2;
            painter.line_segment([egui::pos2(center.x - r * 0.4, l1_y), egui::pos2(center.x + r * 0.4, l1_y)], stroke);
            painter.line_segment([egui::pos2(center.x - r * 0.4, l2_y), egui::pos2(center.x + r * 0.2, l2_y)], stroke);
        }
        LucideIcon::Undo => {
            let p1 = center + egui::vec2(r * 0.5, r * 0.4);
            let p2 = center + egui::vec2(-r * 0.2, -r * 0.4);
            let p3 = center + egui::vec2(-r * 0.6, 0.0);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
            painter.line_segment([p2, p2 + egui::vec2(0.0, r * 0.5)], stroke);
        }
        LucideIcon::Redo => {
            let p1 = center + egui::vec2(-r * 0.5, r * 0.4);
            let p2 = center + egui::vec2(r * 0.2, -r * 0.4);
            let p3 = center + egui::vec2(r * 0.6, 0.0);
            painter.line_segment([p1, p2], stroke);
            painter.line_segment([p2, p3], stroke);
            painter.line_segment([p2, p2 + egui::vec2(0.0, r * 0.5)], stroke);
        }
        LucideIcon::Line => {
            let p1 = center + egui::vec2(-r * 0.8, r * 0.6);
            let p2 = center + egui::vec2(r * 0.8, -r * 0.6);
            painter.line_segment([p1, p2], stroke);
            painter.circle_filled(p1, 2.5, color);
            painter.circle_filled(p2, 2.5, color);
        }
        LucideIcon::Type => {
            let top_left = center + egui::vec2(-r * 0.7, -r * 0.7);
            let top_right = center + egui::vec2(r * 0.7, -r * 0.7);
            let bot = center + egui::vec2(0.0, r * 0.7);
            painter.line_segment([top_left, top_right], stroke);
            painter.line_segment([center + egui::vec2(0.0, -r * 0.7), bot], stroke);
        }
        LucideIcon::TextField => {
            let box_rect = egui::Rect::from_center_size(center, egui::vec2(r * 1.9, r * 1.3));
            painter.rect_stroke(box_rect, 2.0_f32, stroke);
            let ib1 = center + egui::vec2(-r * 0.4, -r * 0.3);
            let ib2 = center + egui::vec2(-r * 0.4, r * 0.3);
            painter.line_segment([ib1, ib2], stroke);
        }
        LucideIcon::CheckBox => {
            let box_rect = egui::Rect::from_center_size(center, egui::vec2(r * 1.6, r * 1.6));
            painter.rect_stroke(box_rect, 2.0_f32, stroke);
            let c1 = center + egui::vec2(-r * 0.4, 0.0);
            let c2 = center + egui::vec2(-r * 0.1, r * 0.3);
            let c3 = center + egui::vec2(r * 0.4, -r * 0.3);
            painter.line_segment([c1, c2], stroke);
            painter.line_segment([c2, c3], stroke);
        }
        LucideIcon::RadioButton => {
            painter.circle_stroke(center, r * 0.8, stroke);
            painter.circle_filled(center, r * 0.35, color);
        }
        LucideIcon::ComboBox => {
            let box_rect = egui::Rect::from_center_size(center, egui::vec2(r * 1.8, r * 1.3));
            painter.rect_stroke(box_rect, 2.0_f32, stroke);
            let a1 = center + egui::vec2(r * 0.2, -r * 0.2);
            let a2 = center + egui::vec2(r * 0.5, r * 0.2);
            let a3 = center + egui::vec2(r * 0.8, -r * 0.2);
            painter.line_segment([a1, a2], stroke);
            painter.line_segment([a2, a3], stroke);
        }
    }
}

/// Render a clean UI button containing a Lucide vector icon + label.
pub fn lucide_button(ui: &mut egui::Ui, icon: LucideIcon, text: &str) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            let color = ui.style().visuals.text_color();
            paint_lucide_icon(ui.painter(), rect, icon, color);
        }
        ui.button(text)
    }).inner
}
