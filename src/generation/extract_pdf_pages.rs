use std::path::Path;
use tokio::task::JoinSet;

#[cfg(feature = "ssr")]
use lopdf::Document;

use crate::generation::generate_pdf::{PdfPagePair, create_pdf_page_pairs};

#[derive(Debug)]
pub enum ExtractionError {
    InvalidPdfPath(String),
    OddPageCount(usize),
    IoError(std::io::Error),
    ExtractionFailed(String),
    PdfProcessingError(String),
    UnreadablePageDimensions(usize),
    WrongPageSize {
        page_num: usize,
        width_cm: f64,
        height_cm: f64,
    },
    InconsistentPageSizes {
        page_num: usize,
        width_cm: f64,
        height_cm: f64,
        ref_width_cm: f64,
        ref_height_cm: f64,
    },
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExtractionError::InvalidPdfPath(path) => write!(f, "Invalid PDF path: {}", path),
            ExtractionError::OddPageCount(count) => {
                write!(
                    f,
                    "PDF has odd number of pages ({}). Expected even number for recto-verso pairs.",
                    count
                )
            }
            ExtractionError::IoError(e) => write!(f, "IO Error: {}", e),
            ExtractionError::ExtractionFailed(msg) => write!(f, "Extraction failed: {}", msg),
            ExtractionError::PdfProcessingError(msg) => write!(f, "PDF processing error: {}", msg),
            ExtractionError::UnreadablePageDimensions(page_num) => {
                write!(f, "Could not read dimensions of page {}", page_num)
            }
            ExtractionError::WrongPageSize {
                page_num,
                width_cm,
                height_cm,
            } => {
                write!(
                    f,
                    "Page {} is {:.2}cm × {:.2}cm. Expected 5cm × 15cm bookmark size.",
                    page_num, width_cm, height_cm
                )
            }
            ExtractionError::InconsistentPageSizes {
                page_num,
                width_cm,
                height_cm,
                ref_width_cm,
                ref_height_cm,
            } => {
                write!(
                    f,
                    "Page {} ({:.2}cm × {:.2}cm) differs from page 1 ({:.2}cm × {:.2}cm). All pages must be the same size.",
                    page_num, width_cm, height_cm, ref_width_cm, ref_height_cm
                )
            }
        }
    }
}

impl std::error::Error for ExtractionError {}

impl From<std::io::Error> for ExtractionError {
    fn from(error: std::io::Error) -> Self {
        ExtractionError::IoError(error)
    }
}

/// Returns the (width, height) in points of a page, following parent-chain inheritance
/// for MediaBox (which may be set on the Pages root rather than each individual page).
#[cfg(feature = "ssr")]
fn get_page_media_box(doc: &Document, page_id: lopdf::ObjectId) -> Option<(f64, f64)> {
    let mut current_id = page_id;
    loop {
        let dict = match doc.get_object(current_id).ok()? {
            lopdf::Object::Dictionary(d) => d,
            lopdf::Object::Stream(s) => &s.dict,
            _ => return None,
        };

        if let Some(mb) = dict.get(b"MediaBox") {
            if let lopdf::Object::Array(arr) = mb {
                if arr.len() >= 4 {
                    let to_f64 = |obj: &lopdf::Object| -> Option<f64> {
                        match obj {
                            lopdf::Object::Integer(i) => Some(*i as f64),
                            lopdf::Object::Real(r) => Some(*r as f64),
                            _ => None,
                        }
                    };
                    let llx = to_f64(&arr[0])?;
                    let lly = to_f64(&arr[1])?;
                    let urx = to_f64(&arr[2])?;
                    let ury = to_f64(&arr[3])?;
                    return Some(((urx - llx).abs(), (ury - lly).abs()));
                }
            }
        }

        match dict.get(b"Parent") {
            Some(lopdf::Object::Reference(parent_id)) => current_id = *parent_id,
            _ => return None,
        }
    }
}

