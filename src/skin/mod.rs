//! Classic skin support: loading `.wsz`/`.zip` archives or skin folders,
//! with per-file fallback to the built-in skin the way the original player
//! falls back to its base skin.

pub mod default_skin;
pub mod sprites;

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::gfx::{Bitmap, rgb};
use sprites::{Sheet, Sprite};

#[derive(Clone, Debug)]
pub struct PlaylistStyle {
    pub normal: u32,
    pub current: u32,
    pub normal_bg: u32,
    pub selected_bg: u32,
    pub font: String,
}

impl Default for PlaylistStyle {
    fn default() -> Self {
        PlaylistStyle {
            normal: rgb(0x00, 0xFF, 0x00),
            current: rgb(0xFF, 0xFF, 0xFF),
            normal_bg: rgb(0x00, 0x00, 0x00),
            selected_bg: rgb(0x00, 0x00, 0xFF),
            font: "Arial".to_string(),
        }
    }
}

/// Polygon outline for a window, as listed in region.txt.
pub type Polygon = Vec<(i32, i32)>;

#[derive(Clone, Debug, Default)]
pub struct Regions {
    pub normal: Vec<Polygon>,
    pub windowshade: Vec<Polygon>,
    pub equalizer: Vec<Polygon>,
    pub equalizer_ws: Vec<Polygon>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CursorKind {
    Close,
    EqClose,
    EqNormal,
    EqSlid,
    EqTitle,
    MainMenu,
    MMenu,
    Min,
    Normal,
    PClose,
    PNormal,
    PosBar,
    PSize,
    PTBar,
    PVScroll,
    PWinBut,
    PWSNorm,
    PWSSize,
    SongName,
    TitleBar,
    VolBal,
    WinBut,
    WSNormal,
    WSPosBar,
}

impl CursorKind {
    pub const ALL: [CursorKind; 24] = [
        CursorKind::Close,
        CursorKind::EqClose,
        CursorKind::EqNormal,
        CursorKind::EqSlid,
        CursorKind::EqTitle,
        CursorKind::MainMenu,
        CursorKind::MMenu,
        CursorKind::Min,
        CursorKind::Normal,
        CursorKind::PClose,
        CursorKind::PNormal,
        CursorKind::PosBar,
        CursorKind::PSize,
        CursorKind::PTBar,
        CursorKind::PVScroll,
        CursorKind::PWinBut,
        CursorKind::PWSNorm,
        CursorKind::PWSSize,
        CursorKind::SongName,
        CursorKind::TitleBar,
        CursorKind::VolBal,
        CursorKind::WinBut,
        CursorKind::WSNormal,
        CursorKind::WSPosBar,
    ];

