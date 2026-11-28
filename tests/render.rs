use gpui::{Bounds, Context, HighlightStyle, InteractiveElement as _, IntoElement, Modifiers, ParentElement as _, Pixels, Render, Styled as _, TestAppContext, VisualTestContext, Window, div,
    point, px, size};
use mdhtml::{Options, parse};
use mdhtml2gpui::{Anchors, Highlighter, SourceMap, Style, block_starts, fallbacks, image_source, md2gpui_mapped, mdhtml2gpui, mdhtml2gpui_anchored, nearest};
use std::{cell::RefCell, rc::Rc};

/// One of every node kind `md` produces with default options plus implicit figures.
const SAMPLE: &str = r#"# Title

A paragraph with *emphasis*, **strong**, ~~strike~~, `code`, [a link](https://answer.ai), ==mark==, H~2~O, x^2^, a note[^n], \(e^{i\pi}\)
and raw `\LaTeX`{=latex}.

1. one
2. two

- [x] done
- [ ] todo

> A quote.

```python
print("hi")
```

---

| Name | Value |
|:-----|------:|
| a    | 1     |

Term
: Definition

::: details
### Summary

Hidden body.
:::

::: note
A plain div.
:::


::: bibliography
Howard and Gugger {year=2020}
: Deep learning for coders.
:::

![A dot](data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%228%22%20height%3D%228%22%3E%3Ccircle%20cx%3D%224%22%20cy%3D%224%22%20r%3D%224%22%2F%3E%3C%2Fsvg%3E)

\[
\int_0^1 x\,dx
\]

<div>raw html</div>

[^n]: The note.
"#;

fn doc() -> mdhtml::Document { parse(SAMPLE, &Options { implicit_figures: true, ..Default::default() }) }

/// Only the constructs GPUI can't draw fall back to their source: maths, raw HTML and raw output for another format.
#[test]
fn only_undrawable_kinds_fall_back() {
    assert_eq!(fallbacks(&doc()), ["math", "raw", "math", "html"]);
}

/// Each displayed block maps to its source line, and a click maps to the nearest block.
#[test]
fn blocks_map_back_to_source() {
    let src = "# A\n\npara\n\n[x]: https://x\n\n- item\n";
    assert_eq!(block_starts(src), [0, 5, 27]);
    let rows = [Bounds::new(point(px(0.), px(0.)), size(px(10.), px(10.))), Bounds::new(point(px(0.), px(20.)), size(px(10.), px(10.)))];
    assert_eq!([nearest(&rows, px(5.)), nearest(&rows, px(16.)), nearest(&rows, px(99.))], [0, 1, 1]);
}

/// Data URLs decode in the crate; other sources pass through as a URI or a file path.
#[gpui::test]
fn images_resolve_their_sources(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let svg = image_source("data:image/svg+xml;utf8,%3Csvg%2F%3E", cx);
        assert!(matches!(svg, gpui::ImageSource::Image(ref i) if i.format == gpui::ImageFormat::Svg && i.bytes == b"<svg/>"));
        let png = image_source("data:image/png;base64,iVBORw0K", cx);
        assert!(matches!(png, gpui::ImageSource::Image(ref i) if i.format == gpui::ImageFormat::Png && i.bytes.starts_with(b"\x89PNG")));
        assert!(matches!(image_source("https://x/a.png", cx), gpui::ImageSource::Resource(gpui::Resource::Uri(_))));
        assert!(matches!(image_source("pics/a.png", cx), gpui::ImageSource::Resource(gpui::Resource::Path(_))));
    });
}

struct Two;

impl Render for Two {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(600.)).child(mdhtml2gpui("a", &doc(), cx)).child(mdhtml2gpui("b", &doc(), cx))
    }
}

/// Every top-level block draws, and a panel's open state belongs to its document, not to its position.
#[gpui::test]
fn documents_render_and_keep_their_own_panel_state(cx: &mut TestAppContext) {
    let window = cx.add_window(|_, _| Two);
    let cx = &mut VisualTestContext::from_window(*window, cx);
    cx.run_until_parked();
    for sel in ["a/heading", "a/paragraph", "a/list", "a/quote", "a/code", "a/break", "a/table", "a/definitions", "a/panel", "a/div", "a/bibliography", "a/figure",
        "a/math", "a/html", "a/footnotes"] {
        assert!(cx.debug_bounds(sel).is_some(), "{sel} not drawn");
    }
    assert!(cx.debug_bounds("a/panel/body").is_none() && cx.debug_bounds("b/panel/body").is_none());
    let summary: Bounds<Pixels> = cx.debug_bounds("a/panel/summary").unwrap();
    cx.simulate_click(summary.center(), Modifiers::default());
    cx.run_until_parked();
    assert!(cx.debug_bounds("a/panel/body").is_some() && cx.debug_bounds("b/panel/body").is_none());
}

