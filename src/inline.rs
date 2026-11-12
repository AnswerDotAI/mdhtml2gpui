use crate::block::At;
use crate::{Style, image_source, style};
use gpui::{App, Div, FontStyle, FontWeight, HighlightStyle, InteractiveText, IntoElement, ParentElement as _, SharedString, StrikethroughStyle, Styled as _,
    StyledImage as _, StyledText, UnderlineStyle, div, img, px};
use mdhtml::Inline;
use std::borrow::Cow;
use std::ops::Range;

/// Text with its style, font and link runs.
#[derive(Default)]
struct Runs { text: String, styles: Vec<(Range<usize>, HighlightStyle)>, fonts: Vec<(Range<usize>, SharedString)>, links: Vec<(Range<usize>, String)> }

/// A stretch of paragraph content. GPUI can't place an image inside a line, so an image splits a paragraph's text.
enum Seg { Text(Runs), Image { url: String, alt: String } }

struct Builder<'a> { st: &'a Style, segs: Vec<Seg>, runs: Runs }

/// The kind name and source text of an inline that renders as its source.
pub(crate) fn source(item: &Inline) -> Option<(&'static str, Cow<'_, str>)> {
    Some(match item {
        Inline::Html(s) => ("html", s.into()), Inline::Raw { text, .. } => ("raw", text.into()), Inline::Operation(op) => ("operation", (&op.syntax).into()),
        Inline::Math { tex, display, .. } => ("math", if *display { format!("\\[{tex}\\]") } else { format!("\\({tex}\\)") }.into()), _ => return None
    })
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

    fn inlines(&mut self, items: &[Inline], style: HighlightStyle) {
        let st = self.st;
        let underline = Some(UnderlineStyle { thickness: px(1.), ..Default::default() });
        for item in items {
            if let Some((_, text)) = source(item) { self.mono(&text, HighlightStyle { color: Some(st.muted), ..style }); continue }
            match item {
                Inline::Text(s) => { self.push(s, style); }
                Inline::SoftBreak => { self.push(" ", style); }
                Inline::HardBreak => { self.push("\n", style); }
                Inline::Code { text, .. } => self.mono(text, HighlightStyle { background_color: Some(st.code_bg), ..style }),
                Inline::Emph { children, .. } => self.inlines(children, HighlightStyle { font_style: Some(FontStyle::Italic), ..style }),
                Inline::Strong { children, .. } => self.inlines(children, HighlightStyle { font_weight: Some(FontWeight::BOLD), ..style }),
                Inline::Strike { children, .. } => self.inlines(children, HighlightStyle { strikethrough: Some(StrikethroughStyle { thickness: px(1.), color: None }), ..style }),
                Inline::Highlight { children, .. } => self.inlines(children, HighlightStyle { background_color: Some(st.mark), ..style }),
                Inline::Link { children, url, .. } => {
                    let start = self.runs.text.len();
                    self.inlines(children, HighlightStyle { color: Some(st.link), underline, ..style });
                    self.runs.links.push((start..self.runs.text.len(), url.clone()));
                }
                Inline::FootnoteRef { label } => { self.push(&format!("[{label}]"), HighlightStyle { color: Some(st.link), ..style }); }
                Inline::Reference { .. } => { self.push(&mdhtml::plain(std::slice::from_ref(item)), HighlightStyle { color: Some(st.link), ..style }); }
                Inline::Image { url, alt, .. } => {
                    let runs = std::mem::take(&mut self.runs);
                    if !runs.text.is_empty() { self.segs.push(Seg::Text(runs)) }
                    self.segs.push(Seg::Image { url: url.clone(), alt: alt.clone() });
                }
                Inline::Element { tag, children, .. } => {
                    let style = match tag.as_str() {
                        "em" | "i" => HighlightStyle { font_style: Some(FontStyle::Italic), ..style },
                        "strong" | "b" => HighlightStyle { font_weight: Some(FontWeight::BOLD), ..style },
                        "u" | "ins" => HighlightStyle { underline, ..style },
                        "mark" => HighlightStyle { background_color: Some(st.mark), ..style },
                        "kbd" | "code" | "samp" => { self.mono(&mdhtml::plain(children), HighlightStyle { background_color: Some(st.code_bg), ..style }); continue }
                        _ => style
                    };
                    self.inlines(children, style)
                }
                Inline::Superscript { children, .. } | Inline::Subscript { children, .. } | Inline::Span { children, .. } => self.inlines(children, style),
                Inline::Html(_) | Inline::Raw { .. } | Inline::Operation(_) | Inline::Math { .. } => unreachable!("rendered by `source`"),
            }
        }
    }
}

fn build(items: &[Inline], cx: &App) -> Vec<Seg> {
    let st = style(cx);
    let mut b = Builder { st: &st, segs: vec![], runs: Runs::default() };
    b.inlines(items, HighlightStyle::default());
    let runs = std::mem::take(&mut b.runs);
    if !runs.text.is_empty() || b.segs.is_empty() { b.segs.push(Seg::Text(runs)) }
    b.segs
}
/// A block of inline content, with clickable links and with images between its lines of text. `at`'s key names its elements, and each run of
/// text records into `at`'s source map.
pub(crate) fn paragraph(at: &At, items: &[Inline], cx: &App) -> Div {
    div().flex().flex_col().gap_1().children(build(items, cx).into_iter().enumerate().map(|(i, seg)| match seg {
        Seg::Text(Runs { text, styles, fonts, links }) => {
            let styled = StyledText::new(text.clone()).with_highlights(styles).with_font_family_overrides(fonts);
            at.record(&text, &styled);
            if links.is_empty() { return styled.into_any_element() }
            let (ranges, urls): (Vec<_>, Vec<_>) = links.into_iter().unzip();
            InteractiveText::new(SharedString::from(format!("{}:{i}", at.key())), styled).on_click(ranges, move |ix, _, cx| cx.open_url(&urls[ix])).into_any_element()
        }
        Seg::Image { url, alt } => img(image_source(&url, cx)).max_w_full().with_fallback(move || div().child(format!("[{alt}]")).into_any_element()).into_any_element()
    }))
}