    pub fn file_stem(self) -> &'static str {
        match self {
            CursorKind::Close => "close",
            CursorKind::EqClose => "eqclose",
            CursorKind::EqNormal => "eqnormal",
            CursorKind::EqSlid => "eqslid",
            CursorKind::EqTitle => "eqtitle",
            CursorKind::MainMenu => "mainmenu",
            CursorKind::MMenu => "mmenu",
            CursorKind::Min => "min",
            CursorKind::Normal => "normal",
            CursorKind::PClose => "pclose",
            CursorKind::PNormal => "pnormal",
            CursorKind::PosBar => "posbar",
            CursorKind::PSize => "psize",
            CursorKind::PTBar => "ptbar",
            CursorKind::PVScroll => "pvscroll",
            CursorKind::PWinBut => "pwinbut",
            CursorKind::PWSNorm => "pwsnorm",
            CursorKind::PWSSize => "pwssize",
            CursorKind::SongName => "songname",
            CursorKind::TitleBar => "titlebar",
            CursorKind::VolBal => "volbal",
            CursorKind::WinBut => "winbut",
            CursorKind::WSNormal => "wsnormal",
            CursorKind::WSPosBar => "wsposbar",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CursorImage {
    pub image: Bitmap,
    pub hot_x: u16,
    pub hot_y: u16,
}

pub struct Skin {
    pub path: Option<PathBuf>,
    sheets: Vec<Option<Bitmap>>,
    /// Which sheets came from the skin itself (rather than the fallback).
    own: Vec<bool>,
    pub viscolors: [u32; 24],
    pub playlist: PlaylistStyle,
    pub regions: Regions,
    pub cursors: HashMap<CursorKind, CursorImage>,
}

impl Skin {
    pub fn sheet(&self, sheet: Sheet) -> Option<&Bitmap> {
        self.sheets[sheet.index()].as_ref()
    }

    pub fn has_own(&self, sheet: Sheet) -> bool {
        self.own[sheet.index()]
    }

    /// Numbers come from nums_ex.bmp when the skin itself ships that file.
    pub fn uses_nums_ex(&self) -> bool {
        self.has_own(Sheet::NumsEx)
            || (!self.has_own(Sheet::Numbers) && self.sheet(Sheet::NumsEx).is_some())
    }

    pub fn draw(&self, dst: &mut Bitmap, sp: Sprite, x: i32, y: i32) {
        if let Some(src) = self.sheet(sp.sheet) {
            dst.blit(src, sp.x, sp.y, sp.w, sp.h, x, y);
        }
    }

    /// Draw part of a sprite: `ox`,`oy` offset within the sprite.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_part(
        &self,
        dst: &mut Bitmap,
        sp: Sprite,
        ox: i32,
        oy: i32,
        w: i32,
        h: i32,
        x: i32,
        y: i32,
    ) {
        if let Some(src) = self.sheet(sp.sheet) {
            dst.blit(
                src,
                sp.x + ox,
                sp.y + oy,
                w.min(sp.w - ox),
                h.min(sp.h - oy),
                x,
                y,
            );
        }
    }

    /// Pixel inside a sprite (used for 1px colour strips such as the EQ graph colours).
    pub fn sprite_pixel(&self, sp: Sprite, x: i32, y: i32) -> u32 {
        self.sheet(sp.sheet)
            .map(|b| b.get(sp.x + x, sp.y + y))
            .unwrap_or(0)
    }

    pub fn builtin() -> Skin {
        default_skin::build()
    }

    /// Load a skin from a `.wsz`/`.zip` archive or a directory, falling back
    /// to `base` for anything the skin does not provide.
    pub fn load(path: &Path, base: &Skin) -> Result<Skin, String> {
        let files = SkinFiles::open(path)?;
        let mut sheets = Vec::new();
        let mut own = Vec::new();
        for sheet in Sheet::ALL {
            match files.image(sheet.file_stem()) {
                Some(img) => {
                    sheets.push(Some(img));
                    own.push(true);
                }
                None => {
                    sheets.push(base.sheet(sheet).cloned());
                    own.push(false);
                }
            }
        }
        let viscolors = files
            .text("viscolor", "txt")
            .map(|t| parse_viscolors(&t, &base.viscolors))
            .unwrap_or(base.viscolors);
        let playlist = files
            .text("pledit", "txt")
            .map(|t| parse_pledit(&t))
            .unwrap_or_default();
        let regions = files
            .text("region", "txt")
            .map(|t| parse_regions(&t))
            .unwrap_or_default();
        let mut cursors = HashMap::new();
        for kind in CursorKind::ALL {
            if let Some(bytes) = files.bytes(kind.file_stem(), "cur")
                && let Some(c) = parse_cursor(&bytes)
            {
                cursors.insert(kind, c);
            }
        }
        Ok(Skin {
            path: Some(path.to_path_buf()),
            sheets,
            own,
            viscolors,
            playlist,
            regions,
            cursors,
        })
    }

    pub(crate) fn from_parts(sheets: Vec<Option<Bitmap>>, viscolors: [u32; 24]) -> Skin {
        let own = sheets.iter().map(|s| s.is_some()).collect();
        Skin {
            path: None,
            sheets,
            own,
            viscolors,
            playlist: PlaylistStyle::default(),
            regions: Regions::default(),
            cursors: HashMap::new(),
        }
    }
}

/// Lookup of files inside a skin archive or folder, case-insensitively by
/// base name, the last match winning (like extraction onto a Windows disk).
struct SkinFiles {
    entries: HashMap<String, Vec<u8>>,
}

impl SkinFiles {
    fn open(path: &Path) -> Result<SkinFiles, String> {
        let mut entries = HashMap::new();
        if path.is_dir() {
            collect_dir(path, &mut entries, 0);
        } else {
            let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut zip =
                zip::ZipArchive::new(file).map_err(|e| format!("{}: {e}", path.display()))?;
            let mut total: u64 = 0;
            for i in 0..zip.len().min(MAX_ENTRIES) {
                let Ok(f) = zip.by_index(i) else { continue };
                if f.is_dir() {
                    continue;
                }
                let Ok(name) = f.name() else { continue };
                let name = name.replace('\\', "/");
                let base = name
                    .rsplit('/')
                    .next()
                    .unwrap_or(&name)
                    .to_ascii_lowercase();
                if !is_skin_file(&base) || f.size() > MAX_FILE_BYTES {
                    continue;
                }
                let mut buf = Vec::new();
                // Never trust the declared size: cap the decompressed read too.
                if f.take(MAX_FILE_BYTES).read_to_end(&mut buf).is_ok() {
                    total += buf.len() as u64;
                    if total > MAX_TOTAL_BYTES {
                        return Err(format!("{}: skin archive too large", path.display()));
                    }
                    entries.insert(base, buf);
                }
            }
        }
        if entries.is_empty() {
            return Err(format!("{}: no skin files found", path.display()));
        }
        Ok(SkinFiles { entries })
    }

