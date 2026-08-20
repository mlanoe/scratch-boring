//! Regression test for a real, previously-diagnosed-but-unfixed gap:
//! `rasterize_svg_to_png` (`assets.br`) used `usvg::Options::default()`
//! unchanged, whose own `fontdb` field starts genuinely empty (confirmed
//! against real `usvg` source) -- with no system fonts loaded, every
//! `<text>` element in an SVG costume silently failed to rasterize at
//! all (`usvg`'s own real runtime warning, `No match for "Sans Serif"
//! font-family`, first surfaced investigating a real downloaded project
//! whose "shop item" costumes bake text directly into their own SVG
//! art -- see README.md's own "Status" section). Fixed by loading system
//! fonts (`fontdb::Database::load_system_fonts`) into a fresh database
//! and wiring it into `Options.fontdb` before every rasterization call.
//!
//! This test rasterizes a small, self-contained SVG with a white
//! background and a large black "Sans Serif" `<text>` glyph, then
//! decodes the resulting PNG (via `resvg`'s own re-exported `tiny_skia`,
//! already a direct dependency -- no new crate needed) and checks that
//! at least one pixel is genuinely dark. Without the fix, `usvg` parses
//! the document fine (no error, no panic) but silently omits every glyph
//! -- the canvas would be a uniform white image, and this exact
//! assertion is what would have caught that regression.

use resvg::tiny_skia::Pixmap;
use scratch_boring::rasterize_svg_to_png;

const SVG_WITH_TEXT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40">
  <rect width="100" height="40" fill="#ffffff"/>
  <text x="5" y="30" font-family="Sans Serif" font-size="32" fill="#000000">A</text>
</svg>"##;

#[test]
fn svg_text_actually_rasterizes_dark_pixels_not_just_the_blank_background() {
    let svg_bytes: Vec<u8> = SVG_WITH_TEXT.as_bytes().to_vec();
    let png_bytes = rasterize_svg_to_png(&svg_bytes)
        .expect("a well-formed SVG with a <text> element should still rasterize to some PNG bytes");

    let pixmap = Pixmap::decode_png(&png_bytes).expect("rasterize_svg_to_png should produce a decodable PNG");

    let has_dark_pixel = pixmap.pixels().iter().any(|p| {
        // Premultiplied color components, u8 001-255ish scale -- a
        // genuinely dark (near-black) pixel, not just anti-aliasing noise
        // on an otherwise white canvas.
        (p.red() as u32) < 80 && (p.green() as u32) < 80 && (p.blue() as u32) < 80
    });

    assert!(
        has_dark_pixel,
        "expected at least one dark pixel from the rasterized \"A\" glyph -- if this fails, SVG text is silently not rendering again (usvg's fontdb is empty)"
    );
}
