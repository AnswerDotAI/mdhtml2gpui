use crate::{Style, image_source, style};
use gpui::{App, Div, FontStyle, FontWeight, HighlightStyle, InteractiveText, IntoElement, ParentElement as _, SharedString, StrikethroughStyle, Styled as _,
    StyledImage as _, StyledText, UnderlineStyle, div, img, px};
use mdhtml::Inline;
use std::ops::Range;

/// Text with its style, font and link runs.
#[derive(Default)]
struct Runs { text: String, styles: Vec<(Range<usize>, HighlightStyle)>, fonts: Vec<(Range<usize>, SharedString)>, links: Vec<(Range<usize>, String)> }

/// A stretch of paragraph content. GPUI can't place an image inside a line, so an image splits a paragraph's text.
enum Seg { Text(Runs), Image { url: String, alt: String } }

struct Builder<'a> { st: &'a Style, images: bool, segs: Vec<Seg>, runs: Runs }

/// The kind name of an inline that renders as its source.
pub(crate) fn source_kind(item: &Inline) -> Option<&'static str> {
    match item { Inline::Html(_) => Some("html"), Inline::Raw { .. } => Some("raw"), Inline::Operation(_) => Some("operation"), Inline::Math { .. } => Some("math"), _ => None }
}

pub(crate) fn children(item: &Inline) -> Option<&[Inline]> {
    match item {
        Inline::Element { children, .. } | Inline::Emph { children, .. } | Inline::Strong { children, .. } | Inline::Strike { children, .. }
        | Inline::Superscript { children, .. } | Inline::Subscript { children, .. } | Inline::Highlight { children, .. } | Inline::Link { children, .. }
        | Inline::Span { children, .. } => Some(children),
        _ => None
    }
}

impl Builder<'_> {
    fn push(&mut self, s: &str, style: HighlightStyle) -> Range<usize> {
        let range = self.runs.text.len()..self.runs.text.len() + s.len();
        self.runs.text.push_str(s);
        if style != HighlightStyle::default() && !s.is_empty() { self.runs.styles.push((range.clone(), style)) }
        range
    }

    fn mono(&mut self, s: &str, style: HighlightStyle) {
        let range = self.push(s, style);
        self.runs.fonts.push((range, self.st.mono_font.clone()));
    }

    fn source(&mut self, s: &str, style: HighlightStyle) { self.mono(s, HighlightStyle { color: Some(self.st.muted), ..style }) }

    fn inlines(&mut self, items: &[Inline], style: HighlightStyle) {
        let st = self.st;
        let mark = if st.dark { gpui::hsla(0.14, 0.6, 0.3, 1.) } else { gpui::hsla(0.14, 0.9, 0.8, 1.) };
        let underline = Some(UnderlineStyle { thickness: px(1.), ..Default::default() });
        for item in items {
            match item {
                Inline::Text(s) => { self.push(s, style); }
                Inline::SoftBreak => { self.push(" ", style); }
                Inline::HardBreak => { self.push("\n", style); }
                Inline::Code { text, .. } => self.mono(text, HighlightStyle { background_color: Some(st.code_bg), ..style }),
                Inline::Emph { children, .. } => self.inlines(children, HighlightStyle { font_style: Some(FontStyle::Italic), ..style }),
                Inline::Strong { children, .. } => self.inlines(children, HighlightStyle { font_weight: Some(FontWeight::BOLD), ..style }),
                Inline::Strike { children, .. } => self.inlines(children, HighlightStyle { strikethrough: Some(StrikethroughStyle { thickness: px(1.), color: None }), ..style }),
                Inline::Highlight { children, .. } => self.inlines(children, HighlightStyle { background_color: Some(mark), ..style }),
                Inline::Link { children, url, .. } => {
                    let start = self.runs.text.len();
                    self.inlines(children, HighlightStyle { color: Some(st.link), underline, ..style });
                    self.runs.links.push((start..self.runs.text.len(), url.clone()));
                }
                Inline::FootnoteRef { label } => { self.push(&format!("[{label}]"), HighlightStyle { color: Some(st.link), ..style }); }
                Inline::Reference { .. } => { self.push(&mdhtml::plain(std::slice::from_ref(item)), HighlightStyle { color: Some(st.link), ..style }); }
                Inline::Image { url, alt, .. } if self.images => {
                    let runs = std::mem::take(&mut self.runs);
                    if !runs.text.is_empty() { self.segs.push(Seg::Text(runs)) }
                    self.segs.push(Seg::Image { url: url.clone(), alt: alt.clone() });
                }
                Inline::Image { alt, .. } => { self.push(alt, style); }
                Inline::Element { tag, children, .. } => {
                    let style = match tag.as_str() {
                        "em" | "i" => HighlightStyle { font_style: Some(FontStyle::Italic), ..style },
                        "strong" | "b" => HighlightStyle { font_weight: Some(FontWeight::BOLD), ..style },
                        "u" | "ins" => HighlightStyle { underline, ..style },
                        "mark" => HighlightStyle { background_color: Some(mark), ..style },
                        "kbd" | "code" | "samp" => { self.mono(&mdhtml::plain(children), HighlightStyle { background_color: Some(st.code_bg), ..style }); continue }
                        _ => style
                    };
                    self.inlines(children, style)
                }
                Inline::Superscript { children, .. } | Inline::Subscript { children, .. } | Inline::Span { children, .. } => self.inlines(children, style),
                Inline::Html(s) => self.source(s, style),
                Inline::Raw { text, .. } => self.source(text, style),
                Inline::Operation(op) => self.source(&op.syntax, style),
                Inline::Math { tex, display, .. } => self.source(&if *display { format!("\\[{tex}\\]") } else { format!("\\({tex}\\)") }, style),
            }
        }
    }
}

fn build(items: &[Inline], images: bool, cx: &App) -> Vec<Seg> {
    let st = style(cx);
    let mut b = Builder { st: &st, images, segs: vec![], runs: Runs::default() };
    b.inlines(items, HighlightStyle::default());
    let runs = std::mem::take(&mut b.runs);
    if !runs.text.is_empty() || b.segs.is_empty() { b.segs.push(Seg::Text(runs)) }
    b.segs
}

fn styled(runs: Runs) -> (StyledText, Vec<(Range<usize>, String)>) {
    (StyledText::new(runs.text).with_highlights(runs.styles).with_font_family_overrides(runs.fonts), runs.links)
}

/// Inline content as one styled text. Images show their alt text, and links are styled but not clickable.
pub fn styled_text(items: &[Inline], cx: &App) -> StyledText {
    let runs = build(items, false, cx).into_iter().find_map(|s| match s { Seg::Text(r) => Some(r), Seg::Image { .. } => None }).unwrap_or_default();
    styled(runs).0
}

/// A block of inline content, with clickable links and with images between its lines of text. `key` names the paragraph's elements.
pub(crate) fn paragraph(key: &str, items: &[Inline], cx: &App) -> Div {
    div().flex().flex_col().gap_1().children(build(items, true, cx).into_iter().enumerate().map(|(i, seg)| match seg {
        Seg::Text(runs) => {
            let (text, links) = styled(runs);
            if links.is_empty() { return text.into_any_element() }
            let (ranges, urls): (Vec<_>, Vec<_>) = links.into_iter().unzip();
            InteractiveText::new(SharedString::from(format!("{key}:{i}")), text).on_click(ranges, move |ix, _, cx| cx.open_url(&urls[ix])).into_any_element()
        }
        Seg::Image { url, alt } => img(image_source(&url, cx)).max_w_full().with_fallback(move || div().child(format!("[{alt}]")).into_any_element()).into_any_element()
    }))
}
