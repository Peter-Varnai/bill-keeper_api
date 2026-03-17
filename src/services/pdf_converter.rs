use image::ImageFormat;
use std::io::Cursor;

const DEFAULT_DPI: u32 = 150;

pub fn convert_pdf_to_jpg(
    pdf_data: &[u8],
    dpi: u32,
    _quality: u8,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let temp_dir = std::env::temp_dir();
    let temp_pdf = temp_dir.join("temp_upload.pdf");

    std::fs::write(&temp_pdf, pdf_data)?;

    let pdf = pdf2image::PDF::from_file(&temp_pdf)?;

    let pages = pdf.render(
        pdf2image::Pages::Range(1..=1),
        pdf2image::RenderOptionsBuilder::default()
            .resolution(pdf2image::DPI::Uniform(dpi))
            .pdftocairo(true)
            .build()?,
    )?;

    if let Some(first_page) = pages.into_iter().next() {
        let mut jpg_data = Vec::new();
        let mut cursor = Cursor::new(&mut jpg_data);

        first_page.write_to(&mut cursor, ImageFormat::Jpeg)?;

        let _ = std::fs::remove_file(temp_pdf);

        return Ok(jpg_data);
    }

    let _ = std::fs::remove_file(temp_pdf);
    Err("No pages found in PDF".into())
}

pub fn convert_pdf_to_jpg_with_defaults(
    pdf_data: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    convert_pdf_to_jpg(pdf_data, DEFAULT_DPI, 95)
}

pub fn should_convert_to_jpg(filename: &str) -> bool {
    filename.to_lowercase().ends_with(".pdf")
}

pub fn replace_extension_with_jpg(filename: &str) -> String {
    if filename.to_lowercase().ends_with(".pdf") {
        format!(
            "{}.jpg",
            filename.trim_end_matches(".pdf").trim_end_matches(".PDF")
        )
    } else {
        filename.to_string()
    }
}
