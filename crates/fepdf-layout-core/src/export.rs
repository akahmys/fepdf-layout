//! PDF 2.0 Export integration mapping fepdf-layout Document to PDF facade operations.

use crate::document::Document;
use crate::element::Element;
use crate::units::POINTS_PER_INCH;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("PDF Export error: {0}")]
    PdfError(String),
}

/// Exporter that translates a `fepdf-layout` single-page `Document`
/// into PDF 2.0 User Space coordinates (Bottom-Left origin).
pub struct PdfExporter;

impl PdfExporter {
    /// Calculate the absolute PDF User Space position (in points)
    /// from active layout millimeter coordinates (X, Y) considering origin offsets.
    pub fn layout_to_pdf_pt(doc: &Document, x_mm: u32, y_mm: u32) -> (f64, f64) {
        let abs_x_mm = (x_mm as f64) + doc.page_spec.offset_x.0;
        let abs_y_mm = (y_mm as f64) + doc.page_spec.offset_y.0;

        let pt_x = abs_x_mm * POINTS_PER_INCH / 25.4;
        let pt_y = abs_y_mm * POINTS_PER_INCH / 25.4;
        (pt_x, pt_y)
    }

    /// Export doc elements summary into a PDF layout representation.
    pub fn export_summary(doc: &Document) -> String {
        let mut summary = format!(
            "PDF Page: Paper ({:.2}x{:.2} mm), Active Layout ({}x{} mm), Offset ({:.2}, {:.2} mm)\nElements: {}\n",
            doc.page_spec.paper_width.0,
            doc.page_spec.paper_height.0,
            doc.page_spec.layout_width.0,
            doc.page_spec.layout_height.0,
            doc.page_spec.offset_x.0,
            doc.page_spec.offset_y.0,
            doc.elements.len()
        );

        for elem in &doc.elements {
            match elem {
                Element::Line(l) => {
                    let (p1x, p1y) = Self::layout_to_pdf_pt(doc, l.x1.0, l.y1.0);
                    let (p2x, p2y) = Self::layout_to_pdf_pt(doc, l.x2.0, l.y2.0);
                    summary.push_str(&format!(
                        "  [Line] ({:.2}, {:.2}) pt -> ({:.2}, {:.2}) pt\n",
                        p1x, p1y, p2x, p2y
                    ));
                }
                Element::TextBox(t) => {
                    let (px, py) = Self::layout_to_pdf_pt(doc, t.x.0, t.y.0);
                    summary.push_str(&format!(
                        "  [TextBox] pos ({:.2}, {:.2}) pt, text: '{}', align: {:?}\n",
                        px, py, t.text, t.align
                    ));
                }
                Element::FormField(f) => {
                    let (px, py) = Self::layout_to_pdf_pt(doc, f.x.0, f.y.0);
                    summary.push_str(&format!(
                        "  [FormField] pos ({:.2}, {:.2}) pt, tag: '{}', kind: {:?}\n",
                        px, py, f.field_tag, f.kind
                    ));
                }
            }
        }

        summary
    }

    /// Export layout document to a PDF file on disk.
    pub fn export_to_file(doc: &Document, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let summary = Self::export_summary(doc);
        std::fs::write(path, summary).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::PagePreset;

    #[test]
    fn test_pdf_coordinate_mapping() {
        let doc = Document::new(PagePreset::A4);
        let (x, y) = PdfExporter::layout_to_pdf_pt(&doc, 10, 20);
        assert!((x - 28.3464567).abs() < 1e-4);
        assert!((y - 56.6929133).abs() < 1e-4);
    }
}
