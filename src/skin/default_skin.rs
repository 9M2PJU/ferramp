//! The built-in skin, drawn procedurally at start-up.
//!
//! Every bitmap is produced by the code in this file (original artwork); it
//! follows the classic skin layout so that every window element sits where a
//! classic skin expects it. Any `.wsz` skin can replace it.

use super::Skin;
use super::sprites::Sheet;
use crate::gfx::{Bitmap, TRANSPARENT, rgb};

// Palette.
const BODY: u32 = rgb(58, 60, 78);
const BODY_HI: u32 = rgb(104, 108, 132);
const BODY_LO: u32 = rgb(30, 31, 42);
const BODY_DARK: u32 = rgb(20, 20, 28);
const FACE: u32 = rgb(84, 87, 108);
const FACE_HI: u32 = rgb(150, 154, 178);
const FACE_LO: u32 = rgb(44, 45, 60);
const LCD_BG: u32 = rgb(0, 0, 0);
const LCD_FG: u32 = rgb(0, 236, 0);
const LCD_DIM: u32 = rgb(0, 120, 0);
const LABEL: u32 = rgb(196, 200, 224);
const TITLE_ON: u32 = rgb(224, 228, 255);
const TITLE_OFF: u32 = rgb(128, 130, 152);
const ACCENT: u32 = rgb(232, 168, 40);

pub const DEFAULT_VISCOLORS: [u32; 24] = [
    rgb(0, 0, 0),
    rgb(24, 33, 41),
    rgb(239, 49, 16),
    rgb(206, 41, 16),
    rgb(214, 90, 0),
    rgb(214, 102, 0),
    rgb(214, 115, 0),
    rgb(198, 123, 8),
    rgb(222, 165, 24),
    rgb(214, 181, 33),
    rgb(189, 222, 41),
    rgb(148, 222, 33),
    rgb(41, 206, 16),
    rgb(50, 190, 16),
    rgb(57, 181, 16),
    rgb(49, 156, 8),
    rgb(41, 148, 0),
    rgb(24, 132, 8),
    rgb(255, 255, 255),
    rgb(214, 214, 222),
    rgb(181, 189, 189),
    rgb(160, 170, 175),
    rgb(148, 156, 165),
    rgb(150, 150, 150),
];

pub fn build() -> Skin {
    let mut sheets: Vec<Option<Bitmap>> = vec![None; Sheet::ALL.len()];
    let mut put = |s: Sheet, b: Bitmap| sheets[s.index()] = Some(b);
    put(Sheet::Text, text_bmp());
    put(Sheet::Numbers, numbers_bmp());
    put(Sheet::Main, main_bmp());
    put(Sheet::TitleBar, titlebar_bmp());
    put(Sheet::CButtons, cbuttons_bmp());
    put(Sheet::PlayPaus, playpaus_bmp());
    put(Sheet::MonoSter, monoster_bmp());
    put(Sheet::PosBar, posbar_bmp());
    put(Sheet::Volume, volume_bmp());
    put(Sheet::Balance, balance_bmp());
    put(Sheet::ShufRep, shufrep_bmp());
    put(Sheet::EqMain, eqmain_bmp());
    put(Sheet::EqEx, eq_ex_bmp());
    put(Sheet::PlEdit, pledit_bmp());
    put(Sheet::Gen, gen_bmp());
    Skin::from_parts(sheets, DEFAULT_VISCOLORS)
}

// ---------------------------------------------------------------------------
// Drawing helpers

fn bevel(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32, hi: u32, lo: u32) {
    b.hline(x, y, w, hi);
    b.vline(x, y, h, hi);
    b.hline(x, y + h - 1, w, lo);
    b.vline(x + w - 1, y, h, lo);
}

fn panel(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32, fill: u32) {
    b.fill_rect(x, y, w, h, fill);
    bevel(b, x, y, w, h, BODY_HI, BODY_LO);
}

fn sunken(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32, fill: u32) {
    b.fill_rect(x, y, w, h, fill);
    bevel(b, x, y, w, h, BODY_LO, BODY_HI);
}

fn lerp(a: u32, b: u32, t: f32) -> u32 {
    let ch = |s: u32| -> [f32; 3] { [((s >> 16) & 255) as f32, ((s >> 8) & 255) as f32, (s & 255) as f32] };
    let (ca, cb) = (ch(a), ch(b));
    let m = |i: usize| (ca[i] + (cb[i] - ca[i]) * t).round().clamp(0.0, 255.0) as u8;
    rgb(m(0), m(1), m(2))
}

fn vgradient(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32, top: u32, bottom: u32) {
    for i in 0..h {
        let t = if h > 1 { i as f32 / (h - 1) as f32 } else { 0.0 };
        b.hline(x, y + i, w, lerp(top, bottom, t));
    }
}

/// Fine horizontal grain on the window body.
fn body_texture(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32) {
    for yy in 0..h {
        let c = if yy % 2 == 0 { BODY } else { lerp(BODY, BODY_LO, 0.12) };
        b.hline(x, y + yy, w, c);
    }
}

fn button_face(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32, pressed: bool) {
    if pressed {
        vgradient(b, x, y, w, h, FACE_LO, FACE);
        bevel(b, x, y, w, h, BODY_DARK, FACE);
    } else {
        vgradient(b, x, y, w, h, lerp(FACE, FACE_HI, 0.35), FACE_LO);
        bevel(b, x, y, w, h, FACE_HI, BODY_DARK);
    }
}

fn pattern(b: &mut Bitmap, x: i32, y: i32, rows: &[&str], c: u32) {
    for (j, row) in rows.iter().enumerate() {
        for (i, ch) in row.chars().enumerate() {
            if ch == '#' {
                b.set(x + i as i32, y + j as i32, c);
            }
        }
    }
}

