//! Minimal software raster: 32-bit pixel buffers, clipped blits and fills.
//!
//! Pixels are `0xAARRGGBB`. Skin bitmaps are always opaque; alpha 0 marks
//! "no pixel" (used for out-of-range sprite reads and region masks).

#[derive(Clone, Debug)]
pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u32>,
}

pub const TRANSPARENT: u32 = 0x0000_0000;

pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

impl Bitmap {
    pub fn new(w: i32, h: i32, fill: u32) -> Bitmap {
        let w = w.max(0);
        let h = h.max(0);
        Bitmap {
            w,
            h,
            px: vec![fill; (w * h) as usize],
        }
    }

    pub fn from_rgba8(w: u32, h: u32, data: &[u8]) -> Bitmap {
        let mut px = Vec::with_capacity((w * h) as usize);
        for c in data.as_chunks::<4>().0 {
            px.push(
                ((c[3] as u32) << 24) | ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | c[2] as u32,
            );
        }
        Bitmap {
            w: w as i32,
            h: h as i32,
            px,
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u32 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            TRANSPARENT
        } else {
            self.px[(y * self.w + x) as usize]
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: u32) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    pub fn fill(&mut self, c: u32) {
        self.px.fill(c);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.w);
        let y1 = (y + h).min(self.h);
        for yy in y0..y1 {
            let row = (yy * self.w) as usize;
            self.px[row + x0 as usize..row + x1.max(x0) as usize].fill(c);
        }
    }

    /// Copy a `w`x`h` region from `src` at (`sx`,`sy`) to (`dx`,`dy`).
    /// Source pixels outside `src` (or transparent ones) are skipped.
    pub fn blit(&mut self, src: &Bitmap, sx: i32, sy: i32, w: i32, h: i32, dx: i32, dy: i32) {
        for y in 0..h {
            let ty = dy + y;
            let fy = sy + y;
            if ty < 0 || ty >= self.h || fy < 0 || fy >= src.h {
                continue;
            }
            for x in 0..w {
                let tx = dx + x;
                let fx = sx + x;
                if tx < 0 || tx >= self.w || fx < 0 || fx >= src.w {
                    continue;
                }
                let c = src.px[(fy * src.w + fx) as usize];
                if c >> 24 != 0 {
                    self.px[(ty * self.w + tx) as usize] = c;
                }
            }
        }
    }

    /// Like `blit` but clips the destination to the rectangle `clip`.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_clipped(
        &mut self,
        src: &Bitmap,
        sx: i32,
        sy: i32,
        w: i32,
        h: i32,
        dx: i32,
        dy: i32,
        clip: Rect,
    ) {
        for y in 0..h {
            let ty = dy + y;
            if ty < clip.y || ty >= clip.y + clip.h {
                continue;
            }
            for x in 0..w {
                let tx = dx + x;
                if tx < clip.x || tx >= clip.x + clip.w {
                    continue;
                }
                let c = src.get(sx + x, sy + y);
                if c >> 24 != 0 {
                    self.set(tx, ty, c);
                }
            }
        }
    }

    /// Draw a 1px rectangle outline.
    pub fn rect_outline(&mut self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        self.fill_rect(x, y, w, 1, c);
        self.fill_rect(x, y + h - 1, w, 1, c);
        self.fill_rect(x, y, 1, h, c);
        self.fill_rect(x + w - 1, y, 1, h, c);
    }

    pub fn hline(&mut self, x: i32, y: i32, w: i32, c: u32) {
        self.fill_rect(x, y, w, 1, c);
    }

    pub fn vline(&mut self, x: i32, y: i32, h: i32, c: u32) {
        self.fill_rect(x, y, 1, h, c);
    }

    /// Write this bitmap into a `0x00RRGGBB` frame buffer, scaled by an
    /// integer factor with nearest-neighbour sampling.
    pub fn present_into(&self, out: &mut [u32], out_w: usize, out_h: usize, scale: usize) {
        let scale = scale.max(1);
        for oy in 0..out_h {
            let sy = (oy / scale) as i32;
            let dst = &mut out[oy * out_w..(oy + 1) * out_w];
            if sy >= self.h {
                dst.fill(0);
                continue;
            }
            let row = &self.px[(sy * self.w) as usize..((sy + 1) * self.w) as usize];
            for (ox, d) in dst.iter_mut().enumerate() {
                let sx = ox / scale;
                *d = if sx < row.len() {
                    row[sx] & 0x00FF_FFFF
                } else {
                    0
                };
            }
        }
    }

    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(self.px.len() * 4);
        for &p in &self.px {
            v.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8, (p >> 24) as u8]);
        }
        v
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blit_clips_and_skips_transparent() {
        let mut src = Bitmap::new(2, 2, rgb(255, 0, 0));
        src.set(1, 1, TRANSPARENT);
        let mut dst = Bitmap::new(3, 3, rgb(0, 0, 0));
        dst.blit(&src, 0, 0, 2, 2, 2, 2);
        assert_eq!(dst.get(2, 2), rgb(255, 0, 0));
        dst.blit(&src, 0, 0, 2, 2, 0, 0);
        assert_eq!(dst.get(1, 1), rgb(0, 0, 0));
        assert_eq!(dst.get(0, 0), rgb(255, 0, 0));
    }

    #[test]
    fn present_scales() {
        let mut b = Bitmap::new(2, 1, rgb(0, 0, 0));
        b.set(1, 0, rgb(1, 2, 3));
        let mut out = vec![0u32; 4 * 2];
        b.present_into(&mut out, 4, 2, 2);
        assert_eq!(out[2], 0x010203);
        assert_eq!(out[7], 0x010203);
        assert_eq!(out[1], 0);
    }
}
