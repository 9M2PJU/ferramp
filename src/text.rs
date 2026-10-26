//! TrueType text for the playlist and menus (Liberation Sans, metric
//! compatible with the font classic skins name in pledit.txt).

use std::cell::RefCell;
use std::collections::HashMap;

use crate::gfx::{Bitmap, Rect};

static REGULAR: &[u8] = include_bytes!("../assets/fonts/LiberationSans-Regular.ttf");
static BOLD: &[u8] = include_bytes!("../assets/fonts/LiberationSans-Bold.ttf");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weight {
    Regular,
    Bold,
}

struct Glyph {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    advance: f32,
    coverage: Vec<u8>,
}

pub struct Fonts {
    regular: swash::FontRef<'static>,
    bold: swash::FontRef<'static>,
    ctx: RefCell<swash::scale::ScaleContext>,
    cache: RefCell<HashMap<(char, u32, Weight), std::rc::Rc<Glyph>>>,
}

impl Fonts {
    pub fn new() -> Fonts {
        Fonts {
            regular: swash::FontRef::from_index(REGULAR, 0).expect("bundled font"),
            bold: swash::FontRef::from_index(BOLD, 0).expect("bundled font"),
            ctx: RefCell::new(swash::scale::ScaleContext::new()),
            cache: RefCell::new(HashMap::new()),
        }
    }

    fn font(&self, w: Weight) -> swash::FontRef<'static> {
        match w {
            Weight::Regular => self.regular,
            Weight::Bold => self.bold,
        }
    }

    fn glyph(&self, c: char, px: f32, w: Weight) -> std::rc::Rc<Glyph> {
        let key = (c, px.to_bits(), w);
        if let Some(g) = self.cache.borrow().get(&key) {
            return g.clone();
        }
        let font = self.font(w);
        let gid = font.charmap().map(c);
        let advance = font.glyph_metrics(&[]).scale(px).advance_width(gid);
        let mut ctx = self.ctx.borrow_mut();
        // Hinted outlines snap stems and accents to the pixel grid, which
        // keeps small text legible the way the classic GDI rendering did.
        let mut scaler = ctx.builder(font).size(px).hint(true).build();
        let img = swash::scale::Render::new(&[swash::scale::Source::Outline])
            .format(swash::zeno::Format::Alpha)
            .render(&mut scaler, gid);
        let g = match img {
            Some(img) => Glyph {
                left: img.placement.left,
                top: img.placement.top,
                width: img.placement.width as i32,
                height: img.placement.height as i32,
                advance,
                coverage: img.data,
            },
            None => Glyph { left: 0, top: 0, width: 0, height: 0, advance, coverage: vec![] },
        };
        let g = std::rc::Rc::new(g);
        self.cache.borrow_mut().insert(key, g.clone());
        g
    }

    /// Ascent in pixels (distance from the top of a line to the baseline).
    pub fn ascent(&self, px: f32, w: Weight) -> i32 {
        self.font(w).metrics(&[]).scale(px).ascent.round() as i32
    }

    pub fn advance(&self, c: char, px: f32, w: Weight) -> f32 {
        self.glyph(c, px, w).advance
    }

    pub fn measure(&self, text: &str, px: f32, w: Weight) -> i32 {
        let mut x = 0.0f32;
        for c in text.chars() {
            x += self.advance(c, px, w).round();
        }
        x.round() as i32
    }

    /// Draw `text` with its top-left at (x, y). Glyphs are snapped to whole
    /// pixels; `crisp` renders without anti-aliasing (classic GDI look).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, dst: &mut Bitmap, text: &str, x: i32, y: i32, px: f32, w: Weight, color: u32, clip: Rect, crisp: bool) -> i32 {
        let baseline = y + self.ascent(px, w);
        let mut pen = x as f32;
        for c in text.chars() {
            let g = self.glyph(c, px, w);
            let gx = pen.round() as i32 + g.left;
            let gy = baseline - g.top;
            for row in 0..g.height {
                for col in 0..g.width {
                    let a = g.coverage[(row * g.width + col) as usize];
                    let tx = gx + col as i32;
                    let ty = gy + row as i32;
                    if !clip.contains(tx, ty) || tx < 0 || ty < 0 || tx >= dst.w || ty >= dst.h {
                        continue;
                    }
                    if crisp {
                        if a >= 110 {
                            dst.set(tx, ty, color);
                        }
                    } else if a > 0 {
                        let bg = dst.get(tx, ty);
                        dst.set(tx, ty, blend(bg, color, a));
                    }
                }
            }
            pen += g.advance.round();
        }
        pen.round() as i32 - x
    }

    /// Truncate `text` with an ellipsis so that it fits in `max_w` pixels.
    pub fn ellipsize(&self, text: &str, px: f32, w: Weight, max_w: i32) -> String {
        if self.measure(text, px, w) <= max_w {
            return text.to_string();
        }
        let ell = "...";
        let ell_w = self.measure(ell, px, w);
        let mut out = String::new();
        let mut width = 0;
        for c in text.chars() {
            let cw = self.advance(c, px, w).round() as i32;
            if width + cw + ell_w > max_w {
                break;
            }
            out.push(c);
            width += cw;
        }
        out.push_str(ell);
        out
    }
}

pub fn blend(bg: u32, fg: u32, a: u8) -> u32 {
    let a = a as u32;
    let ia = 255 - a;
    let ch = |shift: u32| ((((fg >> shift) & 255) * a + ((bg >> shift) & 255) * ia) / 255) << shift;
    0xFF00_0000 | ch(16) | ch(8) | ch(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::rgb;

    #[test]
    fn measure_and_draw() {
        let f = Fonts::new();
        let w = f.measure("Hello", 11.0, Weight::Regular);
        assert!(w > 15 && w < 40, "{w}");
        let mut b = Bitmap::new(60, 20, rgb(0, 0, 0));
        f.draw(&mut b, "Hi", 2, 2, 11.0, Weight::Regular, rgb(255, 255, 255), Rect::new(0, 0, 60, 20), true);
        assert!(b.px.iter().any(|&p| p == rgb(255, 255, 255)));
        let e = f.ellipsize("A very long playlist title indeed", 11.0, Weight::Regular, 60);
        assert!(e.ends_with("...") && f.measure(&e, 11.0, Weight::Regular) <= 60);
    }
}
