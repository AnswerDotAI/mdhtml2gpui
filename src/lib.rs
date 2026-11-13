//! Render mdhtml documents as GPUI elements.

mod block;
mod image;
mod inline;
mod source;

use gpui::{AnyElement, App, Bounds, Global, HighlightStyle, Hsla, ImageSource, IntoElement, ParentElement as _, Pixels, SharedString, div, hsla, px};
use mdhtml::{Block, Document, Inline, Options, TableEntry};
use std::{ops::Range, rc::Rc};

pub use block::{md2gpui, md2gpui_mapped, mdhtml2gpui};
pub use image::image_source;
pub use source::SourceMap;

/// Colours, fonts and hooks for rendering, set by the app as a GPUI global. Text colour and the body font come from the parent element.
#[derive(Clone)]
pub struct Style {
    /// Background of code and code blocks.
    pub code_bg: Hsla,
    /// Rules, table lines and block quote bars.
    pub border: Hsla,
    /// Secondary text: quotes, captions, footnotes and the source of undrawable content.
    pub muted: Hsla,
    pub link: Hsla,
    /// A disclosure title under the pointer.
    pub hover: Hsla,
    /// Background of highlighted text: `==x==` and `<mark>`.
    pub mark: Hsla,
    pub mono_font: SharedString,
    pub mono_size: Pixels,
    /// The disclosure marker for an open (`true`) or closed panel.
    pub chevron: fn(bool, &App) -> AnyElement,
    /// Image source for a URL that isn't a data URL, such as one that needs the app's credentials.
    pub load: Option<Loader>,
    /// Syntax colours for code blocks. Without it they're plain.
    pub highlight: Option<Highlighter>,
}

impl Global for Style {}

/// The app's image source for a URL.
pub type Loader = Rc<dyn Fn(&str) -> ImageSource>;

/// The app's syntax colours for a code block's text, given its language if the block names one: byte ranges of the text and their styles.
pub type Highlighter = Rc<dyn Fn(Option<&str>, &str, &App) -> Vec<(Range<usize>, HighlightStyle)>>;

fn chevron(open: bool, _: &App) -> AnyElement { div().child(if open { "\u{25be}" } else { "\u{25b8}" }).into_any_element() }

/// The platform's usual monospace family. GPUI doesn't resolve the generic name `monospace`, so apps with other fonts set [`Style::mono_font`].
const MONO: &str = if cfg!(target_os = "macos") { "Menlo" } else if cfg!(target_os = "windows") { "Consolas" } else { "DejaVu Sans Mono" };

impl Default for Style {
    fn default() -> Self {
        Self { code_bg: hsla(0.61, 0.2, 0.94, 1.), border: hsla(0.61, 0.13, 0.85, 1.), muted: hsla(0.61, 0.09, 0.42, 1.), link: hsla(0.6, 0.8, 0.45, 1.),
            hover: hsla(0.6, 0.8, 0.35, 1.), mark: hsla(0.14, 0.9, 0.8, 1.), mono_font: MONO.into(), mono_size: px(13.), chevron, load: None, highlight: None }
    }
}

impl Style {
    /// The default style for a dark background.
    pub fn dark() -> Self {
        Self { code_bg: hsla(0.61, 0.15, 0.17, 1.), border: hsla(0.61, 0.1, 0.28, 1.), muted: hsla(0.61, 0.08, 0.62, 1.), link: hsla(0.6, 0.85, 0.7, 1.),
            hover: hsla(0.6, 0.85, 0.8, 1.), mark: hsla(0.14, 0.6, 0.3, 1.), ..Self::default() }
    }
}

fn style(cx: &App) -> Style { cx.try_global::<Style>().cloned().unwrap_or_default() }

/// The node kinds in `doc` that render as their source because GPUI can't draw them, in document order.
pub fn fallbacks(doc: &Document) -> Vec<&'static str> {
    fn inlines(items: &[Inline], out: &mut Vec<&'static str>) {
        for item in items {
            if let Some((kind, _)) = inline::source(item) { out.push(kind) }
            if let Some(children) = inline::children(item) { inlines(children, out) }
        }
    }
    fn blocks(items: &[Block], out: &mut Vec<&'static str>) {
        for b in items {
            if let Some((kind, _)) = block::source(b) { out.push(kind) }
            match b {
                Block::Paragraph { children, .. } | Block::Heading { children, .. } => inlines(children, out),
                Block::Panel(p) => { if let Some(t) = &p.title { inlines(&t.children, out) } blocks(&p.body, out) }
                Block::Element { children, .. } | Block::BlockQuote { children, .. } | Block::Div { children, .. } => blocks(children, out),
                Block::List { items, .. } => for i in items { blocks(&i.blocks, out) },
                Block::DefinitionList { items, .. } => for i in items {
                    for t in &i.terms { inlines(&t.inlines, out) }
                    for d in &i.definitions { blocks(d, out) }
                },
                Block::Bibliography { entries, .. } => for e in entries { inlines(&e.author, out); blocks(&e.body, out) },
                Block::Table { caption, head, rows, foot, .. } => {
                    for entry in head.iter().chain(rows).chain(foot) {
                        match entry { TableEntry::Row(r) => for c in &r.cells { blocks(&c.content, out) }, TableEntry::Operation(_) => out.push("operation") }
                    }
                    inlines(caption, out)
                }
                Block::Figure { caption, body, .. } => { blocks(body, out); inlines(caption, out) }
                _ => {}
            }
        }
    }
    let mut out = vec![];
    blocks(&doc.blocks, &mut out);
    for note in &doc.footnotes { blocks(&note.blocks, &mut out) }
    out
}

/// The byte offset where each displayed top-level block of `src` starts. Link references and attribute definitions aren't displayed.
pub fn block_starts(src: &str) -> Vec<usize> {
    let lines: Vec<usize> = std::iter::once(0).chain(src.match_indices('\n').map(|(i, _)| i + 1)).collect();
    let spans = mdhtml::block_spans(src, &Options::default());
    spans.iter().filter(|s| !matches!(s.kind, "link_ref" | "attr_def")).map(|s| lines.get(s.start).copied().unwrap_or(src.len())).collect()
}

/// The index of the bounds nearest to height `y`.
pub fn nearest(bounds: &[Bounds<Pixels>], y: Pixels) -> usize {
    let gap = |b: &Bounds<Pixels>| f32::from(if y < b.top() { b.top() - y } else if y > b.bottom() { y - b.bottom() } else { px(0.) });
    (0..bounds.len()).min_by(|&i, &j| gap(&bounds[i]).total_cmp(&gap(&bounds[j]))).unwrap_or(0)
}
