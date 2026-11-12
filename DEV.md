# Development

## Prerequisites

GPUI needs the platform's windowing and font libraries. On Ubuntu, CI installs:

```bash
sudo apt-get install -y libfontconfig-dev libwayland-dev libxkbcommon-x11-dev libx11-xcb-dev libvulkan1
```

## Testing

```bash
cargo test
cargo clippy --all-targets
cargo test --test screenshots -- --ignored
cargo run --example gallery
cargo run --example gallery -- --dark
```

`tests/render.rs` holds the story tests. They check which kinds fall back to source, the source mapping and clicks on rendered text, image sources, and a rendered sample of every kind, including panel state staying with its document. Elements carry debug selectors, `<doc>/<kind>`, which GPUI records only in test builds.

`tests/screenshots.rs` is ignored by default. It renders the gallery page off-screen in light and dark mode to `target/gallery-light.png` and `target/gallery-dark.png`.

## Layout

`src/lib.rs` holds the public API: `Style`, `fallbacks`, `block_starts` and `nearest`. `src/block.rs` renders blocks, panels and footnotes. `src/inline.rs` turns inline content into styled text runs, splitting paragraphs around images. `src/image.rs` resolves image sources and decodes data URLs. `src/source.rs` holds `SourceMap`, which aligns rendered text with its source.

`examples/gallery/` is the gallery: `sample.md`, a document with every kind, `page.rs`, the view the screenshot test shares, and `main.rs`.

Formatting is manual; `rustfmt.toml` disables rustfmt, as in mdhtml.

## Release

Publishing runs in GitHub Actions from `.github/workflows/ci.yml` when a `v*` tag is pushed. Release first, then bump: `ship-release` tags `v<version>`, pushes branch and tag, then bumps `Cargo.toml`.
