//! Top Header Toolbar rendering for fepdf-layout.

use eframe::egui;
use fepdf_layout_core::{Command, Document, PagePreset, PdfExporter};
use crate::app::FepdfLayoutApp;
use crate::icons::{lucide_button, LucideIcon};

pub fn render_top_toolbar(app: &mut FepdfLayoutApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("header_toolbar")
        .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(egui::Margin::symmetric(10.0, 6.0)))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 4.0);
            ui.horizontal(|ui| {
            if lucide_button(ui, LucideIcon::FolderOpen, "開く").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("fepdf layout project", &["fepdf-layout", "json"])
                    .pick_file()
                {
                    match Document::load_json(&path) {
                        Ok(doc) => {
                            app.mgr.doc = doc;
                            app.mgr.history = fepdf_layout_core::CommandHistory::new();
                            app.selected_ids.clear();
                            app.status_msg = format!("プロジェクトを読み込みました: {}", path.display());
                        }
                        Err(e) => {
                            app.status_msg = format!("読み込み失敗: {}", e);
                        }
                    }
                }
            }

            if lucide_button(ui, LucideIcon::Save, "保存").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("fepdf layout project", &["fepdf-layout", "json"])
                    .set_file_name("layout_project.fepdf-layout")
                    .save_file()
                {
                    match app.mgr.doc.save_json(&path) {
                        Ok(_) => {
                            app.status_msg = format!("プロジェクトを保存しました: {}", path.display());
                        }
                        Err(e) => {
                            app.status_msg = format!("保存失敗: {}", e);
                        }
                    }
                }
            }

            if lucide_button(ui, LucideIcon::FileText, "PDF出力").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PDF Document", &["pdf"])
                    .set_file_name("form_document.pdf")
                    .save_file()
                {
                    match PdfExporter::export_to_file(&app.mgr.doc, &path) {
                        Ok(_) => {
                            app.status_msg = format!("PDFを出力しました: {}", path.display());
                        }
                        Err(e) => {
                            app.status_msg = format!("PDF出力失敗: {}", e);
                        }
                    }
                }
            }

            ui.separator();

            if lucide_button(ui, LucideIcon::Undo, "Undo").clicked() && app.mgr.history.can_undo() {
                app.mgr.undo();
                app.status_msg = "Undo 完了".to_string();
            }
            if lucide_button(ui, LucideIcon::Redo, "Redo").clicked() && app.mgr.history.can_redo() {
                app.mgr.redo();
                app.status_msg = "Redo 完了".to_string();
            }

            ui.separator();
            ui.label("用紙:");
            let mut current_preset = app.mgr.doc.page_spec.preset;
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
                            app.mgr.execute(Command::SetPagePreset {
                                old: app.mgr.doc.page_spec.preset,
                                new: preset,
                            });
                        }
                    }
                });

            ui.separator();
            ui.label("表示倍率:");
            ui.add(egui::Slider::new(&mut app.zoom, 0.5..=20.0).text("倍"));
        });
    });
}
