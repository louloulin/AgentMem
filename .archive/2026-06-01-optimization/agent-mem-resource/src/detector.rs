//! Media type detection using magic bytes and file extensions

use crate::models::MediaType;
use crate::Result;
use std::path::Path;

/// Media type detector
///
/// Detects media types using:
/// 1. Magic bytes (file header signatures)
/// 2. File extensions from URI
/// 3. Content inspection
pub struct MediaTypeDetector;

impl MediaTypeDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self
    }

    /// Detect media type from URI path and optional data
    ///
    /// # Arguments
    /// * `uri` - Resource URI
    /// * `data` - Optional raw data for magic byte detection
    ///
    /// # Returns
    /// Detected media type
    pub fn detect(&self, uri: &str, data: Option<&[u8]>) -> Result<MediaType> {
        // Try magic bytes first if data is available
        if let Some(bytes) = data {
            if let Some(mt) = self.detect_from_magic_bytes(bytes) {
                return Ok(mt);
            }
        }

        // Fall back to extension detection
        let extension = self.extract_extension(uri);
        Ok(self.detect_from_extension(&extension))
    }

    /// Detect media type from magic bytes (file signatures)
    pub fn detect_from_magic_bytes(&self, data: &[u8]) -> Option<MediaType> {
        if data.len() < 4 {
            return None;
        }

        // PNG: 89 50 4E 47
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Some(MediaType::ImagePng);
        }

        // JPEG: FF D8 FF
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(MediaType::ImageJpeg);
        }

        // GIF: 47 49 46 38
        if data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
            return Some(MediaType::ImageGif);
        }

        // WebP: 52 49 46 46 ... 57 45 42 50
        if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
            return Some(MediaType::ImageWebp);
        }

        // PDF: 25 50 44 46 (%PDF)
        if data.starts_with(&[0x25, 0x50, 0x44, 0x46]) {
            return Some(MediaType::ApplicationPdf);
        }

        // ZIP: 50 4B 03 04 (local file header) or 50 4B 05 06 (empty archive)
        if data.starts_with(&[0x50, 0x4B, 0x03, 0x04])
            || data.starts_with(&[0x50, 0x4B, 0x05, 0x06])
        {
            return Some(MediaType::ApplicationZip);
        }

        // Check for text content (UTF-8)
        if self.is_text_content(data) {
            return Some(MediaType::TextPlain);
        }

        None
    }

    /// Detect media type from file extension
    pub fn detect_from_extension(&self, extension: &str) -> MediaType {
        match extension.to_lowercase().as_str() {
            // Text types
            "txt" => MediaType::TextPlain,
            "md" | "markdown" => MediaType::TextMarkdown,
            "html" | "htm" => MediaType::TextHtml,
            "csv" => MediaType::TextCsv,

            // Image types
            "png" => MediaType::ImagePng,
            "jpg" | "jpeg" => MediaType::ImageJpeg,
            "gif" => MediaType::ImageGif,
            "webp" => MediaType::ImageWebp,
            "svg" => MediaType::ImageSvg,

            // Audio types
            "mp3" | "mpeg" => MediaType::AudioMpeg,
            "wav" => MediaType::AudioWav,
            "ogg" => MediaType::AudioOgg,

            // Video types
            "mp4" => MediaType::VideoMp4,
            "webm" => MediaType::VideoWebm,

            // Application types
            "pdf" => MediaType::ApplicationPdf,
            "json" => MediaType::ApplicationJson,
            "xml" => MediaType::ApplicationXml,
            "zip" => MediaType::ApplicationZip,

            // Unknown
            ext => MediaType::Unknown(format!("application/{}", ext)),
        }
    }

    /// Extract file extension from URI
    fn extract_extension(&self, uri: &str) -> String {
        // Parse URI and get path
        let path: String = if uri.starts_with("http://") || uri.starts_with("https://") {
            // For HTTP URIs, get the path component
            uri.split('/')
                .last()
                .unwrap_or("")
                .split('?')
                .next()
                .unwrap_or("")
                .to_string()
        } else if uri.starts_with("file://") {
            // For file URIs, remove the protocol prefix
            uri.replacen("file://", "", 1)
        } else if uri.starts_with("conv://") || uri.starts_with("doc://") {
            // For custom protocols, try to extract extension
            uri.split('/').last().unwrap_or("").to_string()
        } else {
            // Assume it's already a path
            uri.to_string()
        };

        // Get extension from path
        Path::new(&path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string()
    }

    /// Check if data is text content (UTF-8 valid and mostly printable)
    fn is_text_content(&self, data: &[u8]) -> bool {
        // Try to parse as UTF-8
        match std::str::from_utf8(data) {
            Ok(text) => {
                // Check if mostly printable (at least 90% printable ASCII or UTF-8)
                let printable = text
                    .chars()
                    .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
                    .count();
                let total = text.chars().count();
                total > 0 && (printable as f64 / total as f64) >= 0.9
            }
            Err(_) => false,
        }
    }
}

impl Default for MediaTypeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_from_extension() {
        let detector = MediaTypeDetector::new();

        assert_eq!(detector.detect_from_extension("txt"), MediaType::TextPlain);
        assert_eq!(detector.detect_from_extension("png"), MediaType::ImagePng);
        assert_eq!(
            detector.detect_from_extension("pdf"),
            MediaType::ApplicationPdf
        );
    }

    #[test]
    fn test_detect_from_magic_bytes() {
        let detector = MediaTypeDetector::new();

        // PNG magic bytes
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(
            detector.detect_from_magic_bytes(&png_data),
            Some(MediaType::ImagePng)
        );

        // JPEG magic bytes
        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(
            detector.detect_from_magic_bytes(&jpeg_data),
            Some(MediaType::ImageJpeg)
        );

        // PDF magic bytes
        let pdf_data = b"%PDF-1.4".to_vec();
        assert_eq!(
            detector.detect_from_magic_bytes(&pdf_data),
            Some(MediaType::ApplicationPdf)
        );
    }

    #[test]
    fn test_extract_extension() {
        let detector = MediaTypeDetector::new();

        assert_eq!(
            detector.extract_extension("file:///path/to/document.pdf"),
            "pdf"
        );
        assert_eq!(
            detector.extract_extension("https://example.com/image.png"),
            "png"
        );
        assert_eq!(detector.extract_extension("conv://chat-123"), "");
    }

    #[test]
    fn test_is_text_content() {
        let detector = MediaTypeDetector::new();

        let text = b"Hello, world!";
        assert!(detector.is_text_content(text));

        let binary = vec![0x00, 0x01, 0x02, 0x03];
        assert!(!detector.is_text_content(&binary));
    }

    #[test]
    fn test_detect_combined() {
        let detector = MediaTypeDetector::new();

        // With both URI and data, magic bytes should win
        let uri = "file:///test.txt";
        let data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
        let media_type = detector.detect(uri, Some(&data)).unwrap();

        assert_eq!(media_type, MediaType::ImagePng);

        // With only URI, use extension
        let media_type = detector.detect(uri, None).unwrap();
        assert_eq!(media_type, MediaType::TextPlain);
    }
}
