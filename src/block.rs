use crate::inline::paragraph;
use crate::{SourceMap, Style, style};
use gpui::{AnyElement, App, Div, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString, StatefulInteractiveElement as _,
    Styled as _, StyledText, Window, div, px, relative};
use gpui::prelude::FluentBuilder as _;
use mdhtml::{Align, Block, Disclosure, Document, Options, Panel, TableEntry, TableRow};
use std::rc::Rc;

/// The kind name of a block that renders as its source.
pub(crate) fn source_kind(b: &Block) -> Option<&'static str> {
    match b {
        Block::Html { .. } => Some("html"), Block::Raw { .. } => Some("raw"), Block::Script { .. } => Some("script"), Block::Operation(_) => Some("operation"),
        Block::Math { .. } => Some("math"), _ => None
    }
}

/// Where a block renders: the document's `doc` id, which prefixes element ids and test selectors, `path`, the block's position in the document,
/// and the source map its text records into, if any.
#[derive(Clone)]
pub(crate) struct At { doc: SharedString, path: String, map: Option<Rc<SourceMap>> }

impl At {
    fn child(&self, i: usize) -> At { At { doc: self.doc.clone(), path: format!("{}.{i}", self.path), map: self.map.clone() } }
    pub(crate) fn key(&self) -> SharedString { format!("{}/{}", self.doc, self.path).into() }
    fn tag(&self, el: Div, kind: &str) -> Div { el.debug_selector(|| format!("{}/{kind}", self.doc)) }

    /// Records `text`, laid out by `styled`, in the source map under its top-level block. Footnotes keep block-level placement.
    pub(crate) fn record(&self, text: &str, styled: &StyledText) {
        let block = self.path.strip_prefix('.').and_then(|p| p.split('.').next()?.parse().ok());
        if let (Some(map), Some(block)) = (&self.map, block) { map.record(block, text.into(), styled.layout().clone()) }
    }
}

fn mono(at: &At, text: &str, st: &Style) -> Div {
    let text = text.trim_end_matches('\n');
    let styled = StyledText::new(text.to_string());
    at.record(text, &styled);
    div().font_family(st.mono_font.clone()).text_size(st.mono_size).line_height(relative(1.3)).child(styled)
}

fn source(at: &At, kind: &str, text: &str, st: &Style) -> AnyElement { at.tag(mono(at, text, st).text_color(st.muted), kind).into_any_element() }

fn blocks(at: &At, items: &[Block], cx: &App) -> Div {
    div().flex().flex_col().gap_2().children(items.iter().enumerate().map(|(i, b)| block(&at.child(i), b, cx)))
}

fn caption(number: Option<&str>, prefix: &str, text: Div, st: &Style) -> Div {
    div().text_sm().text_color(st.muted).flex().gap_1().when_some(number, |d, n| d.child(format!("{prefix} {n}:"))).child(text)
}

fn table(at: &At, aligns: &[Align], entries: [&[TableEntry]; 3], st: &Style, cx: &App) -> Div {
    let cols = mdhtml::ast::table_columns(aligns, entries[0], entries[1], entries[2]);
    let row = |r: &TableRow, i: usize, head: bool| -> Div {
        let cells = r.cells.iter().enumerate().map(|(j, c)| {
            let cell = div().flex_1().min_w_0().px_2().py_1().child(blocks(&at.child(i).child(j), &c.content, cx));
            match c.align { Align::Center => cell.text_center(), Align::Right => cell.text_right(), _ => cell }
        });
        let pad = cols.saturating_sub(r.cells.len());
        div().flex().border_b_1().border_color(st.border).when(head, |d| d.font_weight(FontWeight::BOLD)).children(cells)
            .children((0..pad).map(|_| div().flex_1().min_w_0()))
    };
    let (mut i, mut rows) = (0, vec![]);
    for (part, head) in [(entries[0], true), (entries[1], false), (entries[2], false)] {
        for entry in part { rows.push(match entry { TableEntry::Operation(op) => mono(at, &op.syntax, st).text_color(st.muted), TableEntry::Row(r) => row(r, i, head) }); i += 1 }
    }
    div().flex().flex_col().border_t_1().border_color(st.border).children(rows)
}