fn glyph(c: char) -> Option<&'static [&'static str]> {
    Some(match c {
        'A' => &[".##.", "#..#", "####", "#..#", "#..#"],
        'B' => &["###.", "#..#", "###.", "#..#", "###."],
        'C' => &[".###", "#...", "#...", "#...", ".###"],
        'D' => &["###.", "#..#", "#..#", "#..#", "###."],
        'E' => &["####", "#...", "###.", "#...", "####"],
        'F' => &["####", "#...", "###.", "#...", "#..."],
        'G' => &[".###", "#...", "#.##", "#..#", ".###"],
        'H' => &["#..#", "#..#", "####", "#..#", "#..#"],
        'I' => &["###", ".#.", ".#.", ".#.", "###"],
        'J' => &["..##", "...#", "...#", "#..#", ".##."],
        'K' => &["#..#", "#.#.", "##..", "#.#.", "#..#"],
        'L' => &["#...", "#...", "#...", "#...", "####"],
        'M' => &["#..#", "####", "####", "#..#", "#..#"],
        'N' => &["#..#", "##.#", "#.##", "#..#", "#..#"],
        'O' => &[".##.", "#..#", "#..#", "#..#", ".##."],
        'P' => &["###.", "#..#", "###.", "#...", "#..."],
        'Q' => &[".##.", "#..#", "#..#", "#.#.", ".#.#"],
        'R' => &["###.", "#..#", "###.", "#.#.", "#..#"],
        'S' => &[".###", "#...", ".##.", "...#", "###."],
        'T' => &["###", ".#.", ".#.", ".#.", ".#."],
        'U' => &["#..#", "#..#", "#..#", "#..#", ".##."],
        'V' => &["#.#", "#.#", "#.#", "#.#", ".#."],
        'W' => &["#..#", "#..#", "####", "####", "#..#"],
        'X' => &["#..#", "#..#", ".##.", "#..#", "#..#"],
        'Y' => &["#.#", "#.#", ".#.", ".#.", ".#."],
        'Z' => &["####", "...#", ".##.", "#...", "####"],
        '0' => &[".#.", "#.#", "#.#", "#.#", ".#."],
        '1' => &[".#.", "##.", ".#.", ".#.", "###"],
        '2' => &["##.", "..#", ".#.", "#..", "###"],
        '3' => &["##.", "..#", ".#.", "..#", "##."],
        '4' => &["#.#", "#.#", "###", "..#", "..#"],
        '5' => &["###", "#..", "##.", "..#", "##."],
        '6' => &[".##", "#..", "###", "#.#", "###"],
        '7' => &["###", "..#", ".#.", ".#.", ".#."],
        '8' => &["###", "#.#", "###", "#.#", "###"],
        '9' => &["###", "#.#", "###", "..#", "##."],
        '\u{2026}' => &["....", "....", "....", "....", "#.#."],
        '.' => &[".", ".", ".", ".", "#"],
        ':' => &[".", "#", ".", "#", "."],
        '(' => &[".#", "#.", "#.", "#.", ".#"],
        ')' => &["#.", ".#", ".#", ".#", "#."],
        '-' => &["...", "...", "###", "...", "..."],
        '\'' => &["#", "#", ".", ".", "."],
        '!' => &["#", "#", "#", ".", "#"],
        '_' => &["....", "....", "....", "....", "####"],
        '+' => &["...", ".#.", "###", ".#.", "..."],
        '\\' => &["#..", "#..", ".#.", "..#", "..#"],
        '/' => &["..#", "..#", ".#.", "#..", "#.."],
        '[' => &["##", "#.", "#.", "#.", "##"],
        ']' => &["##", ".#", ".#", ".#", "##"],
        '^' => &[".#.", "#.#", "...", "...", "..."],
        '&' => &[".#..", "#.#.", ".#..", "#.#.", ".#.#"],
        '%' => &["#..#", "..#.", ".#..", "#...", "#..#"],
        ',' => &["..", "..", "..", ".#", "#."],
        '=' => &["...", "###", "...", "###", "..."],
        '$' => &[".###", "##..", ".##.", "..##", "###."],
        '#' => &[".#.#", "####", ".#.#", "####", ".#.#"],
        '"' => &["#.#", "#.#", "...", "...", "..."],
        '@' => &[".##.", "#..#", "#.##", "#...", ".###"],
        'Å' => &[".##.", ".##.", "#..#", "####", "#..#", "#..#"],
        'Ä' => &["#..#", ".##.", "#..#", "####", "#..#", "#..#"],
        'Ö' => &["#..#", ".##.", "#..#", "#..#", "#..#", ".##."],
        '?' => &["##.", "..#", ".#.", "...", ".#."],
        '*' => &["...", "#.#", ".#.", "#.#", "..."],
        _ => return None,
    })
}

/// Draw text with the 5x6 cell font in any colour (used for labels).
fn label(b: &mut Bitmap, x: i32, y: i32, text: &str, c: u32) -> i32 {
    let mut cx = x;
    for ch in text.chars() {
        if ch == ' ' {
            cx += 3;
            continue;
        }
        if let Some(g) = glyph(ch.to_ascii_uppercase()) {
            pattern(b, cx, y, g, c);
            cx += g[0].len() as i32 + 1;
        } else {
            cx += 4;
        }
    }
    cx - x
}

fn label_width(text: &str) -> i32 {
    let mut probe = Bitmap::new(1, 1, TRANSPARENT);
    label(&mut probe, 0, 0, text, 0)
}

fn label_centered(b: &mut Bitmap, cx: i32, y: i32, text: &str, c: u32) {
    let w = label_width(text);
    label(b, cx - (w - 1) / 2, y, text, c);
}

// ---------------------------------------------------------------------------
// text.bmp: 31 columns x 3 rows of 5x6 cells.

fn text_bmp() -> Bitmap {
    let mut b = Bitmap::new(155, 18, LCD_BG);
    // Narrow glyphs are centred in their 5-pixel cell.
    let place = |b: &mut Bitmap, row: i32, col: i32, ch: char| {
        if let Some(g) = glyph(ch) {
            let w = g[0].len() as i32;
            let off = if w < 4 { (4 - w + 1) / 2 } else { 0 };
            pattern(b, col * 5 + off, row * 6, g, LCD_FG);
        }
    };
    for (i, ch) in ('A'..='Z').enumerate() {
        place(&mut b, 0, i as i32, ch);
    }
    place(&mut b, 0, 26, '"');
    place(&mut b, 0, 27, '@');
    for (i, ch) in ('0'..='9').enumerate() {
        place(&mut b, 1, i as i32, ch);
    }
    let row1 = ['\u{2026}', '.', ':', '(', ')', '-', '\'', '!', '_', '+', '\\', '/', '[', ']', '^', '&', '%', ',', '=', '$', '#'];
    for (i, ch) in row1.iter().enumerate() {
        place(&mut b, 1, 10 + i as i32, *ch);
    }
    for (i, ch) in ['Å', 'Ö', 'Ä', '?', '*'].iter().enumerate() {
        place(&mut b, 2, i as i32, *ch);
    }
    b
}

// ---------------------------------------------------------------------------
// numbers.bmp: ten 9x13 seven-segment digits. The minus sign is read from the
// middle bar of "2" (x=20,y=6) and its blank counterpart from "1" (x=9,y=6).

