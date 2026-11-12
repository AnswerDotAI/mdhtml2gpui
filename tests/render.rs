use gpui::{Bounds, Context, IntoElement, Modifiers, ParentElement as _, Pixels, Render, Styled as _, TestAppContext, VisualTestContext, Window, div, point, px, size};
use mdhtml::{Options, parse};
use mdhtml2gpui::{block_starts, fallbacks, image_source, mdhtml2gpui, nearest};

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
