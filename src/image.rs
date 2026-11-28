use crate::Style;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use gpui::{AlignItems, AnyElement, App, AvailableSpace, Bounds, Element, ElementId, GlobalElementId, Image, ImageFormat, ImageSource,
    InspectorElementId, IntoElement, LayoutId, Length, ParentElement as _, Pixels, Styled as _, StyledImage as _, Window, div, img, px, relative, size};
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

/// Renders an image at its natural size, scaling height proportionally when clamped to the container's width.
pub(crate) struct FittedImage { source: ImageSource, alt: String }

pub(crate) fn fitted_image(url: &str, alt: String, cx: &App) -> FittedImage {
    FittedImage { source: image_source(url, cx), alt }
}

impl IntoElement for FittedImage {
    type Element = Self;
    fn into_element(self) -> Self { self }
}

impl Element for FittedImage {
    type RequestLayoutState = Option<AnyElement>;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> { None }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> { None }

    fn request_layout(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, window: &mut Window, cx: &mut App) -> (LayoutId, Option<AnyElement>) {
        let nat = img(self.source.clone()).into_any_element().layout_as_root(size(AvailableSpace::MaxContent, AvailableSpace::MaxContent), window, cx);
        if nat.width <= px(0.) || nat.height <= px(0.) {
            let alt = self.alt.clone();
            let mut child = img(self.source.clone()).max_w_full().with_fallback(move || div().child(format!("[{alt}]")).into_any_element()).into_any_element();
            return (child.request_layout(window, cx), Some(child));
        }
        let style = gpui::Style { align_self: Some(AlignItems::FlexStart), max_size: size(relative(1.).into(), Length::Auto), ..Default::default() };
        let id = window.request_measured_layout(style, move |known, avail, _, _| {
            let max_w = known.width.or(match avail.width { AvailableSpace::Definite(w) => Some(w.max(px(0.))), AvailableSpace::MinContent => Some(px(0.)), AvailableSpace::MaxContent => None });
            let (w, h) = match max_w { Some(m) if nat.width > m => (m, nat.height * (m / nat.width)), _ => (nat.width, nat.height) };
            size(w, known.height.unwrap_or(h))
        });
        (id, None)
    }

    fn prepaint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, bounds: Bounds<Pixels>, child: &mut Option<AnyElement>, window: &mut Window, cx: &mut App) {
        if let Some(el) = child { el.prepaint(window, cx); return }
        let mut el = img(self.source.clone()).w(bounds.size.width).h(bounds.size.height).into_any_element();
        el.prepaint_as_root(bounds.origin, bounds.size.map(AvailableSpace::Definite), window, cx);
        *child = Some(el);
    }

    fn paint(&mut self, _: Option<&GlobalElementId>, _: Option<&InspectorElementId>, _: Bounds<Pixels>, child: &mut Option<AnyElement>, _: &mut (), window: &mut Window, cx: &mut App) {
        if let Some(el) = child { el.paint(window, cx) }
    }
}
