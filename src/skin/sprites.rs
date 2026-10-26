//! Sprite coordinates inside the classic skin bitmaps.
//!
//! The skin file format is a public interchange format: these offsets are the
//! layout every classic skin is drawn against. The table follows the layout
//! documented by the Webamp project (MIT licence, see NOTICE).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sheet {
    Main,
    CButtons,
    TitleBar,
    Numbers,
    NumsEx,
    Text,
    PlayPaus,
    MonoSter,
    PosBar,
    Volume,
    Balance,
    ShufRep,
    EqMain,
    EqEx,
    PlEdit,
    Gen,
}

impl Sheet {
    pub const ALL: [Sheet; 16] = [
        Sheet::Main,
        Sheet::CButtons,
        Sheet::TitleBar,
        Sheet::Numbers,
        Sheet::NumsEx,
        Sheet::Text,
        Sheet::PlayPaus,
        Sheet::MonoSter,
        Sheet::PosBar,
        Sheet::Volume,
        Sheet::Balance,
        Sheet::ShufRep,
        Sheet::EqMain,
        Sheet::EqEx,
        Sheet::PlEdit,
        Sheet::Gen,
    ];

    /// Base file name inside a skin archive (without extension).
    pub fn file_stem(self) -> &'static str {
        match self {
            Sheet::Main => "main",
            Sheet::CButtons => "cbuttons",
            Sheet::TitleBar => "titlebar",
            Sheet::Numbers => "numbers",
            Sheet::NumsEx => "nums_ex",
            Sheet::Text => "text",
            Sheet::PlayPaus => "playpaus",
            Sheet::MonoSter => "monoster",
            Sheet::PosBar => "posbar",
            Sheet::Volume => "volume",
            Sheet::Balance => "balance",
            Sheet::ShufRep => "shufrep",
            Sheet::EqMain => "eqmain",
            Sheet::EqEx => "eq_ex",
            Sheet::PlEdit => "pledit",
            Sheet::Gen => "gen",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub sheet: Sheet,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

const fn s(sheet: Sheet, x: i32, y: i32, w: i32, h: i32) -> Sprite {
    Sprite { sheet, x, y, w, h }
}

use Sheet::*;

// main.bmp
pub const MAIN_WINDOW_BACKGROUND: Sprite = s(Main, 0, 0, 275, 116);

// cbuttons.bmp
pub const MAIN_PREVIOUS_BUTTON: Sprite = s(CButtons, 0, 0, 23, 18);
pub const MAIN_PREVIOUS_BUTTON_ACTIVE: Sprite = s(CButtons, 0, 18, 23, 18);
pub const MAIN_PLAY_BUTTON: Sprite = s(CButtons, 23, 0, 23, 18);
pub const MAIN_PLAY_BUTTON_ACTIVE: Sprite = s(CButtons, 23, 18, 23, 18);
pub const MAIN_PAUSE_BUTTON: Sprite = s(CButtons, 46, 0, 23, 18);
pub const MAIN_PAUSE_BUTTON_ACTIVE: Sprite = s(CButtons, 46, 18, 23, 18);
pub const MAIN_STOP_BUTTON: Sprite = s(CButtons, 69, 0, 23, 18);
pub const MAIN_STOP_BUTTON_ACTIVE: Sprite = s(CButtons, 69, 18, 23, 18);
pub const MAIN_NEXT_BUTTON: Sprite = s(CButtons, 92, 0, 22, 18);
pub const MAIN_NEXT_BUTTON_ACTIVE: Sprite = s(CButtons, 92, 18, 22, 18);
pub const MAIN_EJECT_BUTTON: Sprite = s(CButtons, 114, 0, 22, 16);
pub const MAIN_EJECT_BUTTON_ACTIVE: Sprite = s(CButtons, 114, 16, 22, 16);

// monoster.bmp
pub const MAIN_STEREO: Sprite = s(MonoSter, 0, 12, 29, 12);
pub const MAIN_STEREO_SELECTED: Sprite = s(MonoSter, 0, 0, 29, 12);
pub const MAIN_MONO: Sprite = s(MonoSter, 29, 12, 27, 12);
pub const MAIN_MONO_SELECTED: Sprite = s(MonoSter, 29, 0, 27, 12);

// numbers.bmp / nums_ex.bmp
pub const NO_MINUS_SIGN: Sprite = s(Numbers, 9, 6, 5, 1);
pub const MINUS_SIGN: Sprite = s(Numbers, 20, 6, 5, 1);
pub const NO_MINUS_SIGN_EX: Sprite = s(NumsEx, 90, 0, 9, 13);
pub const MINUS_SIGN_EX: Sprite = s(NumsEx, 99, 0, 9, 13);

pub const fn digit(n: i32, ex: bool) -> Sprite {
    s(if ex { NumsEx } else { Numbers }, n * 9, 0, 9, 13)
}

// playpaus.bmp
pub const MAIN_PLAYING_INDICATOR: Sprite = s(PlayPaus, 0, 0, 9, 9);
pub const MAIN_PAUSED_INDICATOR: Sprite = s(PlayPaus, 9, 0, 9, 9);
pub const MAIN_STOPPED_INDICATOR: Sprite = s(PlayPaus, 18, 0, 9, 9);
pub const MAIN_NOT_WORKING_INDICATOR: Sprite = s(PlayPaus, 36, 0, 3, 9);
pub const MAIN_WORKING_INDICATOR: Sprite = s(PlayPaus, 39, 0, 3, 9);

// posbar.bmp
pub const MAIN_POSITION_SLIDER_BACKGROUND: Sprite = s(PosBar, 0, 0, 248, 10);
pub const MAIN_POSITION_SLIDER_THUMB: Sprite = s(PosBar, 248, 0, 29, 10);
pub const MAIN_POSITION_SLIDER_THUMB_SELECTED: Sprite = s(PosBar, 278, 0, 29, 10);

// shufrep.bmp
pub const MAIN_SHUFFLE_BUTTON: Sprite = s(ShufRep, 28, 0, 47, 15);
pub const MAIN_SHUFFLE_BUTTON_DEPRESSED: Sprite = s(ShufRep, 28, 15, 47, 15);
pub const MAIN_SHUFFLE_BUTTON_SELECTED: Sprite = s(ShufRep, 28, 30, 47, 15);
pub const MAIN_SHUFFLE_BUTTON_SELECTED_DEPRESSED: Sprite = s(ShufRep, 28, 45, 47, 15);
pub const MAIN_REPEAT_BUTTON: Sprite = s(ShufRep, 0, 0, 28, 15);
pub const MAIN_REPEAT_BUTTON_DEPRESSED: Sprite = s(ShufRep, 0, 15, 28, 15);
pub const MAIN_REPEAT_BUTTON_SELECTED: Sprite = s(ShufRep, 0, 30, 28, 15);
pub const MAIN_REPEAT_BUTTON_SELECTED_DEPRESSED: Sprite = s(ShufRep, 0, 45, 28, 15);
pub const MAIN_EQ_BUTTON: Sprite = s(ShufRep, 0, 61, 23, 12);
pub const MAIN_EQ_BUTTON_SELECTED: Sprite = s(ShufRep, 0, 73, 23, 12);
pub const MAIN_EQ_BUTTON_DEPRESSED: Sprite = s(ShufRep, 46, 61, 23, 12);
pub const MAIN_EQ_BUTTON_DEPRESSED_SELECTED: Sprite = s(ShufRep, 46, 73, 23, 12);
pub const MAIN_PLAYLIST_BUTTON: Sprite = s(ShufRep, 23, 61, 23, 12);
pub const MAIN_PLAYLIST_BUTTON_SELECTED: Sprite = s(ShufRep, 23, 73, 23, 12);
pub const MAIN_PLAYLIST_BUTTON_DEPRESSED: Sprite = s(ShufRep, 69, 61, 23, 12);
pub const MAIN_PLAYLIST_BUTTON_DEPRESSED_SELECTED: Sprite = s(ShufRep, 69, 73, 23, 12);

// titlebar.bmp
pub const MAIN_TITLE_BAR: Sprite = s(TitleBar, 27, 15, 275, 14);
pub const MAIN_TITLE_BAR_SELECTED: Sprite = s(TitleBar, 27, 0, 275, 14);
pub const MAIN_EASTER_EGG_TITLE_BAR: Sprite = s(TitleBar, 27, 72, 275, 14);
pub const MAIN_EASTER_EGG_TITLE_BAR_SELECTED: Sprite = s(TitleBar, 27, 57, 275, 14);
pub const MAIN_OPTIONS_BUTTON: Sprite = s(TitleBar, 0, 0, 9, 9);
pub const MAIN_OPTIONS_BUTTON_DEPRESSED: Sprite = s(TitleBar, 0, 9, 9, 9);
pub const MAIN_MINIMIZE_BUTTON: Sprite = s(TitleBar, 9, 0, 9, 9);
pub const MAIN_MINIMIZE_BUTTON_DEPRESSED: Sprite = s(TitleBar, 9, 9, 9, 9);
pub const MAIN_SHADE_BUTTON: Sprite = s(TitleBar, 0, 18, 9, 9);
pub const MAIN_SHADE_BUTTON_DEPRESSED: Sprite = s(TitleBar, 9, 18, 9, 9);
pub const MAIN_CLOSE_BUTTON: Sprite = s(TitleBar, 18, 0, 9, 9);
pub const MAIN_CLOSE_BUTTON_DEPRESSED: Sprite = s(TitleBar, 18, 9, 9, 9);
pub const MAIN_CLUTTER_BAR_BACKGROUND: Sprite = s(TitleBar, 304, 0, 8, 43);
pub const MAIN_CLUTTER_BAR_BACKGROUND_DISABLED: Sprite = s(TitleBar, 312, 0, 8, 43);
pub const MAIN_CLUTTER_BAR_BUTTON_O_SELECTED: Sprite = s(TitleBar, 304, 47, 8, 8);
pub const MAIN_CLUTTER_BAR_BUTTON_A_SELECTED: Sprite = s(TitleBar, 312, 55, 8, 7);
pub const MAIN_CLUTTER_BAR_BUTTON_I_SELECTED: Sprite = s(TitleBar, 320, 62, 8, 7);
pub const MAIN_CLUTTER_BAR_BUTTON_D_SELECTED: Sprite = s(TitleBar, 328, 69, 8, 8);
pub const MAIN_CLUTTER_BAR_BUTTON_V_SELECTED: Sprite = s(TitleBar, 336, 77, 8, 7);
pub const MAIN_SHADE_BACKGROUND: Sprite = s(TitleBar, 27, 42, 275, 14);
pub const MAIN_SHADE_BACKGROUND_SELECTED: Sprite = s(TitleBar, 27, 29, 275, 14);
pub const MAIN_SHADE_BUTTON_SELECTED: Sprite = s(TitleBar, 0, 27, 9, 9);
pub const MAIN_SHADE_BUTTON_SELECTED_DEPRESSED: Sprite = s(TitleBar, 9, 27, 9, 9);
pub const MAIN_SHADE_POSITION_BACKGROUND: Sprite = s(TitleBar, 0, 36, 17, 7);
pub const MAIN_SHADE_POSITION_THUMB: Sprite = s(TitleBar, 20, 36, 3, 7);
pub const MAIN_SHADE_POSITION_THUMB_LEFT: Sprite = s(TitleBar, 17, 36, 3, 7);
pub const MAIN_SHADE_POSITION_THUMB_RIGHT: Sprite = s(TitleBar, 23, 36, 3, 7);

// volume.bmp / balance.bmp
pub const MAIN_VOLUME_BACKGROUND: Sprite = s(Volume, 0, 0, 68, 420);
pub const MAIN_VOLUME_THUMB: Sprite = s(Volume, 15, 422, 14, 11);
pub const MAIN_VOLUME_THUMB_SELECTED: Sprite = s(Volume, 0, 422, 14, 11);
pub const MAIN_BALANCE_BACKGROUND: Sprite = s(Balance, 9, 0, 38, 420);
pub const MAIN_BALANCE_THUMB: Sprite = s(Balance, 15, 422, 14, 11);
pub const MAIN_BALANCE_THUMB_ACTIVE: Sprite = s(Balance, 0, 422, 14, 11);

// eqmain.bmp
pub const EQ_WINDOW_BACKGROUND: Sprite = s(EqMain, 0, 0, 275, 116);
pub const EQ_TITLE_BAR: Sprite = s(EqMain, 0, 149, 275, 14);
pub const EQ_TITLE_BAR_SELECTED: Sprite = s(EqMain, 0, 134, 275, 14);
pub const EQ_SLIDER_BACKGROUND: Sprite = s(EqMain, 13, 164, 209, 129);
pub const EQ_SLIDER_THUMB: Sprite = s(EqMain, 0, 164, 11, 11);
pub const EQ_SLIDER_THUMB_SELECTED: Sprite = s(EqMain, 0, 176, 11, 11);
pub const EQ_CLOSE_BUTTON: Sprite = s(EqMain, 0, 116, 9, 9);
pub const EQ_CLOSE_BUTTON_ACTIVE: Sprite = s(EqMain, 0, 125, 9, 9);
pub const EQ_MAXIMIZE_BUTTON_ACTIVE_FALLBACK: Sprite = s(EqMain, 254, 152, 9, 9);
pub const EQ_ON_BUTTON: Sprite = s(EqMain, 10, 119, 26, 12);
pub const EQ_ON_BUTTON_DEPRESSED: Sprite = s(EqMain, 128, 119, 26, 12);
pub const EQ_ON_BUTTON_SELECTED: Sprite = s(EqMain, 69, 119, 26, 12);
pub const EQ_ON_BUTTON_SELECTED_DEPRESSED: Sprite = s(EqMain, 187, 119, 26, 12);
pub const EQ_AUTO_BUTTON: Sprite = s(EqMain, 36, 119, 32, 12);
pub const EQ_AUTO_BUTTON_DEPRESSED: Sprite = s(EqMain, 154, 119, 32, 12);
pub const EQ_AUTO_BUTTON_SELECTED: Sprite = s(EqMain, 95, 119, 32, 12);
pub const EQ_AUTO_BUTTON_SELECTED_DEPRESSED: Sprite = s(EqMain, 213, 119, 32, 12);
pub const EQ_GRAPH_BACKGROUND: Sprite = s(EqMain, 0, 294, 113, 19);
pub const EQ_GRAPH_LINE_COLORS: Sprite = s(EqMain, 115, 294, 1, 19);
pub const EQ_PRESETS_BUTTON: Sprite = s(EqMain, 224, 164, 44, 12);
pub const EQ_PRESETS_BUTTON_SELECTED: Sprite = s(EqMain, 224, 176, 44, 12);
pub const EQ_PREAMP_LINE: Sprite = s(EqMain, 0, 314, 113, 1);

// eq_ex.bmp
pub const EQ_SHADE_BACKGROUND_SELECTED: Sprite = s(EqEx, 0, 0, 275, 14);
pub const EQ_SHADE_BACKGROUND: Sprite = s(EqEx, 0, 15, 275, 14);
pub const EQ_SHADE_VOLUME_SLIDER_LEFT: Sprite = s(EqEx, 1, 30, 3, 7);
pub const EQ_SHADE_VOLUME_SLIDER_CENTER: Sprite = s(EqEx, 4, 30, 3, 7);
pub const EQ_SHADE_VOLUME_SLIDER_RIGHT: Sprite = s(EqEx, 7, 30, 3, 7);
pub const EQ_SHADE_BALANCE_SLIDER_LEFT: Sprite = s(EqEx, 11, 30, 3, 7);
pub const EQ_SHADE_BALANCE_SLIDER_CENTER: Sprite = s(EqEx, 14, 30, 3, 7);
pub const EQ_SHADE_BALANCE_SLIDER_RIGHT: Sprite = s(EqEx, 17, 30, 3, 7);
pub const EQ_MAXIMIZE_BUTTON_ACTIVE: Sprite = s(EqEx, 1, 38, 9, 9);
pub const EQ_MINIMIZE_BUTTON_ACTIVE: Sprite = s(EqEx, 1, 47, 9, 9);
pub const EQ_SHADE_CLOSE_BUTTON: Sprite = s(EqEx, 11, 38, 9, 9);
pub const EQ_SHADE_CLOSE_BUTTON_ACTIVE: Sprite = s(EqEx, 11, 47, 9, 9);

// pledit.bmp
pub const PLAYLIST_TOP_TILE: Sprite = s(PlEdit, 127, 21, 25, 20);
pub const PLAYLIST_TOP_LEFT_CORNER: Sprite = s(PlEdit, 0, 21, 25, 20);
pub const PLAYLIST_TITLE_BAR: Sprite = s(PlEdit, 26, 21, 100, 20);
pub const PLAYLIST_TOP_RIGHT_CORNER: Sprite = s(PlEdit, 153, 21, 25, 20);
pub const PLAYLIST_TOP_TILE_SELECTED: Sprite = s(PlEdit, 127, 0, 25, 20);
pub const PLAYLIST_TOP_LEFT_SELECTED: Sprite = s(PlEdit, 0, 0, 25, 20);
pub const PLAYLIST_TITLE_BAR_SELECTED: Sprite = s(PlEdit, 26, 0, 100, 20);
pub const PLAYLIST_TOP_RIGHT_CORNER_SELECTED: Sprite = s(PlEdit, 153, 0, 25, 20);
pub const PLAYLIST_LEFT_TILE: Sprite = s(PlEdit, 0, 42, 12, 29);
pub const PLAYLIST_RIGHT_TILE: Sprite = s(PlEdit, 31, 42, 20, 29);
pub const PLAYLIST_BOTTOM_TILE: Sprite = s(PlEdit, 179, 0, 25, 38);
pub const PLAYLIST_BOTTOM_LEFT_CORNER: Sprite = s(PlEdit, 0, 72, 125, 38);
pub const PLAYLIST_BOTTOM_RIGHT_CORNER: Sprite = s(PlEdit, 126, 72, 150, 38);
pub const PLAYLIST_VISUALIZER_BACKGROUND: Sprite = s(PlEdit, 205, 0, 75, 38);
pub const PLAYLIST_SHADE_BACKGROUND: Sprite = s(PlEdit, 72, 57, 25, 14);
pub const PLAYLIST_SHADE_BACKGROUND_LEFT: Sprite = s(PlEdit, 72, 42, 25, 14);
pub const PLAYLIST_SHADE_BACKGROUND_RIGHT: Sprite = s(PlEdit, 99, 57, 50, 14);
pub const PLAYLIST_SHADE_BACKGROUND_RIGHT_SELECTED: Sprite = s(PlEdit, 99, 42, 50, 14);
pub const PLAYLIST_SCROLL_HANDLE_SELECTED: Sprite = s(PlEdit, 61, 53, 8, 18);
pub const PLAYLIST_SCROLL_HANDLE: Sprite = s(PlEdit, 52, 53, 8, 18);
pub const PLAYLIST_ADD_URL: Sprite = s(PlEdit, 0, 111, 22, 18);
pub const PLAYLIST_ADD_URL_SELECTED: Sprite = s(PlEdit, 23, 111, 22, 18);
pub const PLAYLIST_ADD_DIR: Sprite = s(PlEdit, 0, 130, 22, 18);
pub const PLAYLIST_ADD_DIR_SELECTED: Sprite = s(PlEdit, 23, 130, 22, 18);
pub const PLAYLIST_ADD_FILE: Sprite = s(PlEdit, 0, 149, 22, 18);
pub const PLAYLIST_ADD_FILE_SELECTED: Sprite = s(PlEdit, 23, 149, 22, 18);
pub const PLAYLIST_REMOVE_ALL: Sprite = s(PlEdit, 54, 111, 22, 18);
pub const PLAYLIST_REMOVE_ALL_SELECTED: Sprite = s(PlEdit, 77, 111, 22, 18);
pub const PLAYLIST_CROP: Sprite = s(PlEdit, 54, 130, 22, 18);
pub const PLAYLIST_CROP_SELECTED: Sprite = s(PlEdit, 77, 130, 22, 18);
pub const PLAYLIST_REMOVE_SELECTED: Sprite = s(PlEdit, 54, 149, 22, 18);
pub const PLAYLIST_REMOVE_SELECTED_SELECTED: Sprite = s(PlEdit, 77, 149, 22, 18);
pub const PLAYLIST_REMOVE_MISC: Sprite = s(PlEdit, 54, 168, 22, 18);
pub const PLAYLIST_REMOVE_MISC_SELECTED: Sprite = s(PlEdit, 77, 168, 22, 18);
pub const PLAYLIST_INVERT_SELECTION: Sprite = s(PlEdit, 104, 111, 22, 18);
pub const PLAYLIST_INVERT_SELECTION_SELECTED: Sprite = s(PlEdit, 127, 111, 22, 18);
pub const PLAYLIST_SELECT_ZERO: Sprite = s(PlEdit, 104, 130, 22, 18);
pub const PLAYLIST_SELECT_ZERO_SELECTED: Sprite = s(PlEdit, 127, 130, 22, 18);
pub const PLAYLIST_SELECT_ALL: Sprite = s(PlEdit, 104, 149, 22, 18);
pub const PLAYLIST_SELECT_ALL_SELECTED: Sprite = s(PlEdit, 127, 149, 22, 18);
pub const PLAYLIST_SORT_LIST: Sprite = s(PlEdit, 154, 111, 22, 18);
pub const PLAYLIST_SORT_LIST_SELECTED: Sprite = s(PlEdit, 177, 111, 22, 18);
pub const PLAYLIST_FILE_INFO: Sprite = s(PlEdit, 154, 130, 22, 18);
pub const PLAYLIST_FILE_INFO_SELECTED: Sprite = s(PlEdit, 177, 130, 22, 18);
pub const PLAYLIST_MISC_OPTIONS: Sprite = s(PlEdit, 154, 149, 22, 18);
pub const PLAYLIST_MISC_OPTIONS_SELECTED: Sprite = s(PlEdit, 177, 149, 22, 18);
pub const PLAYLIST_NEW_LIST: Sprite = s(PlEdit, 204, 111, 22, 18);
pub const PLAYLIST_NEW_LIST_SELECTED: Sprite = s(PlEdit, 227, 111, 22, 18);
pub const PLAYLIST_SAVE_LIST: Sprite = s(PlEdit, 204, 130, 22, 18);
pub const PLAYLIST_SAVE_LIST_SELECTED: Sprite = s(PlEdit, 227, 130, 22, 18);
pub const PLAYLIST_LOAD_LIST: Sprite = s(PlEdit, 204, 149, 22, 18);
pub const PLAYLIST_LOAD_LIST_SELECTED: Sprite = s(PlEdit, 227, 149, 22, 18);
pub const PLAYLIST_ADD_MENU_BAR: Sprite = s(PlEdit, 48, 111, 3, 54);
pub const PLAYLIST_REMOVE_MENU_BAR: Sprite = s(PlEdit, 100, 111, 3, 72);
pub const PLAYLIST_SELECT_MENU_BAR: Sprite = s(PlEdit, 150, 111, 3, 54);
pub const PLAYLIST_MISC_MENU_BAR: Sprite = s(PlEdit, 200, 111, 3, 54);
pub const PLAYLIST_LIST_BAR: Sprite = s(PlEdit, 250, 111, 3, 54);
pub const PLAYLIST_CLOSE_SELECTED: Sprite = s(PlEdit, 52, 42, 9, 9);
pub const PLAYLIST_COLLAPSE_SELECTED: Sprite = s(PlEdit, 62, 42, 9, 9);
pub const PLAYLIST_EXPAND_SELECTED: Sprite = s(PlEdit, 150, 42, 9, 9);

// gen.bmp (generic window frame, used by our extra windows)
pub const GEN_TOP_LEFT_SELECTED: Sprite = s(Gen, 0, 0, 25, 20);
pub const GEN_TOP_LEFT_END_SELECTED: Sprite = s(Gen, 26, 0, 25, 20);
pub const GEN_TOP_CENTER_FILL_SELECTED: Sprite = s(Gen, 52, 0, 25, 20);
pub const GEN_TOP_RIGHT_END_SELECTED: Sprite = s(Gen, 78, 0, 25, 20);
pub const GEN_TOP_LEFT_RIGHT_FILL_SELECTED: Sprite = s(Gen, 104, 0, 25, 20);
pub const GEN_TOP_RIGHT_SELECTED: Sprite = s(Gen, 130, 0, 25, 20);
pub const GEN_TOP_LEFT: Sprite = s(Gen, 0, 21, 25, 20);
pub const GEN_TOP_LEFT_END: Sprite = s(Gen, 26, 21, 25, 20);
pub const GEN_TOP_CENTER_FILL: Sprite = s(Gen, 52, 21, 25, 20);
pub const GEN_TOP_RIGHT_END: Sprite = s(Gen, 78, 21, 25, 20);
pub const GEN_TOP_LEFT_RIGHT_FILL: Sprite = s(Gen, 104, 21, 25, 20);
pub const GEN_TOP_RIGHT: Sprite = s(Gen, 130, 21, 25, 20);
pub const GEN_BOTTOM_LEFT: Sprite = s(Gen, 0, 42, 125, 14);
pub const GEN_BOTTOM_RIGHT: Sprite = s(Gen, 0, 57, 125, 14);
pub const GEN_BOTTOM_FILL: Sprite = s(Gen, 127, 72, 25, 14);
pub const GEN_MIDDLE_LEFT: Sprite = s(Gen, 127, 42, 11, 29);
pub const GEN_MIDDLE_LEFT_BOTTOM: Sprite = s(Gen, 158, 42, 11, 24);
pub const GEN_MIDDLE_RIGHT: Sprite = s(Gen, 139, 42, 8, 29);
pub const GEN_MIDDLE_RIGHT_BOTTOM: Sprite = s(Gen, 170, 42, 8, 24);
pub const GEN_CLOSE_SELECTED: Sprite = s(Gen, 148, 42, 9, 9);

/// Position of a character in text.bmp as (row, column). Each glyph is 5x6.
pub fn font_cell(c: char) -> Option<(i32, i32)> {
    let c = fold_char(c);
    Some(match c {
        'a'..='z' => (0, c as i32 - 'a' as i32),
        '"' => (0, 26),
        '@' => (0, 27),
        ' ' => (0, 30),
        '0'..='9' => (1, c as i32 - '0' as i32),
        '\u{2026}' => (1, 10),
        '.' => (1, 11),
        ':' => (1, 12),
        '(' => (1, 13),
        ')' => (1, 14),
        '-' => (1, 15),
        '\'' => (1, 16),
        '!' => (1, 17),
        '_' => (1, 18),
        '+' => (1, 19),
        '\\' => (1, 20),
        '/' => (1, 21),
        '[' | '<' | '{' => (1, 22),
        ']' | '>' | '}' => (1, 23),
        '^' => (1, 24),
        '&' => (1, 25),
        '%' => (1, 26),
        ',' => (1, 27),
        '=' => (1, 28),
        '$' => (1, 29),
        '#' => (1, 30),
        'å' => (2, 0),
        'ö' => (2, 1),
        'ä' => (2, 2),
        '?' => (2, 3),
        '*' => (2, 4),
        _ => return None,
    })
}

/// Lower-case and strip diacritics the way the skin font expects
/// (the font only carries Å, Ä and Ö as accented letters).
fn fold_char(c: char) -> char {
    let lower = c.to_lowercase().next().unwrap_or(c);
    match lower {
        'å' | 'ä' | 'ö' => lower,
        'à' | 'á' | 'â' | 'ã' | 'ā' => 'a',
        'æ' => 'a',
        'ç' | 'č' | 'ć' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' | 'ń' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ø' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'ů' => 'u',
        'ý' | 'ÿ' => 'y',
        'š' | 'ś' | 'ß' => 's',
        'ž' | 'ź' | 'ż' => 'z',
        'ł' => 'l',
        'ř' => 'r',
        'ť' => 't',
        'ď' => 'd',
        _ => lower,
    }
}

pub fn text_char(c: char) -> Option<Sprite> {
    font_cell(c).map(|(row, col)| s(Text, col * 5, row * 6, 5, 6))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_lookup() {
        assert_eq!(font_cell('A'), Some((0, 0)));
        assert_eq!(font_cell('z'), Some((0, 25)));
        assert_eq!(font_cell('7'), Some((1, 7)));
        assert_eq!(font_cell('Ö'), Some((2, 1)));
        assert_eq!(font_cell('é'), Some((0, 4)));
        assert_eq!(font_cell('~'), None);
        assert_eq!(text_char(' ').unwrap().x, 150);
    }
}