fn numbers_bmp() -> Bitmap {
    let mut b = Bitmap::new(99, 13, LCD_BG);
    // Segments: a top, b upper-right, c lower-right, d bottom, e lower-left,
    // f upper-left, g middle.
    const SEGS: [[bool; 7]; 10] = [
        [true, true, true, true, true, true, false],
        [false, true, true, false, false, false, false],
        [true, true, false, true, true, false, true],
        [true, true, true, true, false, false, true],
        [false, true, true, false, false, true, true],
        [true, false, true, true, false, true, true],
        [true, false, true, true, true, true, true],
        [true, true, true, false, false, false, false],
        [true, true, true, true, true, true, true],
        [true, true, true, true, false, true, true],
    ];
    for (d, segs) in SEGS.iter().enumerate() {
        let ox = d as i32 * 9;
        let on = |b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32| b.fill_rect(ox + x, y, w, h, LCD_FG);
        if segs[0] {
            on(&mut b, 2, 0, 5, 2);
        }
        if segs[1] {
            on(&mut b, 6, 1, 2, 5);
        }
        if segs[2] {
            on(&mut b, 6, 7, 2, 5);
        }
        if segs[3] {
            on(&mut b, 2, 11, 5, 2);
        }
        if segs[4] {
            on(&mut b, 1, 7, 2, 5);
        }
        if segs[5] {
            on(&mut b, 1, 1, 2, 5);
        }
        if segs[6] {
            on(&mut b, 2, 5, 5, 2);
        }
        // Soften the outer corners.
        for (x, y) in [(2, 0), (6, 0), (2, 12), (6, 12)] {
            if b.get(ox + x, y) == LCD_FG {
                b.set(ox + x, y, LCD_DIM);
            }
        }
    }
    // Keep the "no minus" slot blank and make the minus slot a clean bar.
    b.fill_rect(9, 6, 5, 1, LCD_BG);
    b.fill_rect(20, 6, 5, 1, LCD_FG);
    b
}

// ---------------------------------------------------------------------------
// main.bmp

fn main_bmp() -> Bitmap {
    let mut b = Bitmap::new(275, 116, BODY);
    body_texture(&mut b, 0, 0, 275, 116);
    bevel(&mut b, 0, 0, 275, 116, BODY_HI, BODY_DARK);
    bevel(&mut b, 1, 1, 273, 114, lerp(BODY_HI, BODY, 0.5), BODY_LO);

    // Left display (clutter bar, status, time and visualisation).
    sunken(&mut b, 8, 19, 96, 44, LCD_BG);
    b.fill_rect(9, 20, 94, 42, LCD_BG);
    // Hairline separating the time row from the visualisation.
    b.hline(23, 40, 78, rgb(0, 40, 0));

    // Song title box.
    sunken(&mut b, 108, 21, 159, 12, LCD_BG);
    // Bitrate and sample-rate boxes.
    sunken(&mut b, 109, 41, 19, 10, LCD_BG);
    label(&mut b, 130, 43, "KBPS", LABEL);
    sunken(&mut b, 154, 41, 14, 10, LCD_BG);
    label(&mut b, 170, 43, "KHZ", LABEL);

    // Volume / balance troughs (art comes from volume.bmp and balance.bmp).
    sunken(&mut b, 106, 56, 70, 15, BODY_LO);
    sunken(&mut b, 176, 56, 40, 15, BODY_LO);

    // Seek bar groove.
    sunken(&mut b, 15, 71, 250, 12, BODY_LO);

    // Transport area recess.
    b.hline(10, 86, 255, BODY_LO);
    b.hline(10, 87, 255, lerp(BODY_HI, BODY, 0.6));

    // Emblem in the corner (opens the about box).
    sunken(&mut b, 252, 90, 15, 17, BODY_DARK);
    emblem(&mut b, 253, 91);
    b
}

/// The built-in skin's emblem: a stylised "Fe" in a ring.
fn emblem(b: &mut Bitmap, x: i32, y: i32) {
    let ring = [
        "....#####....",
        "..##.....##..",
        ".#.........#.",
        ".#.........#.",
        "#...........#",
        "#...........#",
        "#...........#",
        "#...........#",
        "#...........#",
        ".#.........#.",
        ".#.........#.",
        "..##.....##..",
        "....#####....",
    ];
    pattern(b, x, y + 1, &ring, ACCENT);
    let fe = ["###...", "#.....", "##..#.", "#..#.#", "#..##.", "#..#..", "....##"];
    pattern(b, x + 4, y + 4, &fe, TITLE_ON);
}

// ---------------------------------------------------------------------------
// titlebar.bmp (344 x 87)

fn title_strip(b: &mut Bitmap, y: i32, active: bool, text: &str) {
    let (top, bottom) = if active { (rgb(70, 74, 120), rgb(36, 38, 70)) } else { (rgb(64, 66, 82), rgb(40, 41, 52)) };
    vgradient(b, 27, y, 275, 14, top, bottom);
    bevel(b, 27, y, 275, 14, if active { rgb(120, 126, 190) } else { BODY_HI }, BODY_DARK);
    let fg = if active { TITLE_ON } else { TITLE_OFF };
    let w = label_width(text);
    let tx = 27 + (275 - w) / 2;
    // Decorative ridges either side of the title.
    let ridge = if active { rgb(130, 136, 200) } else { rgb(96, 98, 116) };
    let ridge_lo = if active { rgb(24, 26, 52) } else { rgb(30, 30, 40) };
    for (x0, x1) in [(27 + 18, tx - 5), (tx + w + 4, 27 + 240)] {
        for yy in [4, 7, 10] {
            b.hline(x0, y + yy - 1, x1 - x0, ridge);
            b.hline(x0, y + yy, x1 - x0, ridge_lo);
        }
    }
    label(b, tx, y + 4, text, fg);
    // Built-in buttons drawn into the strip: menu at 6, minimise 244, shade 254, close 264.
    title_button(b, 27 + 6, y + 3, 'm', false, active);
    title_button(b, 27 + 244, y + 3, '_', false, active);
    title_button(b, 27 + 254, y + 3, 's', false, active);
    title_button(b, 27 + 264, y + 3, 'x', false, active);
}

