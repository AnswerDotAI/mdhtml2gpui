use crate::Style;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use gpui::{App, Image, ImageFormat, ImageSource};
use std::path::PathBuf;
use std::sync::Arc;

/// The image a data URL holds, for the formats GPUI decodes.
fn data_image(url: &str) -> Option<Arc<Image>> {
    let (head, body) = url.strip_prefix("data:")?.split_once(',')?;
    let mut parts = head.split(';');
    let format = match parts.next()? {
        "image/png" => ImageFormat::Png, "image/jpeg" | "image/jpg" => ImageFormat::Jpeg, "image/gif" => ImageFormat::Gif, "image/webp" => ImageFormat::Webp,
        "image/svg+xml" => ImageFormat::Svg, "image/bmp" => ImageFormat::Bmp, "image/tiff" => ImageFormat::Tiff, _ => return None
    };
    let bytes = if parts.any(|p| p == "base64") { STANDARD.decode(body).ok()? } else { percent_encoding::percent_decode_str(body).collect() };
    Some(Arc::new(Image::from_bytes(format, bytes)))
}

/// Where an image's bytes come from: a data URL decodes here, other URLs go to the app's [`Style::load`] when set, and otherwise to GPUI as a URI or a
/// file path.
pub fn image_source(url: &str, cx: &App) -> ImageSource {
    if let Some(image) = data_image(url) { return ImageSource::Image(image) }
    if let Some(load) = cx.try_global::<Style>().and_then(|s| s.load.clone()) { return load(url) }
    if url.contains("://") { url.into() } else { PathBuf::from(url).into() }
}
