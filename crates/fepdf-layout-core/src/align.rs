//! Multi-selection alignment algorithms with 1mm grid snapping.

use crate::element::Element;

/// Alignment modes for multi-selected elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignMode {
    /// Align left edges to the minimum X coordinate.
    Left,
    /// Align horizontal centers to the average center X coordinate.
    Center,
    /// Align right edges to the maximum right X coordinate.
    Right,
    /// Align bottom edges to the minimum Y coordinate (Bottom-Left origin).
    Bottom,
    /// Align vertical middles to the average middle Y coordinate.
    Middle,
    /// Align top edges to the maximum top Y coordinate.
    Top,
}

/// Align a set of elements according to `AlignMode`.
/// Returns the updated elements with positions snapped to integer millimeters.
pub fn align_elements(elements: &mut [Element], mode: AlignMode) {
    if elements.len() < 2 {
        return;
    }

    let bounds: Vec<_> = elements.iter().map(|e| e.bounds()).collect();

    match mode {
        AlignMode::Left => {
            let target_x = bounds.iter().map(|b| b.x.0).min().unwrap_or(0);
            for (elem, b) in elements.iter_mut().zip(bounds.iter()) {
                let dx = (target_x as i32) - (b.x.0 as i32);
                elem.translate(dx, 0);
            }
        }
        AlignMode::Right => {
            let target_right = bounds.iter().map(|b| b.x.0 + b.width.0).max().unwrap_or(0);
            for (elem, b) in elements.iter_mut().zip(bounds.iter()) {
                let elem_right = b.x.0 + b.width.0;
                let dx = (target_right as i32) - (elem_right as i32);
                elem.translate(dx, 0);
            }
        }
        AlignMode::Center => {
            let avg_center: f64 = bounds
                .iter()
                .map(|b| (b.x.0 as f64) + (b.width.0 as f64) / 2.0)
                .sum::<f64>()
                / (bounds.len() as f64);
            for (elem, b) in elements.iter_mut().zip(bounds.iter()) {
                let elem_center = (b.x.0 as f64) + (b.width.0 as f64) / 2.0;
                let dx = (avg_center - elem_center).round() as i32;
                elem.translate(dx, 0);
            }
        }
        AlignMode::Bottom => {
            let target_y = bounds.iter().map(|b| b.y.0).min().unwrap_or(0);
            for (elem, b) in elements.iter_mut().zip(bounds.iter()) {
                let dy = (target_y as i32) - (b.y.0 as i32);
                elem.translate(0, dy);
            }
        }
        AlignMode::Top => {
            let target_top = bounds.iter().map(|b| b.y.0 + b.height.0).max().unwrap_or(0);
            for (elem, b) in elements.iter_mut().zip(bounds.iter()) {
                let elem_top = b.y.0 + b.height.0;
                let dy = (target_top as i32) - (elem_top as i32);
                elem.translate(0, dy);
            }
        }
        AlignMode::Middle => {
            let avg_middle: f64 = bounds
                .iter()
                .map(|b| (b.y.0 as f64) + (b.height.0 as f64) / 2.0)
                .sum::<f64>()
                / (bounds.len() as f64);
            for (elem, b) in elements.iter_mut().zip(bounds.iter()) {
                let elem_middle = (b.y.0 as f64) + (b.height.0 as f64) / 2.0;
                let dy = (avg_middle - elem_middle).round() as i32;
                elem.translate(0, dy);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{ElementId, TextBoxElement, Color, TextAlign};
    use crate::units::Mm;

    fn make_textbox(id: u64, x: u32, y: u32, w: u32, h: u32) -> Element {
        Element::TextBox(TextBoxElement {
            id: ElementId(id),
            x: Mm::new(x),
            y: Mm::new(y),
            width: Mm::new(w),
            height: Mm::new(h),
            text: "Test".to_string(),
            font_family: "Sans".to_string(),
            font_size_pt: 10.0,
            text_color: Color::BLACK,
            align: TextAlign::Left,
            horizontal_scaling: 100,
            auto_fit_horizontal: false,
        })
    }

    #[test]
    fn test_align_left() {
        let mut elems = vec![
            make_textbox(1, 10, 20, 30, 40),
            make_textbox(2, 50, 60, 30, 40),
        ];
        align_elements(&mut elems, AlignMode::Left);
        assert_eq!(elems[0].bounds().x, Mm::new(10));
        assert_eq!(elems[1].bounds().x, Mm::new(10));
    }
}