const NOTE: &str = "Some **bold** and a [link](https://x.y) with `code`.\n\n1. first item\n2. second\n";

struct Mapped(Option<std::rc::Rc<SourceMap>>);

impl Render for Mapped {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (el, map) = md2gpui_mapped("m", NOTE, cx);
        self.0 = Some(map);
        div().w(px(600.)).child(el)
    }
}

/// A click on rendered text maps to that character in the source; a click on a generated list number falls back to its block's start.
#[gpui::test]
fn clicks_map_to_source_characters(cx: &mut TestAppContext) {
    let window = cx.add_window(|_, _| Mapped(None));
    let cx = &mut VisualTestContext::from_window(*window, cx);
    cx.run_until_parked();
    let map = window.update(cx, |v, _, _| v.0.clone().unwrap()).unwrap();
    // The first paragraph is the first line; `m/paragraph` would also match the list items' paragraphs.
    let at = map.offset_at(point(px(60.), px(8.)));
    assert!(at > 0 && at < NOTE.find("\n\n").unwrap() && NOTE[at..].starts_with(|c: char| c.is_alphanumeric() || c == ' '), "{at}");
    let list: Bounds<Pixels> = cx.debug_bounds("m/list").unwrap();
    assert_eq!(map.offset_at(list.origin + point(px(2.), px(4.))), NOTE.find("1.").unwrap());
}

/// A code block's text goes to the app's highlighter with the block's language and without its final newline, and its ranges colour the
/// text. Source shown for content GPUI can't draw, such as maths and raw HTML, isn't highlighted.
#[gpui::test]
fn code_blocks_go_to_the_highlighter(cx: &mut TestAppContext) {
    let seen = Rc::new(RefCell::new(vec![]));
    let log = seen.clone();
    let highlight: Highlighter = Rc::new(move |lang, text, _| {
        log.borrow_mut().push((lang.map(String::from), text.to_string()));
        vec![(0..5, HighlightStyle { color: Some(gpui::red()), ..Default::default() })]
    });
    cx.update(|cx| cx.set_global(Style { highlight: Some(highlight), ..Style::default() }));
    let window = cx.add_window(|_, _| Two);
    VisualTestContext::from_window(*window, cx).run_until_parked();
    let seen = seen.borrow();
    assert!(!seen.is_empty() && seen.iter().all(|s| *s == (Some("python".into()), "print(\"hi\")".into())), "{seen:?}");
}

/// Baked table and figure captions use `AssignedNumber::display` directly without duplicating `Table Table 1:` or `Figure Figure 1:`.
#[gpui::test]
fn baked_captions_do_not_duplicate_prefix(cx: &mut TestAppContext) {
    use mdhtml::resolve::{BakeOptions, RefsMode, bake_document};
    let bake = |src: &str| {
        let mut doc = parse(src, &Options { implicit_figures: true, ..Default::default() });
        bake_document(&mut doc, &BakeOptions { refs: RefsMode::Lenient, ..Default::default() }).unwrap();
        doc
    };
    let tbl = bake("| A |\n|:--|\n| 1 |\n: Summary {#tbl-1}\n");
    let fig = bake("![Cap](data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%228%22%20height%3D%228%22%2F%3E){#fig-1}\n");

    struct View(mdhtml::Document, mdhtml::Document);
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let row = |label: &'static str, text: &'static str, sel: &'static str| {
                div().text_sm().flex().gap_1().child(label).child(text).debug_selector(move || sel.into())
            };
            div().w(px(600.)).flex().flex_col().items_start()
                .child(mdhtml2gpui("tbl", &self.0, cx)).child(mdhtml2gpui("fig", &self.1, cx))
                .child(row("Table 1:", "Summary", "ref/tbl")).child(row("Table Table 1:", "Summary", "ref/tbl-dup"))
                .child(row("Figure 1:", "Cap", "ref/fig")).child(row("Figure Figure 1:", "Cap", "ref/fig-dup"))
        }
    }

    let window = cx.add_window(|_, _| View(tbl, fig));
    let cx = &mut VisualTestContext::from_window(*window, cx);
    cx.run_until_parked();

    for (sel, ok_sel, dup_sel) in [("tbl/table", "ref/tbl", "ref/tbl-dup"), ("fig/figure", "ref/fig", "ref/fig-dup")] {
        let (cap, ok, dup) = (cx.debug_bounds(sel).unwrap().size.width, cx.debug_bounds(ok_sel).unwrap().size.width, cx.debug_bounds(dup_sel).unwrap().size.width);
        assert!((f32::from(cap) - f32::from(ok)).abs() < 2., "{sel}: got {cap:?}, expected ~{ok:?} (dup is {dup:?})");
        assert!(cap < dup - px(10.), "{sel}: width {cap:?} looks duplicated against {dup:?}");
    }
}

