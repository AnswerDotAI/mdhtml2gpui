//! `cargo run --example gallery [-- --dark]` shows a sample of every node kind.

mod page;

use gpui::{AppContext as _, WindowOptions};

fn main() {
    let dark = std::env::args().any(|a| a == "--dark");
    gpui_platform::application().run(move |cx| {
        page::init(dark, cx);
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| page::Page)).expect("open the gallery window");
        cx.activate(true);
    });
}
