use image::{codecs::jpeg::JpegEncoder, GenericImageView, ImageFormat, ImageReader};
use std::io::Cursor;

pub fn compress_and_resize(
    data: &[u8],
    max_height: u32,
    quality: u8,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let img = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    let (width, height) = img.dimensions();

    if height > max_height {
        let ratio = max_height as f64 / height as f64;
        let new_width = (width as f64 * ratio) as u32;
        let resized = img.resize(new_width, max_height, image::imageops::FilterType::Lanczos3);
        encode_as_jpeg(&resized, quality)
    } else {
        encode_as_jpeg(&img, quality)
    }
}

fn encode_as_jpeg(
    img: &image::DynamicImage,
    quality: u8,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut output = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut output, quality);
    encoder.encode_image(img)?;
    Ok(output)
}

pub fn is_already_compressed(data: &[u8], max_height: u32) -> bool {
    match ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.decode().ok())
    {
        Some(img) => {
            let (_, height) = img.dimensions();
            let is_jpeg = matches!(
                ImageReader::new(Cursor::new(data))
                    .with_guessed_format()
                    .ok()
                    .and_then(|r| r.format()),
                Some(ImageFormat::Jpeg)
            );
            height <= max_height && is_jpeg && data.len() < 500_000
        }
        None => false,
    }
}