/// Wide or tall images shrink proportionally to fit their container width without vertical letterboxing, small images stay at natural size,
/// and broken images fall back to `[alt]`.
#[gpui::test]
fn images_scale_height_proportionally_without_letterboxing(cx: &mut TestAppContext) {
    let svg = |w: u32, h: u32| format!(
        "data:image/svg+xml;utf8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%22{w}%22%20height%3D%22{h}%22%3E%3Crect%20width%3D%22{w}%22%20height%3D%22{h}%22%20fill%3D%22%23c44%22%2F%3E%3C%2Fsvg%3E"
    );
    let mk = |md: String| parse(&md, &Options::default());
    let docs = vec![
        ("sq", mk(format!("![sq]({})\n", svg(1200, 1200)))),
        ("wide", mk(format!("![wide]({})\n", svg(1200, 600)))),
        ("small", mk(format!("![small]({})\n", svg(16, 16)))),
        ("missing", mk("![missing](/nonexistent-mdhtml2gpui-test.png)\n".into())),
    ];

    struct View(Vec<(&'static str, mdhtml::Document)>);
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().w(px(400.)).flex().flex_col().children(self.0.iter().map(|(id, d)| mdhtml2gpui(*id, d, cx)))
        }
    }

    let window = cx.add_window(|_, _| View(docs));
    let cx = &mut VisualTestContext::from_window(*window, cx);
    for _ in 0..3 { cx.run_until_parked(); cx.refresh().unwrap(); }

    for (sel, want) in [("sq/paragraph", 400.), ("wide/paragraph", 200.), ("small/paragraph", 16.)] {
        let got = f32::from(cx.debug_bounds(sel).unwrap_or_else(|| panic!("missing {sel}")).size.height);
        assert!((got - want).abs() < 2., "{sel}: expected ~{want}px tall, got {got}px");
    }
    let fb = f32::from(cx.debug_bounds("missing/paragraph").unwrap().size.height);
    assert!(fb > 0. && fb < 50., "missing image fallback should render compact alt text, got {fb}px");
}

/// Anchored rendering captures block, nested, inline, and footnote target bounds, and `Style::on_link` receives cross-ref, footnote, and backlink clicks.
#[gpui::test]
fn anchors_and_links_report_bounds_and_clicks(cx: &mut TestAppContext) {
    use mdhtml::resolve::{BakeOptions, RefsMode, bake_document};
    let clicked = Rc::new(RefCell::new(vec![]));
    let log = clicked.clone();
    cx.update(|cx| cx.set_global(Style { on_link: Some(Rc::new(move |u, _, _| log.borrow_mut().push(u.to_string()))), ..Style::default() }));
    let src = "::: {#p-ref}\n[@sec-b]\n:::\n\n[^n] [tag]{#inl}\n\n::: note\n## Target {#sec-b}\n:::\n\nTerm {#term-a}\n: Def\n\n::: bibliography\nAuthor {#ref-a year=2025}\n: Book\n:::\n\n[^n]: Footnote.\n";
    let mut doc = parse(src, &Options::default());
    let _ = bake_document(&mut doc, &BakeOptions { refs: RefsMode::Lenient, ..Default::default() });
    struct View(mdhtml::Document, Rc<Anchors>);
    impl Render for View {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let (el, anchors) = mdhtml2gpui_anchored("a", &self.0, cx);
            self.1 = anchors;
            div().w(px(500.)).child(el)
        }
    }
    let window = cx.add_window(|_, _| View(doc, Rc::default()));
    let cx = &mut VisualTestContext::from_window(*window, cx);
    cx.run_until_parked();
    let anchors = window.read_with(cx, |v, _| v.1.clone()).unwrap();
    for id in ["p-ref", "inl", "sec-b", "term-a", "ref-a", "fn-n", "fnref-n"] {
        assert!(anchors.get(id).is_some(), "missing anchor {id}");
    }
    assert!(anchors.get("sec-b").unwrap().top() < anchors.get("fn-n").unwrap().top());
    for id in ["p-ref", "fnref-n"] {
        let b = anchors.get(id).unwrap();
        cx.simulate_click(b.origin + point(px(8.), b.size.height / 2.), Modifiers::default());
    }
    let back = cx.debug_bounds("a/footnotes/back").unwrap().center();
    cx.simulate_click(back, Modifiers::default());
    cx.run_until_parked();
    assert_eq!(*clicked.borrow(), ["#sec-b", "#fn-n", "#fnref-n"]);
}
