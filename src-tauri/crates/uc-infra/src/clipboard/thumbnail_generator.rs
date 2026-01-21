use anyhow::{Context, Result};
use async_trait::async_trait;
use image::{imageops::FilterType, ColorType, GenericImageView};
use uc_core::clipboard::MimeType;
use uc_core::ports::clipboard::{GeneratedThumbnail, ThumbnailGeneratorPort};

pub struct InfraThumbnailGenerator {
    max_edge: u32,
}

impl InfraThumbnailGenerator {
    pub fn new(max_edge: u32) -> Self {
        Self { max_edge }
    }
}

#[async_trait]
impl ThumbnailGeneratorPort for InfraThumbnailGenerator {
    async fn generate_thumbnail(&self, image_bytes: &[u8]) -> Result<GeneratedThumbnail> {
        let decoded =
            image::load_from_memory(image_bytes).context("decode image bytes for thumbnail")?;
        let (original_width, original_height) = decoded.dimensions();
        let (target_width, target_height) =
            calculate_target_size(original_width, original_height, self.max_edge);

        let resized = if target_width == original_width && target_height == original_height {
            decoded
        } else {
            image::DynamicImage::ImageRgba8(image::imageops::resize(
                &decoded,
                target_width,
                target_height,
                FilterType::Triangle,
            ))
        };

        let rgba = resized.to_rgba8();
        let (thumbnail_width, thumbnail_height) = rgba.dimensions();
        let mut thumbnail_bytes = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut thumbnail_bytes);
        encoder
            .encode(
                rgba.as_raw(),
                thumbnail_width,
                thumbnail_height,
                ColorType::Rgba8.into(),
            )
            .context("encode thumbnail to webp")?;

        Ok(GeneratedThumbnail {
            thumbnail_bytes,
            thumbnail_mime_type: MimeType("image/webp".to_string()),
            original_width: i32::try_from(original_width)
                .context("original width exceeds i32 range")?,
            original_height: i32::try_from(original_height)
                .context("original height exceeds i32 range")?,
        })
    }
}

fn calculate_target_size(width: u32, height: u32, max_edge: u32) -> (u32, u32) {
    if width <= max_edge && height <= max_edge {
        return (width, height);
    }

    if width >= height {
        let scaled_height = ((height as f64) * (max_edge as f64) / (width as f64)).round() as u32;
        (max_edge, scaled_height.max(1))
    } else {
        let scaled_width = ((width as f64) * (max_edge as f64) / (height as f64)).round() as u32;
        (scaled_width.max(1), max_edge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thumbnail_generator_resizes_to_max_edge() {
        let image = image::RgbImage::new(256, 128);
        let mut png_bytes = Vec::new();
        image::DynamicImage::ImageRgb8(image)
            .write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .unwrap();

        let generator = InfraThumbnailGenerator::new(128);
        let output = generator.generate_thumbnail(&png_bytes).await.unwrap();

        assert_eq!(output.thumbnail_mime_type.as_str(), "image/webp");
        let decoded = image::load_from_memory(&output.thumbnail_bytes).unwrap();
        assert_eq!(decoded.width(), 128);
        assert_eq!(decoded.height(), 64);
    }
}