    fn bytes(&self, stem: &str, ext: &str) -> Option<Vec<u8>> {
        self.entries.get(&format!("{stem}.{ext}")).cloned()
    }

    fn text(&self, stem: &str, ext: &str) -> Option<String> {
        self.bytes(stem, ext).map(|b| decode_text(&b))
    }

    fn image(&self, stem: &str) -> Option<Bitmap> {
        for ext in ["bmp", "png"] {
            if let Some(bytes) = self.entries.get(&format!("{stem}.{ext}"))
                && let Some(b) = decode_image(bytes)
            {
                return Some(b);
            }
        }
        None
    }
}

const MAX_ENTRIES: usize = 4096;
const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;

/// Only files a classic skin can contain are read.
fn is_skin_file(base: &str) -> bool {
    let Some((stem, ext)) = base.rsplit_once('.') else {
        return false;
    };
    match ext {
        "bmp" | "png" => Sheet::ALL.iter().any(|s| s.file_stem() == stem),
        "txt" => matches!(stem, "viscolor" | "pledit" | "region"),
        "cur" | "ani" => CursorKind::ALL.iter().any(|c| c.file_stem() == stem),
        _ => false,
    }
}

fn collect_dir(dir: &Path, out: &mut HashMap<String, Vec<u8>>, depth: u32) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            if depth < 2 {
                collect_dir(&p, out, depth + 1);
            }
        } else if let Some(n) = p.file_name() {
            let base = n.to_string_lossy().to_ascii_lowercase();
            let small = std::fs::metadata(&p)
                .map(|m| m.len() <= MAX_FILE_BYTES)
                .unwrap_or(false);
            if is_skin_file(&base)
                && small
                && let Ok(b) = std::fs::read(&p)
            {
                out.insert(base, b);
            }
        }
    }
}

/// Skins are written with Windows code pages; decode as UTF-8 when valid,
/// otherwise as Windows-1252.
pub fn decode_text(b: &[u8]) -> String {
    match std::str::from_utf8(b) {
        Ok(s) => s.trim_start_matches('\u{feff}').to_string(),
        Err(_) => b.iter().map(|&c| cp1252(c)).collect(),
    }
}

fn cp1252(c: u8) -> char {
    const HIGH: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž',
        '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}',
        'ž', 'Ÿ',
    ];
    match c {
        0x80..=0x9F => HIGH[(c - 0x80) as usize],
        _ => c as char,
    }
}

pub fn decode_image(bytes: &[u8]) -> Option<Bitmap> {
    let fmt = if bytes.starts_with(b"BM") {
        image::ImageFormat::Bmp
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        image::ImageFormat::Png
    } else {
        return None;
    };
    let img = image::load_from_memory_with_format(bytes, fmt)
        .ok()?
        .to_rgba8();
    let mut b = Bitmap::from_rgba8(img.width(), img.height(), img.as_raw());
    // Skin bitmaps are opaque; some 32-bit BMPs carry a zero alpha channel.
    for p in &mut b.px {
        *p |= 0xFF00_0000;
    }
    Some(b)
}