fn title_button(b: &mut Bitmap, x: i32, y: i32, kind: char, pressed: bool, active: bool) {
    let face = if active { rgb(96, 100, 150) } else { rgb(80, 82, 100) };
    b.fill_rect(x, y, 9, 9, if pressed { lerp(face, BODY_DARK, 0.4) } else { face });
    if pressed {
        bevel(b, x, y, 9, 9, BODY_DARK, face);
    } else {
        bevel(b, x, y, 9, 9, if active { rgb(170, 176, 230) } else { BODY_HI }, BODY_DARK);
    }
    let c = if active { TITLE_ON } else { TITLE_OFF };
    let o = if pressed { 1 } else { 0 };
    let icon: &[&str] = match kind {
        'm' => &["#####", ".....", "#####", ".....", "#####"],
        '_' => &[".....", ".....", ".....", ".....", "#####"],
        's' => &["#####", "#...#", "#####", ".....", "....."],
        'u' => &[".....", ".....", "#####", "#...#", "#####"],
        'x' => &["#...#", ".#.#.", "..#..", ".#.#.", "#...#"],
        _ => &[],
    };
    pattern(b, x + 2 + o, y + 2 + o, icon, c);
}

fn titlebar_bmp() -> Bitmap {
    let mut b = Bitmap::new(344, 87, TRANSPARENT);
    title_strip(&mut b, 0, true, "FERRAMP");
    title_strip(&mut b, 15, false, "FERRAMP");
    // Window-shade backgrounds.
    shade_strip(&mut b, 29, true);
    shade_strip(&mut b, 42, false);
    // Alternate title bars (shown in the hidden easter-egg mode).
    title_strip(&mut b, 57, true, "IT REALLY FERRAMPS");
    title_strip(&mut b, 72, false, "IT REALLY FERRAMPS");

    title_button(&mut b, 0, 0, 'm', false, true);
    title_button(&mut b, 0, 9, 'm', true, true);
    title_button(&mut b, 9, 0, '_', false, true);
    title_button(&mut b, 9, 9, '_', true, true);
    title_button(&mut b, 18, 0, 'x', false, true);
    title_button(&mut b, 18, 9, 'x', true, true);
    title_button(&mut b, 0, 18, 's', false, true);
    title_button(&mut b, 9, 18, 's', true, true);
    title_button(&mut b, 0, 27, 'u', false, true);
    title_button(&mut b, 9, 27, 'u', true, true);

    // Shade-mode seek bar: groove and three thumb variants.
    b.fill_rect(0, 36, 17, 7, BODY_LO);
    bevel(&mut b, 0, 36, 17, 7, BODY_DARK, BODY_HI);
    for (x, c) in [(17, rgb(200, 90, 60)), (20, ACCENT), (23, rgb(120, 200, 80))] {
        b.fill_rect(x, 36, 3, 7, c);
        b.vline(x, 36, 7, lerp(c, rgb(255, 255, 255), 0.4));
        b.vline(x + 2, 36, 7, lerp(c, BODY_DARK, 0.5));
    }

    // Clutter bar: background, disabled background, selected letters.
    clutter_bar(&mut b, 304, 0, false);
    clutter_bar(&mut b, 312, 0, true);
    let letters = [('O', 304, 47, 8), ('A', 312, 55, 7), ('I', 320, 62, 7), ('D', 328, 69, 8), ('V', 336, 77, 7)];
    for (ch, x, y, h) in letters {
        b.fill_rect(x, y, 8, h, rgb(10, 60, 10));
        clutter_letter(&mut b, x, y, h, ch, LCD_FG);
    }
    b
}

fn shade_strip(b: &mut Bitmap, y: i32, active: bool) {
    let (top, bottom) = if active { (rgb(70, 74, 120), rgb(36, 38, 70)) } else { (rgb(64, 66, 82), rgb(40, 41, 52)) };
    vgradient(b, 27, y, 275, 14, top, bottom);
    bevel(b, 27, y, 275, 14, if active { rgb(120, 126, 190) } else { BODY_HI }, BODY_DARK);
    // Mini time display recess (127..152) and visualiser recess (79..117).
    sunken(b, 27 + 125, y + 2, 29, 10, LCD_BG);
    sunken(b, 27 + 78, y + 3, 40, 8, LCD_BG);
    // Mini transport buttons drawn into the strip.
    let glyphs: [(i32, &[&str]); 6] = [
        (169, &["#..#", "#.##", "####", "#.##", "#..#"]),
        (177, &["#...", "##..", "###.", "##..", "#..."]),
        (187, &["#.#.", "#.#.", "#.#.", "#.#.", "#.#."]),
        (196, &["###.", "###.", "###.", "###.", "....."]),
        (205, &["#..#", "##.#", "####", "##.#", "#..#"]),
        (216, &["..#..", ".###.", "#####", ".....", "#####"]),
    ];
    let c = if active { TITLE_ON } else { TITLE_OFF };
    for (x, g) in glyphs {
        pattern(b, 27 + x, y + 4, g, c);
    }
    // Colon of the mini time display (characters at 127+7, 12 | 20, 25).
    b.set(27 + 127 + 18, y + 5, LCD_FG);
    b.set(27 + 127 + 18, y + 8, LCD_FG);
    // Seek bar groove in shade mode at x=226.
    b.fill_rect(27 + 226, y + 4, 17, 7, BODY_LO);
    title_button(b, 27 + 6, y + 3, 'm', false, active);
    title_button(b, 27 + 244, y + 3, '_', false, active);
    title_button(b, 27 + 254, y + 3, 'u', false, active);
    title_button(b, 27 + 264, y + 3, 'x', false, active);
}

fn clutter_bar(b: &mut Bitmap, x: i32, y: i32, disabled: bool) {
    b.fill_rect(x, y, 8, 43, BODY_DARK);
    let c = if disabled { rgb(40, 60, 40) } else { LCD_DIM };
    let cells = [('O', 3, 8), ('A', 11, 7), ('I', 18, 7), ('D', 25, 8), ('V', 33, 7)];
    for (ch, cy, h) in cells {
        clutter_letter(b, x, y + cy, h, ch, c);
    }
}

fn clutter_letter(b: &mut Bitmap, x: i32, y: i32, h: i32, ch: char, c: u32) {
    if let Some(g) = glyph(ch) {
        let w = g[0].len() as i32;
        pattern(b, x + (8 - w) / 2, y + (h - 5) / 2, g, c);
    }
}

// ---------------------------------------------------------------------------
// cbuttons.bmp (136 x 36)

