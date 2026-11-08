//! PDF text extraction.

/// Extracted content from a PDF document.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedPdf {
    pub text: String,
    pub page_count: Option<usize>,
    pub extraction_method: String,
}

/// Extract text from PDF bytes.
pub fn extract_pdf_text(bytes: &[u8]) -> Result<ExtractedPdf, String> {
    match pdf_extract::extract_text_from_mem(bytes) {
        Ok(text) => {
            let cleaned = text.trim().to_string();
            Ok(ExtractedPdf {
                text: cleaned,
                page_count: None, // pdf-extract doesn't expose page count easily
                extraction_method: "pdf-extract".into(),
            })
        }
        Err(e) => Err(format!("PDF extraction failed: {}", e)),
    }
}