pub fn parse_viscolors(text: &str, fallback: &[u32; 24]) -> [u32; 24] {
    let mut out = *fallback;
    let mut i = 0;
    for line in text.lines() {
        if i >= 24 {
            break;
        }
        let nums: Vec<u32> = line
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .take(3)
            .filter_map(|s| s.parse().ok())
            .collect();
        if nums.len() == 3 {
            out[i] = rgb(
                nums[0].min(255) as u8,
                nums[1].min(255) as u8,
                nums[2].min(255) as u8,
            );
            i += 1;
        }
    }
    out
}

/// Minimal INI reader: section and key names are lower-cased.
pub fn parse_ini(text: &str) -> HashMap<String, HashMap<String, String>> {
    let mut out: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with("//") {
            continue;
        }
        if line.starts_with('[') {
            if let Some(end) = line.find(']') {
                section = line[1..end].trim().to_ascii_lowercase();
            }
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            out.entry(section.clone())
                .or_default()
                .insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    out
}

pub fn parse_color(s: &str) -> Option<u32> {
    let s = s.trim().trim_start_matches('#');
    let hex: String = s
        .chars()
        .take_while(|c| c.is_ascii_hexdigit())
        .take(6)
        .collect();
    if hex.len() < 6 {
        return None;
    }
    u32::from_str_radix(&hex, 16).ok().map(|v| v | 0xFF00_0000)
}

pub fn parse_pledit(text: &str) -> PlaylistStyle {
    let ini = parse_ini(text);
    let mut style = PlaylistStyle::default();
    if let Some(t) = ini.get("text") {
        let set = |key: &str, slot: &mut u32| {
            if let Some(c) = t.get(key).and_then(|v| parse_color(v)) {
                *slot = c;
            }
        };
        set("normal", &mut style.normal);
        set("current", &mut style.current);
        set("normalbg", &mut style.normal_bg);
        set("selectedbg", &mut style.selected_bg);
        if let Some(f) = t.get("font")
            && !f.is_empty()
        {
            style.font = f.clone();
        }
    }
    style
}

pub fn parse_regions(text: &str) -> Regions {
    let ini = parse_ini(text);
    let get = |name: &str| -> Vec<Polygon> {
        let Some(sec) = ini.get(name) else {
            return vec![];
        };
        let (Some(np), Some(pl)) = (sec.get("numpoints"), sec.get("pointlist")) else {
            return vec![];
        };
        let counts: Vec<usize> = np
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let nums: Vec<i32> = pl
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let points: Vec<(i32, i32)> = nums
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| (c[0], c[1]))
            .collect();
        let mut idx = 0;
        let mut polys = Vec::new();
        for n in counts {
            if idx + n > points.len() {
                break;
            }
            if n >= 3 {
                polys.push(points[idx..idx + n].to_vec());
            }
            idx += n;
        }
        polys
    };
    Regions {
        normal: get("normal"),
        windowshade: get("windowshade"),
        equalizer: get("equalizer"),
        equalizer_ws: get("equalizerws"),
    }
}

