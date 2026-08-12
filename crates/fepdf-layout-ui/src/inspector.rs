//! Left Property Inspector UI for fepdf-layout.

use eframe::egui;
use fepdf_layout_core::{Command, Element, LineCap, Mm, TextAlign};
use crate::app::FepdfLayoutApp;

pub fn render_property_inspector(app: &mut FepdfLayoutApp, ctx: &egui::Context) {
    egui::SidePanel::left("left_inspector")
        .resizable(true)
        .default_width(260.0)
        .width_range(160.0..=500.0)
        .show(ctx, |ui| {
            ui.set_max_width(ui.available_width());
            if app.selected_ids.is_empty() {
                ui.label("パーツ未選択");
            } else if app.selected_ids.len() == 1 {
                let id = *app.selected_ids.iter().next().unwrap();
                if let Some(elem) = app.mgr.doc.get_element(id).cloned() {
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
                                app.mgr.execute(Command::UpdateElement {
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
                                    app.mgr.execute(Command::UpdateElement {
                                        old: old_elem.clone(),
                                        new: Element::Line(l.clone()),
                                    });
                                }
                            });
                        }
                        Element::TextBox(mut tb) => {
                            ui.label("■ テキスト内容");
                            let old_elem = app.mgr.doc.get_element(id).unwrap().clone();
                            if ui.text_edit_singleline(&mut tb.text).changed() {
                                app.mgr.execute(Command::UpdateElement {
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
                                let old_elem = app.mgr.doc.get_element(id).unwrap().clone();
                                if ui.text_edit_singleline(&mut ff.field_tag).changed() {
                                    app.mgr.execute(Command::UpdateElement {
                                        old: old_elem,
                                        new: Element::FormField(ff.clone()),
                                    });
                                }
                            });
                        }
                    }
                }
            } else {
                ui.label(format!("選択中パーツ: {} 個", app.selected_ids.len()));
                ui.separator();
                ui.label("■ 整列操作 (1mm Grid Snap)");
                ui.horizontal(|ui| {
                    if ui.button("⬅ 左揃え").clicked() {
                        app.align_selected(fepdf_layout_core::AlignMode::Left);
                    }
                    if ui.button("↔ 中央揃え").clicked() {
                        app.align_selected(fepdf_layout_core::AlignMode::Center);
                    }
                    if ui.button("➡️ 右揃え").clicked() {
                        app.align_selected(fepdf_layout_core::AlignMode::Right);
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("⬇ 下揃え").clicked() {
                        app.align_selected(fepdf_layout_core::AlignMode::Bottom);
                    }
                    if ui.button("↕ 上下中央").clicked() {
                        app.align_selected(fepdf_layout_core::AlignMode::Middle);
                    }
                    if ui.button("⬆ 上揃え").clicked() {
                        app.align_selected(fepdf_layout_core::AlignMode::Top);
                    }
                });
            }
        });
}
