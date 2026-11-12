#[path = "../examples/gallery/page.rs"]
mod page;

use gpui::{AppContext as _, Bounds, HeadlessAppContext, WindowBounds, WindowOptions, px, size};
use std::sync::Arc;

/// Renders the gallery page in light and dark mode to `target/gallery-<light|dark>.png`.
#[test]
#[ignore]
fn gallery_screenshots() {
    for dark in [false, true] {
        let mut cx = HeadlessAppContext::with_platform(gpui_platform::current_platform(true).text_system(), Arc::new(()), gpui_platform::current_headless_renderer);
        cx.update(|cx| page::init(dark, cx));
        let bounds = Bounds { origin: Default::default(), size: size(px(900.), px(2000.)) };
        let opts = WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), focus: false, show: false, ..Default::default() };
        let handle = cx.update(|cx| cx.open_window(opts, |_, cx| cx.new(|_| page::Page))).unwrap();
        for _ in 0..3 {
            cx.update_window(handle.into(), |_, window, cx| window.draw(cx).clear(cx)).unwrap();
            cx.run_until_parked();
        }
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/target");
        std::fs::create_dir_all(dir).unwrap();
        cx.capture_screenshot(handle.into()).unwrap().save(format!("{dir}/gallery-{}.png", if dark { "dark" } else { "light" })).unwrap();
    }
}
