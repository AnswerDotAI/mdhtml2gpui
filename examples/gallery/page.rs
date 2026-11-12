use gpui::{App, Context, Global, InteractiveElement as _, IntoElement, ParentElement as _, Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window,
    div, hsla, Hsla};
use mdhtml::{Options, parse};
use mdhtml2gpui::{Style, mdhtml2gpui};

pub const SAMPLE: &str = include_str!("sample.md");

/// The page's colours, and its body font, which the renderer inherits.
pub struct Body { font: SharedString, bg: Hsla, fg: Hsla }

impl Global for Body {}

/// The sample document on the page that the [`Body`] global describes.
pub struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = cx.global::<Body>();
        let doc = parse(SAMPLE, &Options { implicit_figures: true, ..Default::default() });
        div().id("page").size_full().overflow_y_scroll().bg(body.bg).text_color(body.fg).font_family(body.font.clone()).p_6().child(mdhtml2gpui("sample", &doc, cx))
    }
}

/// The first of `names` that's installed, or else the first.
fn installed(names: &[&str], cx: &App) -> SharedString {
    let all = cx.text_system().all_font_names();
    names.iter().find(|n| all.iter().any(|a| a == *n)).unwrap_or(&names[0]).to_string().into()
}

/// Sets the light or `dark` style, with installed body and monospace fonts.
pub fn init(dark: bool, cx: &mut App) {
    let mono_font = installed(&["DejaVu Sans Mono", "Noto Sans Mono", "Liberation Mono", "Menlo", "Consolas"], cx);
    let (bg, fg) = if dark { (hsla(0.61, 0.15, 0.08, 1.), hsla(0.61, 0.1, 0.88, 1.)) } else { (hsla(0., 0., 1., 1.), hsla(0.61, 0.15, 0.15, 1.)) };
    cx.set_global(Body { font: installed(&["Noto Sans", "Liberation Sans", "DejaVu Sans", "Helvetica", "Segoe UI"], cx), bg, fg });
    cx.set_global(Style { mono_font, ..if dark { Style::dark() } else { Style::default() } });
}
