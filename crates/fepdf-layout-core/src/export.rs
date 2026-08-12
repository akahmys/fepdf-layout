//! PDF 2.0 Export integration mapping fepdf-layout Document to PDF facade operations.

use crate::document::Document;
use crate::element::Element;
use crate::units::POINTS_PER_INCH;
use fepdf_sdk as fepdf;
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
                        "  [Line] ({:.2}, {:.2}) pt -> ({:.2}, {:.2}) pt, cap: {:?}\n",
                        p1x, p1y, p2x, p2y, l.line_cap
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

    /// Generate valid PDF 2.0 binary document bytes from the Document model.
    pub fn generate_pdf_bytes(doc: &Document) -> Vec<u8> {
        let paper_w_pt = doc.page_spec.paper_width.to_pt();
        let paper_h_pt = doc.page_spec.paper_height.to_pt();

        let mut stream_content = String::new();

        for elem in &doc.elements {
            match elem {
                Element::Line(l) => {
                    let (x1_pt, y1_pt) = Self::layout_to_pdf_pt(doc, l.x1.0, l.y1.0);
                    let (x2_pt, y2_pt) = Self::layout_to_pdf_pt(doc, l.x2.0, l.y2.0);
                    let stroke_w_pt = l.stroke_width.to_pt();
                    let cap_code = match l.line_cap {
                        crate::element::LineCap::Butt => 0,
                        crate::element::LineCap::Round => 1,
                        crate::element::LineCap::Square => 2,
                    };
                    let r = l.stroke_color.r as f64 / 255.0;
                    let g = l.stroke_color.g as f64 / 255.0;
                    let b = l.stroke_color.b as f64 / 255.0;

                    stream_content.push_str(&format!(
                        "{:.2} w {} J {:.3} {:.3} {:.3} RG {:.2} {:.2} m {:.2} {:.2} l S\n",
                        stroke_w_pt, cap_code, r, g, b, x1_pt, y1_pt, x2_pt, y2_pt
                    ));
                }
                Element::TextBox(t) => {
                    let (px, py) = Self::layout_to_pdf_pt(doc, t.x.0, t.y.0);
                    let r = t.text_color.r as f64 / 255.0;
                    let g = t.text_color.g as f64 / 255.0;
                    let b = t.text_color.b as f64 / 255.0;
                    let clean_text = t.text.replace('(', "\\(").replace(')', "\\)");

                    stream_content.push_str(&format!(
                        "BT /F1 {:.1} Tf {:.3} {:.3} {:.3} rg {:.2} {:.2} Td ({}) Tj ET\n",
                        t.font_size_pt, r, g, b, px, py + t.font_size_pt * 0.8, clean_text
                    ));
                }
                Element::FormField(f) => {
                    let (px, py) = Self::layout_to_pdf_pt(doc, f.x.0, f.y.0);
                    let (w_pt, h_pt) = (f.width.to_pt(), f.height.to_pt());
                    stream_content.push_str(&format!(
                        "0.7 0.7 0.7 RG 1.0 w {:.2} {:.2} {:.2} {:.2} re S\n",
                        px, py, w_pt, h_pt
                    ));
                }
            }
        }

        let stream_bytes = stream_content.as_bytes();
        let stream_len = stream_bytes.len();

        let mut pdf = Vec::new();
        pdf.extend_from_slice(b"%PDF-2.0\n%\xC2\xE5\xC2\xF2\n");

        let mut offsets = Vec::new();

        // Obj 1: Catalog
        offsets.push(pdf.len());
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Obj 2: Pages
        offsets.push(pdf.len());
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        // Obj 3: Page
        offsets.push(pdf.len());
        let page_obj = format!(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>\nendobj\n",
            paper_w_pt, paper_h_pt
        );
        pdf.extend_from_slice(page_obj.as_bytes());

        // Obj 4: Stream
        offsets.push(pdf.len());
        let stream_header = format!("4 0 obj\n<< /Length {} >>\nstream\n", stream_len);
        pdf.extend_from_slice(stream_header.as_bytes());
        pdf.extend_from_slice(stream_bytes);
        pdf.extend_from_slice(b"\nendstream\nendobj\n");

        // Obj 5: Font
        offsets.push(pdf.len());
        pdf.extend_from_slice(b"5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");

        // XRef Table
        let xref_offset = pdf.len();
        pdf.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
        for &off in &offsets {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }

        // Trailer
        let trailer = format!(
            "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            xref_offset
        );
        pdf.extend_from_slice(trailer.as_bytes());

        pdf
    }

    /// Export layout document to a valid PDF 2.0 file on disk,
    /// verified through the `fepdf` core engine facade.
    pub fn export_to_file(doc: &Document, path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let pdf_bytes = Self::generate_pdf_bytes(doc);

        // Verify generated PDF bytes using fepdf core engine facade
        let _pdf_doc = fepdf::Document::open(
            bytes::Bytes::from(pdf_bytes.clone()),
            &fepdf_core::ingest::IngestionOptions::default(),
        ).map_err(|e| format!("fepdf engine validation failed: {:?}", e))?;

        std::fs::write(path, pdf_bytes).map_err(|e| e.to_string())
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

    #[test]
    fn test_pdf_binary_generation_header() {
        let doc = Document::new(PagePreset::A4);
        let bytes = PdfExporter::generate_pdf_bytes(&doc);
        assert!(bytes.starts_with(b"%PDF-2.0\n"));
        assert!(bytes.ends_with(b"%%EOF\n"));
    }
}