/// Rasterise region polygons into a mask (true = visible), even-odd per
/// polygon, unioned. Matches the pixel-centre sampling of a GDI polygon region.
pub fn region_mask(polys: &[Polygon], w: i32, h: i32) -> Vec<bool> {
    let mut mask = vec![false; (w * h) as usize];
    for poly in polys {
        for y in 0..h {
            let py = y as f64 + 0.5;
            let mut xs: Vec<f64> = Vec::new();
            for i in 0..poly.len() {
                let (x0, y0) = poly[i];
                let (x1, y1) = poly[(i + 1) % poly.len()];
                let (x0, y0, x1, y1) = (x0 as f64, y0 as f64, x1 as f64, y1 as f64);
                if (y0 <= py && y1 > py) || (y1 <= py && y0 > py) {
                    xs.push(x0 + (py - y0) * (x1 - x0) / (y1 - y0));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for pair in xs.as_chunks::<2>().0 {
                let start = (pair[0] - 0.5).ceil().max(0.0) as i32;
                let end = ((pair[1] - 0.5).ceil() as i32).min(w);
                for x in start..end {
                    mask[(y * w + x) as usize] = true;
                }
            }
        }
    }
    mask
}

/// Parse a Windows .cur file (an ICO container with hotspots). Animated
/// cursors (RIFF/ANI) use their first frame.
pub fn parse_cursor(bytes: &[u8]) -> Option<CursorImage> {
    if bytes.starts_with(b"RIFF") {
        return parse_ani_first_frame(bytes);
    }
    if bytes.len() < 22 || bytes[2] != 2 {
        return None;
    }
    let hot_x = u16::from_le_bytes([bytes[10], bytes[11]]);
    let hot_y = u16::from_le_bytes([bytes[12], bytes[13]]);
    // The ico decoder accepts the same container with type 1.
    let mut ico = bytes.to_vec();
    ico[2] = 1;
    // Directory entry "planes"/"bpp" fields hold the hotspot in a .cur; reset them.
    ico[10] = 1;
    ico[11] = 0;
    ico[12] = 0;
    ico[13] = 0;
    let img = image::load_from_memory_with_format(&ico, image::ImageFormat::Ico)
        .ok()?
        .to_rgba8();
    Some(CursorImage {
        image: Bitmap::from_rgba8(img.width(), img.height(), img.as_raw()),
        hot_x,
        hot_y,
    })
}

fn parse_ani_first_frame(bytes: &[u8]) -> Option<CursorImage> {
    // Walk RIFF chunks looking for the first "icon" chunk inside LIST/fram.
    let mut i = 12;
    while i + 8 <= bytes.len() {
        let id = &bytes[i..i + 4];
        let len = u32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?) as usize;
        if id == b"LIST" {
            i += 12; // descend into the list
            continue;
        }
        if id == b"icon" {
            return parse_cursor(bytes.get(i + 8..i + 8 + len)?);
        }
        i += 8 + len + (len & 1);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viscolor_parsing() {
        let t = "0,0,0, // background\n24,33,41 // dots\n255, 0,0\n";
        let base = [rgb(9, 9, 9); 24];
        let v = parse_viscolors(t, &base);
        assert_eq!(v[0], rgb(0, 0, 0));
        assert_eq!(v[1], rgb(24, 33, 41));
        assert_eq!(v[2], rgb(255, 0, 0));
        assert_eq!(v[3], rgb(9, 9, 9));
    }

    #[test]
    fn pledit_parsing() {
        let t = "[Text]\nNormal=#00FF00\nCurrent=FFFFFF\nNormalBG=#000000\nSelectedBG=#0000FF00\nFont=Tahoma\n";
        let s = parse_pledit(t);
        assert_eq!(s.current, rgb(255, 255, 255));
        assert_eq!(s.selected_bg, rgb(0, 0, 255));
        assert_eq!(s.font, "Tahoma");
    }

    #[test]
    fn region_parsing_and_mask() {
        let t = "[Normal]\nNumPoints=4\nPointList=0,0, 4,0, 4,4, 0,4\n[WindowShade]\nNumPoints=2\nPointList=0,0,1,1";
        let r = parse_regions(t);
        assert_eq!(r.normal.len(), 1);
        assert!(r.windowshade.is_empty());
        let m = region_mask(&r.normal, 6, 6);
        assert!(m[0]);
        assert!(m[3 * 6 + 3]);
        assert!(!m[4 * 6 + 4]);
        assert!(!m[5]);
    }

    #[test]
    fn skin_file_filter() {
        assert!(is_skin_file("main.bmp"));
        assert!(is_skin_file("pledit.txt"));
        assert!(is_skin_file("eqslid.cur"));
        assert!(!is_skin_file("readme.txt"));
        assert!(!is_skin_file("huge.bin"));
    }

    #[test]
    fn colors() {
        assert_eq!(parse_color("#ff0000"), Some(rgb(255, 0, 0)));
        assert_eq!(parse_color("00ff00ff"), Some(rgb(0, 255, 0)));
        assert_eq!(parse_color("#fff"), None);
    }
}
