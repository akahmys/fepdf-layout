//! Center workspace canvas rendering, grid snapping, and event interaction.

use eframe::egui;
use fepdf_layout_core::{Color, Command, Element, LineCap, LineElement, Mm, StrokeStyle};
use crate::app::{FepdfLayoutApp, LineHandleKind, ToolState};

pub fn render_workspace_canvas(app: &mut FepdfLayoutApp, ctx: &egui::Context) {
    // Mouse wheel & 2-finger trackpad pinch-to-zoom handler
    let zoom_delta = ctx.input(|i| i.zoom_delta());
    if (zoom_delta - 1.0).abs() > 0.001 {
        app.zoom = (app.zoom * zoom_delta).clamp(0.5, 20.0);
    }

    let scroll_y = ctx.input(|i| i.smooth_scroll_delta.y);
    if ctx.input(|i| i.modifiers.ctrl || i.modifiers.command || i.modifiers.mac_cmd) && scroll_y.abs() > 0.1 {
        let factor = if scroll_y > 0.0 { 1.05 } else { 0.95 };
        app.zoom = (app.zoom * factor).clamp(0.5, 20.0);
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let page_spec = app.mgr.doc.page_spec;
                let scale = 2.0_f32 * app.zoom; // 1mm = 2.0 * zoom pixels
                let page_w_px = (page_spec.layout_width.0 as f32) * scale;
                let page_h_px = (page_spec.layout_height.0 as f32) * scale;

                let avail_size = ui.available_size();
                let margin_x = if page_w_px + 40.0 < avail_size.x {
                    ((avail_size.x - page_w_px) / 2.0).max(20.0)
                } else {
                    20.0
                };
                let margin_y = if page_h_px + 40.0 < avail_size.y {
                    ((avail_size.y - page_h_px) / 2.0).max(20.0)
                } else {
                    20.0
                };

                let content_w = (page_w_px + margin_x * 2.0).max(avail_size.x);
                let content_h = (page_h_px + margin_y * 2.0).max(avail_size.y);

                let (response, painter) = ui.allocate_painter(
                    egui::vec2(content_w, content_h),
                    egui::Sense::hover(),
                );

                let canvas_origin = response.rect.min + egui::vec2(margin_x, margin_y);
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

                // 1. Render White Paper Background
                painter.rect_filled(page_rect, 0.0, egui::Color32::WHITE);
                painter.rect_stroke(
                    page_rect,
                    0.0,
                    egui::Stroke::new(1.5_f32, egui::Color32::from_gray(160)),
                );

                // 2. Render 1mm Fine Grid & 5mm Major Grid
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
                for elem in &app.mgr.doc.elements {
                    let _bounds = elem.bounds();
                    let is_selected = app.selected_ids.contains(&elem.id());

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
                        Element::TextBox(tb) => {
                            let min_p = mm_to_screen(tb.x.0, tb.y.0 + tb.height.0);
                            let max_p = mm_to_screen(tb.x.0 + tb.width.0, tb.y.0);
                            let elem_rect = egui::Rect::from_min_max(min_p, max_p);

                            painter.rect_filled(elem_rect, 0.0, egui::Color32::from_gray(245));
                            painter.rect_stroke(
                                elem_rect,
                                0.0,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(180)),
                            );

                            painter.text(
                                elem_rect.left_top() + egui::vec2(4.0, 4.0),
                                egui::Align2::LEFT_TOP,
                                &tb.text,
                                egui::FontId::proportional(12.0 * app.zoom),
                                egui::Color32::BLACK,
                            );

                            if is_selected {
                                painter.rect_stroke(
                                    elem_rect.expand(2.0),
                                    2.0,
                                    egui::Stroke::new(2.0_f32, egui::Color32::BLUE),
                                );
                            }
                        }
                        Element::FormField(ff) => {
                            let min_p = mm_to_screen(ff.x.0, ff.y.0 + ff.height.0);
                            let max_p = mm_to_screen(ff.x.0 + ff.width.0, ff.y.0);
                            let elem_rect = egui::Rect::from_min_max(min_p, max_p);

                            painter.rect_filled(
                                elem_rect,
                                0.0,
                                egui::Color32::from_rgb(230, 240, 255),
                            );
                            painter.rect_stroke(
                                elem_rect,
                                0.0,
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(100, 150, 240)),
                            );

                            painter.text(
                                elem_rect.left_top() + egui::vec2(4.0, 2.0),
                                egui::Align2::LEFT_TOP,
                                format!("[{}]", ff.field_tag),
                                egui::FontId::proportional(10.0 * app.zoom),
                                egui::Color32::from_rgb(0, 80, 180),
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

                // 4. Live Rubber-Band Line Creation Preview & Single Hover Coordinates
                match app.tool_state {
                    ToolState::LineWaitStart => {
                        if let Some(pointer_pos) = ctx.pointer_interact_pos() {
                            if page_rect.contains(pointer_pos) {
                                let (curr_x, curr_y) = screen_to_mm(pointer_pos);
                                draw_coord_badge(&painter, pointer_pos, &format!("{}, {}", curr_x, curr_y));
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

                                draw_coord_badge(&painter, p1, &format!("{}, {}", start_x, start_y));
                                draw_coord_badge(&painter, p2, &format!("{}, {}", curr_x, curr_y));
                            }
                        }
                    }
                    ToolState::Select => {}
                }

                // 5. Mouse Press & Drag Handling with Single Hover Coordinate Display
                if let Some(pointer_pos) = ctx.pointer_interact_pos() {
                    if page_rect.contains(pointer_pos) {
                        let (mouse_x, mouse_y) = screen_to_mm(pointer_pos);

                        // Show mouse hover coordinate only when NOT in line creation mode to prevent duplication
                        if app.tool_state == ToolState::Select && app.active_line_handle.is_none() {
                            draw_coord_badge(&painter, pointer_pos, &format!("{}, {}", mouse_x, mouse_y));
                        }

                        // MOUSE PRESS DOWN EVENT
                        if ctx.input(|i| i.pointer.primary_pressed()) {
                            match app.tool_state {
                                ToolState::LineWaitStart => {
                                    app.tool_state = ToolState::LineWaitEnd {
                                        start_x: mouse_x,
                                        start_y: mouse_y,
                                    };
                                    app.status_msg = format!("始点を指定しました ({}, {}) mm。終点をクリックしてください", mouse_x, mouse_y);
                                }
                                ToolState::LineWaitEnd { start_x, start_y } => {
                                    let id = app.mgr.doc.next_id();
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
                                    app.mgr.execute(Command::AddElement(line.clone()));
                                    app.selected_ids.clear();
                                    app.selected_ids.insert(id);
                                    app.tool_state = ToolState::Select;
                                    app.status_msg = format!("直線 #{} を作成しました", id.0);
                                }
                                ToolState::Select => {
                                    let mut hit_handle = None;
                                    let mut hit_id = None;

                                    // First check Start / End vertex handle hit for selected lines!
                                    for id in app.selected_ids.iter().cloned() {
                                        if let Some(Element::Line(l)) = app.mgr.doc.get_element(id) {
                                            let p1 = mm_to_screen(l.x1.0, l.y1.0);
                                            let p2 = mm_to_screen(l.x2.0, l.y2.0);

                                            if pointer_pos.distance(p1) <= 10.0 {
                                                hit_handle = Some((id, LineHandleKind::StartPoint));
                                                break;
                                            } else if pointer_pos.distance(p2) <= 10.0 {
                                                hit_handle = Some((id, LineHandleKind::EndPoint));
                                                break;
                                            }
                                        }
                                    }

                                    if let Some(handle_info) = hit_handle {
                                        app.active_line_handle = Some(handle_info);
                                        app.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                                    } else {
                                        // Standard element hit testing (reverse z-order)
                                        for elem in app.mgr.doc.elements.iter().rev() {
                                            let bounds = elem.bounds();
                                            if mouse_x >= bounds.x.0
                                                && mouse_x <= bounds.x.0 + bounds.width.0
                                                && mouse_y >= bounds.y.0
                                                && mouse_y <= bounds.y.0 + bounds.height.0
                                            {
                                                hit_id = Some(elem.id());
                                                break;
                                            }
                                        }

                                        if let Some(id) = hit_id {
                                            if !ctx.input(|i| i.modifiers.shift) {
                                                app.selected_ids.clear();
                                            }
                                            app.selected_ids.insert(id);
                                            app.status_msg = format!("パーツ #{} を選択しました", id.0);

                                            if let Some(Element::Line(_)) = app.mgr.doc.get_element(id) {
                                                app.active_line_handle = Some((id, LineHandleKind::Body));
                                            }
                                        } else if !ctx.input(|i| i.modifiers.shift) {
                                            app.selected_ids.clear();
                                            app.status_msg = "選択を解除しました".to_string();
                                        }
                                        app.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                                    }
                                }
                            }
                        }

                        // MOUSE DRAGGING EVENT
                        if ctx.input(|i| i.pointer.primary_down()) {
                            if let Some((last_x, last_y)) = app.last_drag_mm {
                                let dx = (mouse_x as i32) - last_x;
                                let dy = (mouse_y as i32) - last_y;

                                if dx != 0 || dy != 0 {
                                    if let Some((line_id, handle_kind)) = app.active_line_handle {
                                        if let Some(Element::Line(mut l)) = app.mgr.doc.get_element(line_id).cloned() {
                                            let old_elem = Element::Line(l.clone());
                                            match handle_kind {
                                                LineHandleKind::StartPoint => {
                                                    l.x1 = Mm::new(mouse_x);
                                                    l.y1 = Mm::new(mouse_y);
                                                    app.status_msg = format!("始点移動: ({}, {}) mm", mouse_x, mouse_y);
                                                }
                                                LineHandleKind::EndPoint => {
                                                    l.x2 = Mm::new(mouse_x);
                                                    l.y2 = Mm::new(mouse_y);
                                                    app.status_msg = format!("終点移動: ({}, {}) mm", mouse_x, mouse_y);
                                                }
                                                LineHandleKind::Body => {
                                                    let mut elem = Element::Line(l.clone());
                                                    elem.translate(dx, dy);
                                                    if let Element::Line(new_l) = elem {
                                                        l = new_l;
                                                    }
                                                    app.status_msg = format!("直線移動: dx={}mm, dy={}mm", dx, dy);
                                                }
                                            }
                                            app.mgr.execute(Command::UpdateElement {
                                                old: old_elem,
                                                new: Element::Line(l),
                                            });
                                            app.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                                        }
                                    } else {
                                        // Move non-line elements
                                        for id in app.selected_ids.clone() {
                                            app.mgr.execute(Command::MoveElement { id, dx, dy });
                                        }
                                        app.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                                        app.status_msg = format!("移動: dx={}mm, dy={}mm", dx, dy);
                                    }
                                }
                            } else {
                                app.last_drag_mm = Some((mouse_x as i32, mouse_y as i32));
                            }

                            // Display live coordinate tooltip badge while dragging line handles or elements!
                            if let Some((line_id, handle_kind)) = app.active_line_handle {
                                if let Some(Element::Line(l)) = app.mgr.doc.get_element(line_id) {
                                    match handle_kind {
                                        LineHandleKind::StartPoint => {
                                            let p = mm_to_screen(l.x1.0, l.y1.0);
                                            draw_coord_badge(&painter, p, &format!("{}, {}", l.x1.0, l.y1.0));
                                        }
                                        LineHandleKind::EndPoint => {
                                            let p = mm_to_screen(l.x2.0, l.y2.0);
                                            draw_coord_badge(&painter, p, &format!("{}, {}", l.x2.0, l.y2.0));
                                        }
                                        LineHandleKind::Body => {
                                            let p1 = mm_to_screen(l.x1.0, l.y1.0);
                                            draw_coord_badge(&painter, p1, &format!("{}, {} | {}, {}", l.x1.0, l.y1.0, l.x2.0, l.y2.0));
                                        }
                                    }
                                }
                            }
                        } else if !ctx.input(|i| i.pointer.primary_down()) {
                            app.last_drag_mm = None;
                            app.active_line_handle = None;
                        }
                    }
                }
            });
    });
}