fn block(at: &At, b: &Block, cx: &App) -> AnyElement {
    let st = style(cx);
    let el = match b {
        Block::Heading { level, children, number, .. } => {
            let heading = div().font_weight(FontWeight::BOLD).flex().gap_2().when_some(number.as_ref(), |d, n| d.child(n.display.clone()))
                .child(paragraph(at, &at.key(), children, cx));
            at.tag(match level { 1 => heading.text_2xl(), 2 => heading.text_xl(), 3 => heading.text_lg(), _ => heading }, "heading")
        }
        Block::Paragraph { children, .. } => at.tag(paragraph(at, &at.key(), children, cx), "paragraph"),
        Block::CodeBlock { text, .. } => at.tag(mono(at, text, &st).p_2().rounded_md().bg(st.code_bg), "code"),
        Block::List { ordered, start, tight, items, .. } => at.tag(div().flex().flex_col().map(|d| if *tight { d.gap_1() } else { d.gap_2() })
            .children(items.iter().enumerate().map(|(i, item)| {
                let marker = match item.checked {
                    Some(true) => "\u{2611}".into(), Some(false) => "\u{2610}".into(), None if *ordered => format!("{}.", start + i), None => "\u{2022}".into()
                };
                div().flex().items_start().gap_2().child(marker).child(blocks(&at.child(i), &item.blocks, cx).flex_1().min_w_0())
            })), "list"),
        Block::BlockQuote { children, .. } => at.tag(blocks(at, children, cx).pl_3().border_l_2().border_color(st.border).text_color(st.muted), "quote"),
        Block::ThematicBreak { .. } => at.tag(div().h(px(1.)).bg(st.border), "break"),
        Block::Table { aligns, head, rows, foot, caption: cap, number, .. } => {
            let t = table(at, aligns, [head, rows, foot], &st, cx);
            at.tag(div().flex().flex_col().gap_1().child(t).when(!cap.is_empty(), |d| {
                d.child(caption(number.as_ref().map(|n| n.display.as_str()), "Table", paragraph(at, &at.key(), cap, cx), &st))
            }), "table")
        }
        Block::DefinitionList { items, .. } => at.tag(div().flex().flex_col().gap_2().children(items.iter().enumerate().map(|(i, item)| {
            let at = at.child(i);
            div().flex().flex_col().gap_1()
                .children(item.terms.iter().map(|t| paragraph(&at, &at.key(), &t.inlines, cx).font_weight(FontWeight::BOLD)))
                .children(item.definitions.iter().enumerate().map(|(j, d)| blocks(&at.child(j), d, cx).pl_4()))
        })), "definitions"),
        Block::Bibliography { entries, .. } => at.tag(div().flex().flex_col().gap_2().children(entries.iter().enumerate().map(|(i, e)| {
            let at = at.child(i);
            let head = div().flex().gap_2().font_weight(FontWeight::BOLD).child(paragraph(&at, &at.key(), &e.author, cx)).when_some(e.year.as_ref(), |d, y| d.child(format!("({y})")));
            div().flex().flex_col().gap_1().child(head).child(blocks(&at, &e.body, cx).pl_4())
        })), "bibliography"),
        Block::Figure { caption: cap, body, number, .. } => at.tag(blocks(at, body, cx).when(!cap.is_empty(), |d| {
            d.child(caption(number.as_ref().map(|n| n.display.as_str()), "Figure", paragraph(at, &at.key(), cap, cx), &st))
        }), "figure"),
        Block::Panel(panel) => return Details { at: at.clone(), panel: panel.clone() }.into_any_element(),
        Block::Element { children, .. } => at.tag(blocks(at, children, cx), "element"),
        Block::Div { children, .. } => at.tag(blocks(at, children, cx), "div"),
        Block::Math { tex, .. } => return source(at, "math", &format!("\\[\n{tex}\n\\]"), &st),
        Block::Html { raw } => return source(at, "html", raw, &st),
        Block::Raw { text, .. } | Block::Script { text, .. } => return source(at, source_kind(b).unwrap_or("raw"), text, &st),
        Block::Operation(op) => return source(at, "operation", &op.syntax, &st),
    };
    el.into_any_element()
}

