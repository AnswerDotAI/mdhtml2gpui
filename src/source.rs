use crate::{block_starts, nearest};
use gpui::{Bounds, Pixels, Point, TextLayout};
use mdhtml::Options;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;

/// Where a rendered document's text came from in its Markdown source, for placing a cursor where the user clicked.
pub struct SourceMap { src: String, segs: RefCell<Vec<(usize, String, TextLayout)>>, pub(crate) blocks: RefCell<Vec<Bounds<Pixels>>>, starts: OnceCell<Vec<usize>>, offsets: OnceCell<Vec<Vec<(usize, usize)>>> }

impl SourceMap {
    pub(crate) fn new(src: &str) -> Self { Self { src: src.into(), segs: RefCell::default(), blocks: RefCell::default(), starts: OnceCell::new(), offsets: OnceCell::new() } }

    pub(crate) fn record(&self, block: usize, text: String, layout: TextLayout) { self.segs.borrow_mut().push((block, text, layout)) }

    /// Each recorded text segment's rendered text and layout, in the order they were drawn.
    pub fn segments(&self) -> Vec<(String, TextLayout)> { self.segs.borrow().iter().map(|(_, text, layout)| (text.clone(), layout.clone())).collect() }

    fn starts(&self) -> &[usize] { self.starts.get_or_init(|| block_starts(&self.src)) }

    /// The source byte offset under `point`: the clicked character of the text of a top-level block, or else the start of the nearest block.
    pub fn offset_at(&self, point: Point<Pixels>) -> usize {
        let segs = self.segs.borrow();
        if let Some(i) = segs.iter().position(|(_, _, layout)| layout.bounds().contains(&point)) {
            let offsets = self.offsets.get_or_init(|| align(&self.src, self.starts(), &segs.iter().map(|(b, t, _)| (*b, t.as_str())).collect::<Vec<_>>()));
            if let Some(at) = lookup(&offsets[i], segs[i].2.index_for_position(point).unwrap_or_else(|ix| ix)) { return at }
        }
        self.starts().get(nearest(&self.blocks.borrow(), point.y)).copied().unwrap_or(0)
    }
}

/// The source offset of rendered byte `ix`, from `offsets`' `(rendered, source)` pairs.
fn lookup(offsets: &[(usize, usize)], ix: usize) -> Option<usize> { offsets.iter().rev().find(|(r, _)| *r <= ix).map(|(_, s)| *s) }

/// For each rendered `(block, text)` segment, `(rendered byte, source byte)` pairs for its characters and its end. A top-level block's
/// segments align, in order, with its source characters: mdhtml's text runs, inline code and line ends, or all of the block when it has
/// none, as a code block doesn't. Each rendered character takes the next equal source character within a short window. One without a
/// match, such as an entity's result, maps to where the previous match ended.
fn align(src: &str, starts: &[usize], segs: &[(usize, &str)]) -> Vec<Vec<(usize, usize)>> {
    let runs: Vec<_> = mdhtml::inlines(src, &Options::default()).into_iter().filter(|n| matches!(n.kind, "text" | "code")).map(|n| n.range).collect();
    let chars = |b: usize| -> Vec<(usize, char)> {
        let block = starts[b]..starts.get(b + 1).copied().unwrap_or(src.len());
        let at = |start: usize, s: &str| s.char_indices().map(move |(i, c)| (start + i, c)).collect::<Vec<_>>();
        let mut cs: Vec<_> = runs.iter().filter(|r| block.contains(&r.start)).flat_map(|r| at(r.start, &src[r.clone()])).collect();
        if cs.is_empty() { return at(block.start, &src[block]) }
        cs.extend(src[block.clone()].match_indices('\n').map(|(i, _)| (block.start + i, '\n')));
        cs.sort_unstable();
        cs
    };
    let mut cursors: HashMap<usize, Cursor> = HashMap::new();
    segs.iter().map(|&(b, text)| {
        if b >= starts.len() { return vec![] }
        let cur = cursors.entry(b).or_insert_with(|| { let chars = chars(b); Cursor { end: chars.first().map_or(starts[b], |c| c.0), chars, next: 0 } });
        let mut out: Vec<_> = text.char_indices().map(|(ri, ch)| {
            let same = |c: char| if ch.is_whitespace() { c.is_whitespace() } else { c == ch };
            let hit = (cur.next..cur.chars.len()).take(if ch.is_whitespace() { 8 } else { 64 }).find(|&k| same(cur.chars[k].1));
            let Some(k) = hit else { return (ri, cur.end) };
            let (at, c) = cur.chars[k];
            (cur.next, cur.end) = (k + 1, at + c.len_utf8());
            (ri, at)
        }).collect();
        out.push((text.len(), cur.end));
        out
    }).collect()
}

/// A block's source characters, the index of the next one to match, and where the last match ended.
struct Cursor { chars: Vec<(usize, char)>, next: usize, end: usize }

#[cfg(test)]
mod tests {
    use super::*;

    /// Each rendered character maps to its own character in the source, skipping markup, link URLs and list markers.
    #[test]
    fn rendered_text_aligns_with_its_source() {
        let src = "# Notes\n\nSome **bold** and a [link](https://x.y) with `code` &amp; more.\nSecond line\n\n1. first item\n2. second\n\n> quoted *text*\n\n```py\nx = 1\n```\n";
        let para = "Some bold and a link with code & more. Second line";
        let segs = [(0, "Notes"), (1, para), (2, "first item"), (2, "second"), (3, "quoted text"), (4, "x = 1")];
        let offsets = align(src, &block_starts(src), &segs);
        let at = |seg: usize, word: &str| lookup(&offsets[seg], segs[seg].1.find(word).unwrap());
        for (seg, word, source) in [(0, "Notes", "Notes"), (1, "bold", "bold"), (1, "link", "link]"), (1, "code", "code`"), (1, "more", "more"),
            (1, "Second", "Second"), (2, "item", "item"), (3, "second", "second"), (4, "text", "text*"), (5, "x", "x = 1")] {
            assert_eq!(at(seg, word), src.find(source), "{word}");
        }
    }
}