fn cbuttons_bmp() -> Bitmap {
    let mut b = Bitmap::new(136, 36, TRANSPARENT);
    let icons: [(i32, i32, &[&str]); 5] = [
        (0, 23, &["#...#", "#..##", "#.###", "#####", "#.###", "#..##", "#...#"]),
        (23, 23, &["#....", "##...", "###..", "####.", "###..", "##...", "#...."]),
        (46, 23, &["##.##", "##.##", "##.##", "##.##", "##.##", "##.##", "##.##"]),
        (69, 23, &["#####", "#####", "#####", "#####", "#####", "#####", "....."]),
        (92, 22, &["#...#", "##..#", "###.#", "#####", "###.#", "##..#", "#...#"]),
    ];
    for (x, w, icon) in icons {
        for (row, pressed) in [(0, false), (18, true)] {
            button_face(&mut b, x, row, w, 18, pressed);
            let o = if pressed { 1 } else { 0 };
            let iw = icon[0].len() as i32;
            pattern(&mut b, x + (w - iw) / 2 + o, row + 5 + o, icon, if pressed { ACCENT } else { LABEL });
        }
    }
    let eject = ["...#...", "..###..", ".#####.", "#######", ".......", "#######"];
    for (row, pressed) in [(0, false), (16, true)] {
        button_face(&mut b, 114, row, 22, 16, pressed);
        let o = if pressed { 1 } else { 0 };
        pattern(&mut b, 114 + 7 + o, row + 5 + o, &eject, if pressed { ACCENT } else { LABEL });
    }
    b
}

// ---------------------------------------------------------------------------
// playpaus.bmp (42 x 9)

fn playpaus_bmp() -> Bitmap {
    let mut b = Bitmap::new(42, 9, LCD_BG);
    pattern(&mut b, 3, 1, &["#....", "###..", "#####", "###..", "#...."], LCD_FG);
    b.fill_rect(1, 1, 1, 7, LCD_FG);
    pattern(&mut b, 11, 2, &["##.##", "##.##", "##.##", "##.##", "##.##"], ACCENT);
    pattern(&mut b, 20, 2, &["#####", "#####", "#####", "#####", "#####"], rgb(220, 60, 40));
    // Working / not-working indicators (3px wide, at 36 and 39).
    b.fill_rect(36, 0, 3, 9, LCD_BG);
    b.fill_rect(39, 0, 3, 9, LCD_BG);
    b.fill_rect(40, 2, 1, 5, LCD_FG);
    b
}

// ---------------------------------------------------------------------------
// monoster.bmp (58 x 24)

fn monoster_bmp() -> Bitmap {
    let mut b = Bitmap::new(58, 24, BODY);
    for (row, lit) in [(0, true), (12, false)] {
        let c = if lit { LCD_FG } else { rgb(70, 80, 90) };
        b.fill_rect(0, row, 29, 12, BODY);
        b.fill_rect(29, row, 29, 12, BODY);
        label(&mut b, 1, row + 3, "STEREO", c);
        label(&mut b, 29 + 3, row + 3, "MONO", c);
    }
    b
}

// ---------------------------------------------------------------------------
// posbar.bmp (307 x 10)

fn posbar_bmp() -> Bitmap {
    let mut b = Bitmap::new(307, 10, TRANSPARENT);
    b.fill_rect(0, 0, 248, 10, BODY_LO);
    bevel(&mut b, 0, 0, 248, 10, BODY_DARK, BODY_HI);
    for x in (6..242).step_by(10) {
        b.vline(x, 3, 4, lerp(BODY_LO, BODY_HI, 0.35));
    }
    for (x, pressed) in [(248, false), (278, true)] {
        button_face(&mut b, x, 0, 29, 10, pressed);
        let c = if pressed { ACCENT } else { LABEL };
        for gx in [x + 11, x + 14, x + 17] {
            b.vline(gx, 3, 4, c);
        }
    }
    b
}

// ---------------------------------------------------------------------------
// volume.bmp (68 x 433) and balance.bmp (47 x 433)

fn level_track(b: &mut Bitmap, x: i32, y: i32, w: i32, fill_frac: f32, color: u32) {
    // 68x13 (or 38x13) trough with a coloured fill and tick marks.
    b.fill_rect(x, y, w, 13, BODY_DARK);
    bevel(b, x, y, w, 13, BODY_DARK, BODY_HI);
    let inner = w - 4;
    let filled = (inner as f32 * fill_frac).round() as i32;
    vgradient(b, x + 2, y + 4, filled, 5, lerp(color, rgb(255, 255, 255), 0.25), lerp(color, BODY_DARK, 0.3));
    for t in (x + 3..x + w - 2).step_by(4) {
        b.set(t, y + 10, lerp(BODY_DARK, BODY_HI, 0.5));
    }
}

fn volume_bmp() -> Bitmap {
    let mut b = Bitmap::new(68, 433, TRANSPARENT);
    for i in 0..28 {
        let t = i as f32 / 27.0;
        let color = if t < 0.5 { lerp(rgb(30, 200, 40), rgb(230, 210, 30), t * 2.0) } else { lerp(rgb(230, 210, 30), rgb(230, 50, 30), (t - 0.5) * 2.0) };
        level_track(&mut b, 0, i * 15, 68, t, color);
    }
    slider_thumb(&mut b, 15, 422, false);
    slider_thumb(&mut b, 0, 422, true);
    b
}

fn balance_bmp() -> Bitmap {
    let mut b = Bitmap::new(47, 433, TRANSPARENT);
    for i in 0..28 {
        let t = i as f32 / 27.0;
        let color = lerp(rgb(30, 200, 40), rgb(230, 50, 30), t);
        b.fill_rect(9, i * 15, 38, 13, BODY_DARK);
        bevel(&mut b, 9, i * 15, 38, 13, BODY_DARK, BODY_HI);
        b.fill_rect(9 + 18, i * 15 + 3, 2, 7, lerp(BODY_DARK, BODY_HI, 0.5));
        let half = (16.0 * t).round() as i32;
        vgradient(&mut b, 9 + 19 - half, i * 15 + 4, half, 5, color, lerp(color, BODY_DARK, 0.3));
        vgradient(&mut b, 9 + 19, i * 15 + 4, half, 5, color, lerp(color, BODY_DARK, 0.3));
    }
    slider_thumb(&mut b, 15, 422, false);
    slider_thumb(&mut b, 0, 422, true);
    b
}

fn slider_thumb(b: &mut Bitmap, x: i32, y: i32, pressed: bool) {
    button_face(b, x, y, 14, 11, pressed);
    let c = if pressed { ACCENT } else { LABEL };
    for gx in [x + 5, x + 7, x + 9] {
        b.vline(gx, y + 3, 5, c);
    }
}

// ---------------------------------------------------------------------------
// shufrep.bmp (92 x 85)