/// Validates that all pages in the document have:
/// 1. Dimensions matching 5cm × 15cm (±0.5cm tolerance)
/// 2. The same dimensions as the first page (±1pt tolerance)
#[cfg(feature = "ssr")]
fn validate_page_sizes(doc: &Document) -> Result<(), ExtractionError> {
    const CM: f64 = 28.3465; // points per cm
    const EXPECTED_W: f64 = 5.0 * CM; // 141.73pt
    const EXPECTED_H: f64 = 15.0 * CM; // 425.20pt
    const SIZE_TOL: f64 = 14.17; // ±0.5cm
    const UNIFORM_TOL: f64 = 1.0; // ±1pt

    let pages = doc.get_pages();
    let mut sorted: Vec<_> = pages.iter().collect();
    sorted.sort_by_key(|(n, _)| *n);

    let mut reference: Option<(f64, f64)> = None;

    for (&page_num, &page_id) in &sorted {
        let (w, h) = get_page_media_box(doc, page_id)
            .ok_or(ExtractionError::UnreadablePageDimensions(page_num as usize))?;

        match reference {
            None => {
                // Normalize to portrait orientation before checking expected size
                let (pw, ph) = if w <= h { (w, h) } else { (h, w) };
                if (pw - EXPECTED_W).abs() > SIZE_TOL || (ph - EXPECTED_H).abs() > SIZE_TOL {
                    return Err(ExtractionError::WrongPageSize {
                        page_num: page_num as usize,
                        width_cm: pw / CM,
                        height_cm: ph / CM,
                    });
                }
                reference = Some((w, h));
            }
            Some((rw, rh)) => {
                if (w - rw).abs() > UNIFORM_TOL || (h - rh).abs() > UNIFORM_TOL {
                    return Err(ExtractionError::InconsistentPageSizes {
                        page_num: page_num as usize,
                        width_cm: w / CM,
                        height_cm: h / CM,
                        ref_width_cm: rw / CM,
                        ref_height_cm: rh / CM,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Copy a PDF file to the output directory and create page pairs
async fn process_pdf_file<P: AsRef<Path>>(
    pdf_path: P,
) -> Result<Vec<PdfPagePair>, ExtractionError> {
    let pdf_path = pdf_path.as_ref();

    if !pdf_path.exists() {
        return Err(ExtractionError::InvalidPdfPath(
            pdf_path.to_string_lossy().to_string(),
        ));
    }

    let document = Document::load(pdf_path)
        .map_err(|e| ExtractionError::PdfProcessingError(format!("Failed to load PDF: {}", e)))?;

    let page_count = document.get_pages().len();

    if page_count % 2 != 0 {
        return Err(ExtractionError::OddPageCount(page_count));
    }

    validate_page_sizes(&document)?;

    let pairs = create_pdf_page_pairs(pdf_path, page_count)
        .map_err(|e| ExtractionError::PdfProcessingError(e.to_string()))?;

    Ok(pairs)
}

/// Process multiple PDF files and create page pairs for each
pub async fn split_pages_from_input_pdfs(
    pdf_files: &[String],
) -> Result<Vec<PdfPagePair>, ExtractionError> {
    if pdf_files.is_empty() {
        return Ok(Vec::new());
    }
    let mut all_pairs = Vec::new();
    let mut join_set = JoinSet::new();

    // Process all PDFs concurrently
    for pdf_path in pdf_files {
        let pdf_path = pdf_path.clone();

        join_set.spawn(async move { process_pdf_file(pdf_path.as_str()).await });
    }

    // Collect results
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(mut pairs)) => all_pairs.append(&mut pairs),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(ExtractionError::ExtractionFailed(e.to_string())),
        }
    }

    if all_pairs.is_empty() {
        return Err(ExtractionError::ExtractionFailed(
            "No valid page pairs created".to_string(),
        ));
    }

    Ok(all_pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_pdf_with_pages(mediabox: Vec<lopdf::Object>, page_count: usize) -> lopdf::Document {
        use lopdf::{Object, dictionary};

        let mut doc = lopdf::Document::with_version("1.4");

        let pages_id = doc.new_object_id();
        let mut kids = Vec::new();
        let mut page_ids = Vec::new();
        for _ in 0..page_count {
            page_ids.push(doc.new_object_id());
        }

        for &page_id in &page_ids {
            kids.push(Object::Reference(page_id));
        }

        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => kids,
                "Count" => page_count as i64,
            }),
        );

        for &page_id in &page_ids {
            doc.objects.insert(
                page_id,
                Object::Dictionary(dictionary! {
                    "Type" => "Page",
                    "Parent" => Object::Reference(pages_id),
                    "MediaBox" => mediabox.clone(),
                }),
            );
        }

        let catalog_id = doc.new_object_id();
        doc.objects.insert(
            catalog_id,
            Object::Dictionary(dictionary! {
                "Type" => "Catalog",
                "Pages" => Object::Reference(pages_id),
            }),
        );
        doc.trailer.set("Root", Object::Reference(catalog_id));
        doc
    }

    // 5cm × 15cm in points (rounded to integers)
    const BOOKMARK_W: i64 = 142; // ~141.73pt
    const BOOKMARK_H: i64 = 425; // ~425.20pt

    #[test]
    fn test_validate_page_sizes_correct() {
        let mediabox = vec![
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(BOOKMARK_W),
            lopdf::Object::Integer(BOOKMARK_H),
        ];
        let doc = make_pdf_with_pages(mediabox, 2);
        assert!(validate_page_sizes(&doc).is_ok());
    }

    #[test]
    fn test_validate_page_sizes_wrong_size() {
        // A4 page (595pt × 842pt ≈ 21cm × 29.7cm)
        let mediabox = vec![
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(595),
            lopdf::Object::Integer(842),
        ];
        let doc = make_pdf_with_pages(mediabox, 2);
        assert!(matches!(
            validate_page_sizes(&doc),
            Err(ExtractionError::WrongPageSize { .. })
        ));
    }

    #[test]
    fn test_validate_page_sizes_inconsistent() {
        use lopdf::{Object, dictionary};

        let mut doc = lopdf::Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let page1_id = doc.new_object_id();
        let page2_id = doc.new_object_id();

        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![Object::Reference(page1_id), Object::Reference(page2_id)],
                "Count" => 2,
            }),
        );
        doc.objects.insert(
            page1_id,
            Object::Dictionary(dictionary! {
                "Type" => "Page",
                "Parent" => Object::Reference(pages_id),
                "MediaBox" => vec![Object::Integer(0), Object::Integer(0),
                                   Object::Integer(BOOKMARK_W), Object::Integer(BOOKMARK_H)],
            }),
        );
        // Page 2 has a different height
        doc.objects.insert(
            page2_id,
            Object::Dictionary(dictionary! {
                "Type" => "Page",
                "Parent" => Object::Reference(pages_id),
                "MediaBox" => vec![Object::Integer(0), Object::Integer(0),
                                   Object::Integer(BOOKMARK_W), Object::Integer(BOOKMARK_H + 50)],
            }),
        );
        let catalog_id = doc.new_object_id();
        doc.objects.insert(
            catalog_id,
            Object::Dictionary(dictionary! {
                "Type" => "Catalog",
                "Pages" => Object::Reference(pages_id),
            }),
        );
        doc.trailer.set("Root", Object::Reference(catalog_id));

        assert!(matches!(
            validate_page_sizes(&doc),
            Err(ExtractionError::InconsistentPageSizes { .. })
        ));
    }

    #[test]
    fn test_process_pdf_file_correct_size() {
        let temp_dir = TempDir::new().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");

        let mediabox = vec![
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(BOOKMARK_W),
            lopdf::Object::Integer(BOOKMARK_H),
        ];
        let doc = make_pdf_with_pages(mediabox, 2);
        doc.save(&pdf_path).unwrap();

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(process_pdf_file(&pdf_path));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_create_pairs_even_pages() {
        let pdf_path = Path::new("test.pdf");
        let pairs = create_pdf_page_pairs(pdf_path, 4).unwrap();

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].recto_page, 1);
        assert_eq!(pairs[0].verso_page, 2);
        assert_eq!(pairs[1].recto_page, 3);
        assert_eq!(pairs[1].verso_page, 4);
    }

    #[test]
    fn test_create_pairs_odd_pages_error() {
        let pdf_path = Path::new("test.pdf");
        let result = create_pdf_page_pairs(pdf_path, 3);

        assert!(result.is_err());
    }
}
