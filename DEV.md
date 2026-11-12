# Development

## Prerequisites

GPUI needs the platform's windowing and font libraries. On Ubuntu, install the packages in the "Install GPUI's system libraries" step of `.github/workflows/ci.yml`.

## Testing

```bash
cargo test
cargo clippy --all-targets -- -D warnings
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

```bash
cargo test
ship-release
```

`ship-release` pushes the branch and a `v<version>` tag, then bumps the version in `Cargo.toml`. A `v*` tag starts the `publish` job in `.github/workflows/ci.yml`. After the tests pass, it publishes the crate to crates.io through trusted publishing and creates the GitHub release.

Trusted publishing works only for a crate that already exists on crates.io. Publish the first release by hand with `cargo publish` and a crates.io token. Then add this repository's `ci.yml` as a trusted publisher in the crate's crates.io settings. Later tags publish without a token.