fn shufrep_bmp() -> Bitmap {
    let mut b = Bitmap::new(92, 85, TRANSPARENT);
    // Rows: 0 off, 15 off+pressed, 30 on, 45 on+pressed.
    for (row, on, pressed) in [(0, false, false), (15, false, true), (30, true, false), (45, true, true)] {
        toggle(&mut b, 0, row, 28, 15, "REP", on, pressed);
        toggle(&mut b, 28, row, 47, 15, "SHUFFLE", on, pressed);
    }
    // EQ / PL buttons: x 0 eq, 23 pl; y 61 off, 73 on; +46 pressed.
    for (y, on) in [(61, false), (73, true)] {
        for (x, pressed) in [(0, false), (46, true)] {
            toggle(&mut b, x, y, 23, 12, "EQ", on, pressed);
            toggle(&mut b, x + 23, y, 23, 12, "PL", on, pressed);
        }
    }
    b
}

#[allow(clippy::too_many_arguments)]
fn toggle(b: &mut Bitmap, x: i32, y: i32, w: i32, h: i32, text: &str, on: bool, pressed: bool) {
    button_face(b, x, y, w, h, pressed);
    let o = if pressed { 1 } else { 0 };
    // Indicator LED at the left.
    let led = if on { LCD_FG } else { rgb(30, 50, 30) };
    b.fill_rect(x + 3 + o, y + h / 2 - 1 + o, 2, 3, led);
    let tw = label_width(text);
    let tx = x + 6 + (w - 6 - tw) / 2 + o;
    label(b, tx, y + (h - 5) / 2 + o, text, if on { TITLE_ON } else { LABEL });
}

// ---------------------------------------------------------------------------
// eqmain.bmp (275 x 315)

const EQ_BANDS: [&str; 10] = ["60", "170", "310", "600", "1K", "3K", "6K", "12K", "14K", "16K"];

fn eqmain_bmp() -> Bitmap {
    let mut b = Bitmap::new(275, 315, TRANSPARENT);
    // Window background.
    body_texture(&mut b, 0, 0, 275, 116);
    bevel(&mut b, 0, 0, 275, 116, BODY_HI, BODY_DARK);
    bevel(&mut b, 1, 1, 273, 114, lerp(BODY_HI, BODY, 0.5), BODY_LO);
    // Graph recess and preset area.
    sunken(&mut b, 85, 16, 115, 21, BODY_DARK);
    // Slider recesses.
    sunken(&mut b, 19, 36, 18, 67, BODY_LO);
    sunken(&mut b, 76, 36, 180, 67, BODY_LO);
    // dB scale labels.
    label(&mut b, 45, 37, "+12DB", LABEL);
    label(&mut b, 45, 65, "+0DB", LABEL);
    label(&mut b, 45, 96, "-12DB", LABEL);
    label_centered(&mut b, 28, 105, "PREAMP", LABEL);
    for (i, name) in EQ_BANDS.iter().enumerate() {
        label_centered(&mut b, 78 + 18 * i as i32 + 7, 105, name, LABEL);
    }

    // Title bars: selected at 134, normal at 149.
    eq_title(&mut b, 134, true);
    eq_title(&mut b, 149, false);
    // Close button (normal + pressed) at 0,116 / 0,125.
    title_button(&mut b, 0, 116, 'x', false, true);
    title_button(&mut b, 0, 125, 'x', true, true);

    // ON / AUTO buttons at y=119.
    for (x_on, x_auto, on, pressed) in [(10, 36, false, false), (69, 95, true, false), (128, 154, false, true), (187, 213, true, true)] {
        toggle(&mut b, x_on, 119, 26, 12, "ON", on, pressed);
        toggle(&mut b, x_auto, 119, 32, 12, "AUTO", on, pressed);
    }

    // Band slider backgrounds: 28 frames of 14x63 at stride 15x65.
    for i in 0..28 {
        let fx = 13 + (i % 14) * 15;
        let fy = 164 + (i / 14) * 65;
        band_background(&mut b, fx, fy, i as f32 / 27.0);
    }
    // Thumbs.
    eq_thumb(&mut b, 0, 164, false);
    eq_thumb(&mut b, 0, 176, true);
    // Presets button.
    for (y, pressed) in [(164, false), (176, true)] {
        button_face(&mut b, 224, y, 44, 12, pressed);
        let o = if pressed { 1 } else { 0 };
        label(&mut b, 224 + 6 + o, y + 4 + o, "PRESETS", LABEL);
    }
    // Graph background, line colours and preamp line.
    b.fill_rect(0, 294, 113, 19, BODY_DARK);
    for y in [0, 9, 18] {
        b.hline(0, 294 + y, 113, rgb(50, 52, 70));
    }
    for x in (0..113).step_by(12) {
        b.vline(x + 2, 294, 19, rgb(36, 38, 52));
    }
    for i in 0..19 {
        let t = i as f32 / 18.0;
        let c = if t < 0.5 { lerp(rgb(255, 60, 40), rgb(250, 220, 40), t * 2.0) } else { lerp(rgb(250, 220, 40), rgb(40, 220, 60), (t - 0.5) * 2.0) };
        b.set(115, 294 + i, c);
    }
    for x in 0..113 {
        if x % 3 != 2 {
            b.set(x, 314, rgb(120, 124, 150));
        }
    }
    b
}

fn eq_title(b: &mut Bitmap, y: i32, active: bool) {
    let (top, bottom) = if active { (rgb(70, 74, 120), rgb(36, 38, 70)) } else { (rgb(64, 66, 82), rgb(40, 41, 52)) };
    vgradient(b, 0, y, 275, 14, top, bottom);
    bevel(b, 0, y, 275, 14, if active { rgb(120, 126, 190) } else { BODY_HI }, BODY_DARK);
    let fg = if active { TITLE_ON } else { TITLE_OFF };
    label_centered(b, 137, y + 4, "EQUALIZER", fg);
    title_button(b, 254, y + 3, 's', false, active);
    title_button(b, 264, y + 3, 'x', false, active);
}

fn band_background(b: &mut Bitmap, x: i32, y: i32, level: f32) {
    b.fill_rect(x, y, 14, 63, BODY_LO);
    // Centre groove.
    b.fill_rect(x + 5, y + 2, 4, 59, BODY_DARK);
    b.vline(x + 9, y + 2, 59, lerp(BODY_LO, BODY_HI, 0.4));
    // Coloured fill from the centre line towards the level.
    let color = if level >= 0.5 { lerp(rgb(250, 220, 40), rgb(255, 70, 40), (level - 0.5) * 2.0) } else { lerp(rgb(250, 220, 40), rgb(40, 210, 70), (0.5 - level) * 2.0) };
    let center = y + 31;
    let reach = ((level - 0.5).abs() * 2.0 * 28.0).round() as i32;
    if level >= 0.5 {
        b.fill_rect(x + 6, center - reach, 2, reach + 1, color);
    } else {
        b.fill_rect(x + 6, center, 2, reach + 1, color);
    }
    b.set(x + 4, center, LABEL);
    b.set(x + 10, center, LABEL);
}

