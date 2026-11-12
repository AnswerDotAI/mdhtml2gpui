# mdhtml2gpui

`mdhtml2gpui` renders Markdown as [GPUI](https://www.gpui.rs) elements, for Rust apps built on GPUI. It parses with [mdhtml](https://github.com/AnswerDotAI/mdhtml) and draws each block and inline kind of mdhtml's `Document` with plain GPUI elements. It needs no component library. Content GPUI can't draw, such as maths, shows as its source.

Add it with the GPUI version it is built against:

```toml
[dependencies]
gpui = { package = "gpui-pre", version = "=0.3.12" }
mdhtml2gpui = "0.1"
```

## Usage

```rust
use mdhtml2gpui::{Style, md2gpui};

// Once, at startup.
cx.set_global(Style { mono_font: "JetBrains Mono".into(), ..Style::default() });

// In a view's render, with one id per document.
div().text_color(fg).child(md2gpui("note-1", "# Hello\n\nSome *text*.", cx))
```

`md2gpui` parses Markdown with mdhtml's default options. To parse once and render the same document on many frames, parse it with `mdhtml::parse` from the `mdhtml-crate` package and pass the `Document` to `mdhtml2gpui`. Give each document its own id. The id keys the open state of the document's disclosure panels and prefixes its element ids.

## Style

Set a `Style` as a GPUI global to choose colours, fonts and image loading. Without one, the crate uses `Style::default()`, which suits a light background. Start from `Style::dark()` for a dark background. Text colour and the body font come from the parent element.

- `code_bg`: background of inline code and code blocks.
- `border`: rules, table lines and block quote bars.
- `muted`: quotes, captions, footnotes and the source of content GPUI can't draw.
- `link`: links, footnote references, cross-references and the border of callouts.
- `hover`: a disclosure title under the pointer.
- `mark`: background of highlighted text, `==marked==` and `<mark>`.
- `mono_font` and `mono_size`: the font for code. Use a real family name, because GPUI doesn't resolve the generic family `monospace`. The default is Menlo on macOS, Consolas on Windows and DejaVu Sans Mono elsewhere.
- `chevron`: draws the disclosure marker for an open or closed panel. The default is a text triangle.
- `load`: returns the image source for a URL, for example to fetch it with the app's credentials.

The crate decodes data URLs itself, whether or not `load` is set. It passes other URLs to `load` when it is set. Without `load`, a URL containing `://` goes to GPUI's loader and anything else is read as a file path. `image_source(url, cx)` applies these rules.

## Supported content

Blocks:

- headings, with their numbers
- paragraphs, including hard breaks
- ordered, bullet and task lists
- block quotes
- code blocks, in the monospace font
- thematic breaks
- tables, with column alignment, header rows and captions
- definition lists
- panels. Callouts have a left border in the link colour. Clicking a disclosure's title opens or closes its body. The document id and the panel's position key its open state.
- divs, and HTML elements that mdhtml parsed, drawn as their children
- bibliographies
- figures, with captions
- footnotes, collected at the end

Inlines: emphasis, strong, strikethrough, highlight, code, links, footnote references, cross-references, spans, and HTML elements such as `kbd`, `u` and `mark`. Clicking a link opens its URL. GPUI can't place an image inside a line of text. An image, SVG included, splits its paragraph and appears between the text before it and the text after it. An image that fails to load shows its alt text in brackets.

Superscript and subscript draw as plain text, because GPUI text runs can't change their baseline or size. Table cells ignore `colspan` and `rowspan`.

These show their source in the muted monospace style:

- maths, inline and display. Drawing it would need a TeX renderer such as RaTeX.
- raw HTML that mdhtml keeps opaque
- raw content for another format, such as `{=latex}`
- scripts and template operations

`fallbacks(&doc)` lists, in document order, the kinds in a document that show their source. Each is one of `math`, `html`, `raw`, `script` or `operation`.

## Source mapping

`block_starts(src)` returns the byte offset where each displayed top-level block of `src` starts. It skips link reference and attribute definitions. `nearest(&bounds, y)` returns the index of the bounds nearest to height `y`. With the rendered blocks' bounds, they map a click to the start of a block's source, for example to place an editor's cursor.

`md2gpui_mapped(id, src, cx)` renders like `md2gpui` and also returns a `SourceMap`. `map.offset_at(point)` returns the source byte offset of the character under `point`. It matches each rendered character of a top-level block, in order, against the source's text runs, inline code and line ends. The match skips markup, link URLs and list markers. A click on generated text such as a list number, in a footnote, or outside any text returns the start of the nearest block. `offset_at` does the matching on its first call, not during rendering.

`map.segments()` returns the rendered text and `TextLayout` of each recorded segment, in drawing order. An app can register these with its own text-selection system. Footnotes and generated text, such as list numbers, aren't recorded.

## Development

See [DEV.md](DEV.md).