/// An mdhtml panel: a callout, or a disclosure whose title toggles its body. Its open state is keyed by the document id and the panel's position.
#[derive(IntoElement)]
struct Details { at: At, panel: Panel }

impl RenderOnce for Details {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Details { at, panel } = self;
        let st = style(cx);
        let key = at.key();
        let open = window.use_keyed_state(key.clone(), cx, |_, _| panel.disclosure == Disclosure::Open);
        let toggles = panel.disclosure != Disclosure::Fixed;
        let shown = !toggles || *open.read(cx);
        let title = match &panel.title {
            Some(t) => paragraph(&at, &format!("{key}/title"), &t.children, cx).into_any_element(),
            None => panel.label().into_any_element()
        };
        let summary = div().id(SharedString::from(format!("{key}/summary"))).flex().items_center().gap_1().font_weight(FontWeight::BOLD)
            .children(toggles.then(|| (st.chevron)(shown, cx))).child(title)
            .when(toggles, |s| s.cursor_pointer().hover(move |s| s.text_color(st.hover)).on_click(move |_, _, cx| open.update(cx, |o, cx| { *o = !*o; cx.notify() })))
            .debug_selector(|| format!("{}/panel/summary", at.doc));
        let body = shown.then(|| blocks(&at, &panel.body, cx).debug_selector(|| format!("{}/panel/body", at.doc)));
        at.tag(div().flex().flex_col().gap_1().when(!panel.kind.is_empty(), |d| d.p_2().rounded_md().border_l_2().border_color(st.link))
            .child(summary).children(body), "panel")
    }
}

/// `doc` as GPUI elements, footnotes last. `id` names the document: it keys panel state and prefixes element ids, so give each document its own.
pub fn mdhtml2gpui(id: impl Into<SharedString>, doc: &Document, cx: &App) -> Div { render(At { doc: id.into(), path: String::new(), map: None }, doc, cx) }

fn render(at: At, doc: &Document, cx: &App) -> Div {
    let st = style(cx);
    let notes = (!doc.footnotes.is_empty()).then(|| {
        let at = At { path: "fn".into(), ..at.clone() };
        at.tag(div().flex().flex_col().gap_1().pt_2().border_t_1().border_color(st.border).text_sm().text_color(st.muted)
            .children(doc.footnotes.iter().enumerate().map(|(i, n)| div().flex().gap_2().child(format!("[{}]", n.label)).child(blocks(&at.child(i), &n.blocks, cx)))),
            "footnotes")
    });
    blocks(&at, &doc.blocks, cx).children(notes)
}

/// Markdown `src`, parsed with mdhtml's default options, as GPUI elements.
pub fn md2gpui(id: impl Into<SharedString>, src: &str, cx: &App) -> Div { mdhtml2gpui(id, &mdhtml::parse(src, &Options::default()), cx) }

/// Like [`md2gpui`], with a [`SourceMap`] that maps a point in the rendered document back to a byte offset in `src`.
pub fn md2gpui_mapped(id: impl Into<SharedString>, src: &str, cx: &App) -> (Div, Rc<SourceMap>) {
    let map = Rc::new(SourceMap::new(src));
    let seen = map.clone();
    let el = render(At { doc: id.into(), path: String::new(), map: Some(map.clone()) }, &mdhtml::parse(src, &Options::default()), cx)
        .on_children_prepainted(move |bounds, _, _| *seen.blocks.borrow_mut() = bounds);
    (el, map)
}