fn eq_thumb(b: &mut Bitmap, x: i32, y: i32, pressed: bool) {
    button_face(b, x, y, 11, 11, pressed);
    let c = if pressed { ACCENT } else { LABEL };
    b.hline(x + 3, y + 4, 5, c);
    b.hline(x + 3, y + 6, 5, c);
}

// ---------------------------------------------------------------------------
// eq_ex.bmp (275 x 56)

fn eq_ex_bmp() -> Bitmap {
    let mut b = Bitmap::new(275, 56, TRANSPARENT);
    for (y, active) in [(0, true), (15, false)] {
        let (top, bottom) = if active { (rgb(70, 74, 120), rgb(36, 38, 70)) } else { (rgb(64, 66, 82), rgb(40, 41, 52)) };
        vgradient(&mut b, 0, y, 275, 14, top, bottom);
        bevel(&mut b, 0, y, 275, 14, if active { rgb(120, 126, 190) } else { BODY_HI }, BODY_DARK);
        let fg = if active { TITLE_ON } else { TITLE_OFF };
        label(&mut b, 6, y + 4, "EQ", fg);
        // Volume (61..158) and balance (164..207) grooves.
        b.fill_rect(61, y + 4, 97, 6, BODY_DARK);
        b.fill_rect(164, y + 4, 43, 6, BODY_DARK);
        title_button(&mut b, 254, y + 3, 'u', false, active);
        title_button(&mut b, 264, y + 3, 'x', false, active);
    }
    for (x, c) in [(1, rgb(40, 200, 60)), (4, rgb(230, 210, 40)), (7, rgb(230, 70, 40))] {
        b.fill_rect(x, 30, 3, 7, c);
    }
    for (x, c) in [(11, rgb(230, 70, 40)), (14, rgb(40, 200, 60)), (17, rgb(230, 70, 40))] {
        b.fill_rect(x, 30, 3, 7, c);
    }
    title_button(&mut b, 1, 38, 's', true, true);
    title_button(&mut b, 1, 47, 'u', true, true);
    title_button(&mut b, 11, 38, 'x', false, true);
    title_button(&mut b, 11, 47, 'x', true, true);
    b
}

// ---------------------------------------------------------------------------
// pledit.bmp (280 x 186)

fn pledit_bmp() -> Bitmap {
    let mut b = Bitmap::new(280, 186, TRANSPARENT);
    for (y, active) in [(0, true), (21, false)] {
        let (top, bottom) = if active { (rgb(70, 74, 120), rgb(36, 38, 70)) } else { (rgb(64, 66, 82), rgb(40, 41, 52)) };
        let edge = if active { rgb(120, 126, 190) } else { BODY_HI };
        for (x, w) in [(0, 25), (26, 100), (127, 25), (153, 25)] {
            vgradient(&mut b, x, y, w, 20, top, bottom);
            b.hline(x, y, w, edge);
            b.hline(x, y + 19, w, BODY_DARK);
            // Lower 6 rows blend into the frame.
            b.fill_rect(x, y + 14, w, 6, BODY);
            b.hline(x, y + 14, w, BODY_LO);
        }
        b.vline(0, y, 20, edge);
        b.vline(177, y, 20, BODY_DARK);
        let fg = if active { TITLE_ON } else { TITLE_OFF };
        label_centered(&mut b, 26 + 50, y + 4, "PLAYLIST", fg);
        // Ridges on the tile.
        let ridge = if active { rgb(130, 136, 200) } else { rgb(96, 98, 116) };
        for yy in [4, 7, 10] {
            b.hline(127, y + yy - 1, 25, ridge);
            b.hline(127, y + yy, 25, BODY_DARK);
            b.hline(4, y + yy - 1, 19, ridge);
            b.hline(4, y + yy, 19, BODY_DARK);
            b.hline(153, y + yy - 1, 3, ridge);
            b.hline(153, y + yy, 3, BODY_DARK);
        }
        title_button(&mut b, 153 + 4, y + 3, 's', false, active);
        title_button(&mut b, 153 + 14, y + 3, 'x', false, active);
    }
    // Left and right frame tiles.
    b.fill_rect(0, 42, 12, 29, BODY);
    b.vline(0, 42, 29, BODY_HI);
    b.vline(11, 42, 29, BODY_DARK);
    b.fill_rect(31, 42, 20, 29, BODY);
    b.vline(31, 42, 29, BODY_HI);
    b.fill_rect(31 + 5, 42, 8, 29, BODY_DARK);
    b.vline(31 + 19, 42, 29, BODY_DARK);
    // Scroll handles.
    for (x, pressed) in [(52, false), (61, true)] {
        button_face(&mut b, x, 53, 8, 18, pressed);
        for yy in [59, 62, 65] {
            b.hline(x + 2, yy, 4, if pressed { ACCENT } else { LABEL });
        }
    }
    title_button(&mut b, 52, 42, 'x', true, true);
    title_button(&mut b, 62, 42, 's', true, true);
    title_button(&mut b, 150, 42, 'u', true, true);
    // Shade backgrounds.
    for (y, active) in [(42, true), (57, false)] {
        let (top, bottom) = if active { (rgb(70, 74, 120), rgb(36, 38, 70)) } else { (rgb(64, 66, 82), rgb(40, 41, 52)) };
        let edge = if active { rgb(120, 126, 190) } else { BODY_HI };
        vgradient(&mut b, 72, y, 25, 14, top, bottom);
        b.hline(72, y, 25, edge);
        b.vline(72, y, 14, edge);
        b.hline(72, y + 13, 25, BODY_DARK);
        vgradient(&mut b, 99, y, 50, 14, top, bottom);
        b.hline(99, y, 50, edge);
        b.hline(99, y + 13, 50, BODY_DARK);
        b.vline(148, y, 14, BODY_DARK);
        // Resize grip, shade and close buttons at 99+20.., 99+29, 99+39.
        title_button(&mut b, 99 + 29, y + 3, 'u', false, active);
        title_button(&mut b, 99 + 39, y + 3, 'x', false, active);
        for i in 0..3 {
            b.set(99 + 22 + i * 2, y + 9, LABEL);
        }
    }
    // Shade background tile (the middle one) only needs the gradient: done above at x=72.

    // Bottom tile, bottom-left corner and bottom-right corner.
    pl_bottom(&mut b, 179, 0, 25);
    pl_bottom(&mut b, 0, 72, 125);
    pl_bottom(&mut b, 126, 72, 150);
    // Bottom-left: menu button faces ADD / REM / SEL / MISC at x 14, 43, 72, 101 (y = 38-12-18 = 8).
    for (i, name) in ["ADD", "REM", "SEL", "MISC"].iter().enumerate() {
        let x = 14 + 29 * i as i32;
        button_face(&mut b, x, 72 + 8, 22, 18, false);
        label_centered(&mut b, x + 11, 72 + 8 + 7, name, LABEL);
    }
    // Bottom-right: info panel, mini transport, LIST button and resize grip.
    let br = 126;
    sunken(&mut b, br + 5, 72 + 8, 92, 13, LCD_BG);
    sunken(&mut b, br + 64, 72 + 22, 30, 9, LCD_BG);
    // Colon of the mini time (time text at br+66, 23 from the bottom bar top).
    b.set(br + 66 + 18, 72 + 24, LCD_FG);
    b.set(br + 66 + 18, 72 + 27, LCD_FG);
    let mini: [(i32, &[&str]); 6] = [
        (3, &["#.#", "###", "#.#"]),
        (13, &["#..", "##.", "#.."]),
        (23, &["#.#", "#.#", "#.#"]),
        (33, &["###", "###", "###"]),
        (43, &["#.#", "###", "#.#"]),
        (53, &[".#.", "###", "###"]),
    ];
    for (x, g) in mini {
        button_face(&mut b, br + x, 72 + 22, 10, 10, false);
        pattern(&mut b, br + x + 3, 72 + 25, g, LABEL);
    }
    button_face(&mut b, br + 150 - 22 - 22, 72 + 8, 22, 18, false);
    label_centered(&mut b, br + 150 - 44 + 11, 72 + 8 + 7, "LIST", LABEL);
    // Scroll arrows (above the grip) and resize grip.
    pattern(&mut b, br + 150 - 15, 72 + 2, &["..#..", ".###."], LABEL);
    pattern(&mut b, br + 150 - 15, 72 + 8, &[".###.", "..#.."], LABEL);
    for i in 0..5 {
        for j in 0..=i {
            b.set(br + 150 - 3 - j * 2 + i - i, 72 + 36 - (i - j) * 2, LABEL);
        }
    }
    // Visualiser background.
    pl_bottom(&mut b, 205, 0, 75);
    sunken(&mut b, 205 + 1, 11, 74, 20, LCD_BG);

    // Pop-up menu buttons (22x18, normal + highlighted) and their side bars.
    let menu_cols: [(i32, &[&str]); 5] = [
        (0, &["URL", "DIR", "FILE"]),
        (54, &["ALL", "CROP", "SEL", "MISC"]),
        (104, &["INV", "NONE", "ALL"]),
        (154, &["SORT", "INFO", "OPTS"]),
        (204, &["NEW", "SAVE", "LOAD"]),
    ];
    for (x, names) in menu_cols {
        for (j, name) in names.iter().enumerate() {
            let y = 111 + 19 * j as i32;
            for (dx, hot) in [(0, false), (23, true)] {
                button_face(&mut b, x + dx, y, 22, 18, hot);
                label_centered(&mut b, x + dx + 11, y + 7, name, if hot { ACCENT } else { LABEL });
            }
        }
    }
    for (x, h) in [(48, 54), (100, 72), (150, 54), (200, 54), (250, 54)] {
        b.fill_rect(x, 111, 3, h, ACCENT);
        b.vline(x + 2, 111, h, lerp(ACCENT, BODY_DARK, 0.5));
    }
    b
}

