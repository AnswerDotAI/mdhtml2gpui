# mdhtml2gpui

`mdhtml2gpui` renders [mdhtml](https://github.com/AnswerDotAI/mdhtml) documents as [GPUI](https://www.gpui.rs) elements. It walks mdhtml's semantic `Document` and maps each block and inline kind to plain GPUI elements, so it depends on GPUI and `mdhtml-crate` alone and works in any GPUI app, with or without a component library. Content GPUI can't draw shows as its source.

Add it with the GPUI version it is built against:

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.12" }
mdhtml2gpui = "0.1"
```

## Usage

```rust
use mdhtml2gpui::{Style, md2gpui};

// Once, at startup. Text colour and the body font come from the parent element.
cx.set_global(Style { mono_font: "JetBrains Mono".into(), ..Style::default() });

// In a view's render: one id per document, which keys its panels' open state.
div().text_color(fg).child(md2gpui("note-1", "# Hello\n\nSome *text*.", cx))
```

`md2gpui` parses Markdown with mdhtml's default options. `mdhtml2gpui` takes an already parsed `Document`, so an app can parse once and keep the document while it renders many frames.

`Style` holds the code background, border, muted text, link and hover colours, the monospace font and size, and whether the background is dark. `Style::dark()` is a starting point for dark themes. `Style::chevron` draws the disclosure marker, a text triangle by default. `Style::load` returns the image source for URLs that aren't data URLs, for example to fetch them with the app's credentials. Without it, URLs go to GPUI's own loader and other sources are read as file paths. GPUI doesn't resolve the generic family `monospace`, so the default monospace font is Menlo on macOS, Consolas on Windows and DejaVu Sans Mono elsewhere.

## Supported content

Blocks:

- headings, with assigned numbers
- paragraphs, including hard breaks
- ordered, bullet and task lists
- block quotes
- code blocks, in the monospace font
- thematic breaks
- tables, with column alignment, header rows and captions
- definition lists
- panels: callouts and disclosures. A disclosure's title toggles its body, and its open state is keyed by the document id and the panel's position.
- divs and HTML elements mdhtml parsed, drawn as their children
- bibliographies
- figures, with captions
- footnotes, collected at the end

Inlines: emphasis, strong, strikethrough, highlight, code, links (clicking opens the URL), footnote references, references, spans, and HTML elements such as `kbd`, `u` and `mark`. Images, SVG included, appear between lines of text, since GPUI can't place an image inside a line. Data URLs decode in the crate.

These show their source in the muted monospace style. `fallbacks(&doc)` lists the ones a document contains:

- maths, inline and display; drawing it needs a TeX renderer such as RaTeX, which this crate doesn't depend on
- raw HTML that mdhtml keeps opaque
- raw content for another format, such as `{=latex}`
- scripts and template operations

Superscript and subscript draw as plain text, because GPUI text runs can't change their baseline or size. Table cells ignore `colspan` and `rowspan`.

## Source mapping

`block_starts(src)` gives the byte offset where each displayed top-level block starts, and `nearest(&bounds, y)` picks the block nearest a click. Together they map a click on rendered content back to its source, for example to place an editor's cursor.

## Development

See [DEV.md](DEV.md).

## Release

```bash
cargo test
ship-release
```

`ship-release` pushes a `v<version>` tag, which starts the release workflow in GitHub Actions, then bumps the version. The workflow publishes the crate to crates.io and creates the GitHub release. For the first release, publish manually with a token (`cargo publish`), then add this repo's `ci.yml` as a trusted publisher in the crate's crates.io settings. Later tags publish through CI without a token.