fn pl_bottom(b: &mut Bitmap, x: i32, y: i32, w: i32) {
    body_texture(b, x, y, w, 38);
    b.hline(x, y, w, BODY_LO);
    b.hline(x, y + 37, w, BODY_DARK);
}

// ---------------------------------------------------------------------------
// gen.bmp: frame for generic windows (194 x 109).

fn gen_bmp() -> Bitmap {
    let mut b = Bitmap::new(194, 109, TRANSPARENT);
    for (y, active) in [(0, true), (21, false)] {
        let (top, bottom) = if active { (rgb(70, 74, 120), rgb(36, 38, 70)) } else { (rgb(64, 66, 82), rgb(40, 41, 52)) };
        for i in 0..6 {
            let x = i * 26;
            vgradient(&mut b, x, y, 25, 20, top, bottom);
            b.fill_rect(x, y + 14, 25, 6, BODY);
        }
    }
    b.fill_rect(0, 42, 125, 14, BODY);
    b.fill_rect(0, 57, 125, 14, BODY);
    b.fill_rect(127, 72, 25, 14, BODY);
    b.fill_rect(127, 42, 11, 29, BODY);
    b.fill_rect(158, 42, 11, 24, BODY);
    b.fill_rect(139, 42, 8, 29, BODY);
    b.fill_rect(170, 42, 8, 24, BODY);
    title_button(&mut b, 148, 42, 'x', true, true);
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skin::sprites;

    #[test]
    fn builtin_sheets_have_expected_sizes() {
        let skin = build();
        let size = |s: Sheet| skin.sheet(s).map(|b| (b.w, b.h)).unwrap();
        assert_eq!(size(Sheet::Main), (275, 116));
        assert_eq!(size(Sheet::Text), (155, 18));
        assert_eq!(size(Sheet::Numbers), (99, 13));
        assert_eq!(size(Sheet::EqMain), (275, 315));
        assert_eq!(size(Sheet::PlEdit), (280, 186));
    }

    #[test]
    fn minus_sign_lives_in_digit_two() {
        let skin = build();
        let n = skin.sheet(Sheet::Numbers).unwrap();
        let m = sprites::MINUS_SIGN;
        for x in 0..m.w {
            assert_eq!(n.get(m.x + x, m.y), LCD_FG);
        }
        let nm = sprites::NO_MINUS_SIGN;
        for x in 0..nm.w {
            assert_eq!(n.get(nm.x + x, nm.y), LCD_BG);
        }
    }

    #[test]
    fn every_font_cell_glyph_fits() {
        for c in ('A'..='Z').chain('0'..='9') {
            let g = glyph(c).unwrap();
            assert!(g.len() <= 6 && g.iter().all(|r| r.len() <= 5), "{c}");
        }
    }
}
