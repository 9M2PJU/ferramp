//! The playlist editor: resizable in 25x29 steps, skin-drawn frame and
//! pop-up button menus, track list drawn with the skin's playlist colours.

use std::time::Instant;

use crate::gfx::{Bitmap, Rect};
use crate::playlist::format_time;
use crate::skin::CursorKind;
use crate::skin::sprites::{self as sp, Sprite};
use crate::state::State;
use crate::text::Weight;
use crate::ui::main_win::{draw_mini_time, draw_text_clipped};
use crate::ui::{Button, Cmd, MenuId, MouseEvent, WinKind, WmRequest};

pub const ROW_H: i32 = 13;
pub const FONT_PX: f32 = 10.0;
const LIST_TOP: i32 = 20;
const LIST_PAD_TOP: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopMenu {
    Add,
    Remove,
    Select,
    Misc,
    List,
}

impl PopMenu {
    fn items(self) -> &'static [(Sprite, Sprite, PopItem)] {
        use PopItem::*;
        match self {
            PopMenu::Add => &[
                (sp::PLAYLIST_ADD_URL, sp::PLAYLIST_ADD_URL_SELECTED, AddUrl),
                (sp::PLAYLIST_ADD_DIR, sp::PLAYLIST_ADD_DIR_SELECTED, AddDir),
                (
                    sp::PLAYLIST_ADD_FILE,
                    sp::PLAYLIST_ADD_FILE_SELECTED,
                    AddFile,
                ),
            ],
            PopMenu::Remove => &[
                (
                    sp::PLAYLIST_REMOVE_MISC,
                    sp::PLAYLIST_REMOVE_MISC_SELECTED,
                    RemoveMisc,
                ),
                (
                    sp::PLAYLIST_REMOVE_ALL,
                    sp::PLAYLIST_REMOVE_ALL_SELECTED,
                    RemoveAll,
                ),
                (sp::PLAYLIST_CROP, sp::PLAYLIST_CROP_SELECTED, Crop),
                (
                    sp::PLAYLIST_REMOVE_SELECTED,
                    sp::PLAYLIST_REMOVE_SELECTED_SELECTED,
                    RemoveSelected,
                ),
            ],
            PopMenu::Select => &[
                (
                    sp::PLAYLIST_INVERT_SELECTION,
                    sp::PLAYLIST_INVERT_SELECTION_SELECTED,
                    Invert,
                ),
                (
                    sp::PLAYLIST_SELECT_ZERO,
                    sp::PLAYLIST_SELECT_ZERO_SELECTED,
                    SelectNone,
                ),
                (
                    sp::PLAYLIST_SELECT_ALL,
                    sp::PLAYLIST_SELECT_ALL_SELECTED,
                    SelectAll,
                ),
            ],
            PopMenu::Misc => &[
                (
                    sp::PLAYLIST_SORT_LIST,
                    sp::PLAYLIST_SORT_LIST_SELECTED,
                    Sort,
                ),
                (
                    sp::PLAYLIST_FILE_INFO,
                    sp::PLAYLIST_FILE_INFO_SELECTED,
                    FileInfo,
                ),
                (
                    sp::PLAYLIST_MISC_OPTIONS,
                    sp::PLAYLIST_MISC_OPTIONS_SELECTED,
                    MiscOptions,
                ),
            ],
            PopMenu::List => &[
                (
                    sp::PLAYLIST_NEW_LIST,
                    sp::PLAYLIST_NEW_LIST_SELECTED,
                    NewList,
                ),
                (
                    sp::PLAYLIST_SAVE_LIST,
                    sp::PLAYLIST_SAVE_LIST_SELECTED,
                    SaveList,
                ),
                (
                    sp::PLAYLIST_LOAD_LIST,
                    sp::PLAYLIST_LOAD_LIST_SELECTED,
                    LoadList,
                ),
            ],
        }
    }

    fn bar(self) -> Sprite {
        match self {
            PopMenu::Add => sp::PLAYLIST_ADD_MENU_BAR,
            PopMenu::Remove => sp::PLAYLIST_REMOVE_MENU_BAR,
            PopMenu::Select => sp::PLAYLIST_SELECT_MENU_BAR,
            PopMenu::Misc => sp::PLAYLIST_MISC_MENU_BAR,
            PopMenu::List => sp::PLAYLIST_LIST_BAR,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopItem {
    AddUrl,
    AddDir,
    AddFile,
    RemoveMisc,
    RemoveAll,
    Crop,
    RemoveSelected,
    Invert,
    SelectNone,
    SelectAll,
    Sort,
    FileInfo,
    MiscOptions,
    NewList,
    SaveList,
    LoadList,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Widget {
    Shade,
    Close,
    TitleBar,
    Row(usize),
    ListBlank,
    Scrollbar,
    MenuButton(PopMenu),
    MenuItem(PopMenu, usize),
    MiniPrev,
    MiniPlay,
    MiniPause,
    MiniStop,
    MiniNext,
    MiniEject,
    MiniTime,
    ScrollUp,
    ScrollDown,
    Resize,
    Body,
}

#[derive(Default)]
pub struct PlWin {
    pressed: Option<Widget>,
    hover_inside: bool,
    open_menu: Option<PopMenu>,
    menu_hover: Option<usize>,
    /// Row dragging: mouse y where the drag started and rows moved so far.
    drag_rows: Option<(i32, i32)>,
    scroll_grab: i32,
    last_click: Option<(usize, Instant)>,
}

pub struct Geometry {
    pub w: i32,
    pub h: i32,
}

impl Geometry {
    pub fn of(st: &State) -> Geometry {
        let (w, h) = st.base_size(WinKind::Playlist);
        Geometry { w, h }
    }

    pub fn list_rect(&self) -> Rect {
        Rect::new(12, LIST_TOP, self.w - 32, self.h - LIST_TOP - 38)
    }

    pub fn visible_rows(&self) -> usize {
        ((self.h - LIST_TOP - 38 - LIST_PAD_TOP) / ROW_H).max(0) as usize
    }

    fn menu_button(&self, m: PopMenu) -> Rect {
        let y = self.h - 30;
        match m {
            PopMenu::Add => Rect::new(14, y, 22, 18),
            PopMenu::Remove => Rect::new(43, y, 22, 18),
            PopMenu::Select => Rect::new(72, y, 22, 18),
            PopMenu::Misc => Rect::new(101, y, 22, 18),
            PopMenu::List => Rect::new(self.w - 44, y, 22, 18),
        }
    }

    fn menu_item_rect(&self, m: PopMenu, i: usize) -> Rect {
        let b = self.menu_button(m);
        let n = m.items().len() as i32;
        Rect::new(b.x, b.y - (n - 1 - i as i32) * 18, 22, 18)
    }

    fn scroll_track(&self) -> Rect {
        Rect::new(self.w - 15, LIST_TOP, 8, self.h - 58)
    }
}

impl PlWin {
    pub fn new() -> PlWin {
        PlWin::default()
    }

    pub fn hit(&self, st: &State, x: i32, y: i32) -> Widget {
        let g = Geometry::of(st);
        let r = Rect::new;
        if st.pl.shade {
            return if r(g.w - 11, 3, 9, 9).contains(x, y) {
                Widget::Close
            } else if r(g.w - 21, 3, 9, 9).contains(x, y) {
                Widget::Shade
            } else if r(g.w - 29, 3, 9, 9).contains(x, y) {
                Widget::Resize
            } else {
                Widget::TitleBar
            };
        }
        if let Some(m) = self.open_menu {
            for i in 0..m.items().len() {
                if g.menu_item_rect(m, i).contains(x, y) {
                    return Widget::MenuItem(m, i);
                }
            }
        }
        if r(g.w - 11, 3, 9, 9).contains(x, y) {
            return Widget::Close;
        }
        if r(g.w - 21, 3, 9, 9).contains(x, y) {
            return Widget::Shade;
        }
        if y < LIST_TOP {
            return Widget::TitleBar;
        }
        let list = g.list_rect();
        if list.contains(x, y) {
            let row = (y - LIST_TOP - LIST_PAD_TOP).div_euclid(ROW_H);
            let idx = st.playlist.scroll as i32 + row;
            if row >= 0
                && (row as usize) < g.visible_rows()
                && (idx as usize) < st.playlist.len()
                && y >= LIST_TOP + LIST_PAD_TOP
            {
                return Widget::Row(idx as usize);
            }
            return Widget::ListBlank;
        }
        if r(g.w - 20, LIST_TOP, 20, g.h - LIST_TOP - 38).contains(x, y) {
            return Widget::Scrollbar;
        }
        for m in [
            PopMenu::Add,
            PopMenu::Remove,
            PopMenu::Select,
            PopMenu::Misc,
            PopMenu::List,
        ] {
            if g.menu_button(m).contains(x, y) {
                return Widget::MenuButton(m);
            }
        }
        let bx = g.w - 150;
        let by = g.h - 38;
        let minis = [
            Widget::MiniPrev,
            Widget::MiniPlay,
            Widget::MiniPause,
            Widget::MiniStop,
            Widget::MiniNext,
            Widget::MiniEject,
        ];
        for (i, w) in minis.iter().enumerate() {
            if r(bx + 3 + i as i32 * 10, by + 22, 10, 10).contains(x, y) {
                return *w;
            }
        }
        if r(bx + 66, by + 23, 30, 6).contains(x, y) {
            return Widget::MiniTime;
        }
        if r(g.w - 15, by + 2, 8, 5).contains(x, y) {
            return Widget::ScrollUp;
        }
        if r(g.w - 15, by + 8, 8, 5).contains(x, y) {
            return Widget::ScrollDown;
        }
        if r(g.w - 20, g.h - 20, 20, 20).contains(x, y) {
            return Widget::Resize;
        }
        Widget::Body
    }

    pub fn cursor(&self, st: &State, x: i32, y: i32) -> CursorKind {
        let w = self.pressed.unwrap_or_else(|| self.hit(st, x, y));
        match (st.pl.shade, w) {
            (_, Widget::Close) => CursorKind::PClose,
            (_, Widget::Shade) => CursorKind::PWinBut,
            (false, Widget::Resize) => CursorKind::PSize,
            (true, Widget::Resize) => CursorKind::PWSSize,
            (true, _) => CursorKind::PWSNorm,
            (false, Widget::TitleBar) => CursorKind::PTBar,
            (false, Widget::Scrollbar) => CursorKind::PVScroll,
            _ => CursorKind::PNormal,
        }
    }

    fn pressed_here(&self, w: Widget) -> bool {
        self.pressed == Some(w) && self.hover_inside
    }

    pub fn render(&mut self, st: &mut State, out: &mut Bitmap) {
        if st.pl.shade {
            self.render_shade(st, out);
            return;
        }
        let g = Geometry::of(st);
        let selected = st.focused == Some(WinKind::Playlist);
        let skin = &st.skin;
        let style = skin.playlist.clone();
        out.fill(style.normal_bg);

        // Top bar: left corner, tiles, centred title, tiles, right corner.
        let (tl, tile, title, tr) = if selected {
            (
                sp::PLAYLIST_TOP_LEFT_SELECTED,
                sp::PLAYLIST_TOP_TILE_SELECTED,
                sp::PLAYLIST_TITLE_BAR_SELECTED,
                sp::PLAYLIST_TOP_RIGHT_CORNER_SELECTED,
            )
        } else {
            (
                sp::PLAYLIST_TOP_LEFT_CORNER,
                sp::PLAYLIST_TOP_TILE,
                sp::PLAYLIST_TITLE_BAR,
                sp::PLAYLIST_TOP_RIGHT_CORNER,
            )
        };
        let spacers = st.pl_size.0 % 2 == 0;
        let fill_w = (g.w - 150 - if spacers { 25 } else { 0 }) / 2;
        let mut x = 0;
        skin.draw(out, tl, x, 0);
        x += 25;
        if spacers {
            skin.draw_part(out, tile, 0, 0, 12, 20, x, 0);
            x += 12;
        }
        tile_right_aligned(skin, out, tile, x, fill_w, 0);
        x += fill_w;
        skin.draw(out, title, x, 0);
        x += 100;
        if spacers {
            skin.draw_part(out, tile, 0, 0, 13, 20, x, 0);
            x += 13;
        }
        tile_right_aligned(skin, out, tile, x, fill_w, 0);
        skin.draw(out, tr, g.w - 25, 0);
        if self.pressed_here(Widget::Shade) {
            skin.draw(out, sp::PLAYLIST_COLLAPSE_SELECTED, g.w - 21, 3);
        }
        if self.pressed_here(Widget::Close) {
            skin.draw(out, sp::PLAYLIST_CLOSE_SELECTED, g.w - 11, 3);
        }

        // Side tiles.
        let mut y = LIST_TOP;
        while y < g.h - 38 {
            let h = (g.h - 38 - y).min(29);
            skin.draw_part(out, sp::PLAYLIST_LEFT_TILE, 0, 0, 12, h, 0, y);
            skin.draw_part(out, sp::PLAYLIST_RIGHT_TILE, 0, 0, 20, h, g.w - 20, y);
            y += 29;
        }

        // Bottom: tiles, corners, optional visualiser panel.
        let by = g.h - 38;
        let mut x = 0;
        while x < g.w {
            skin.draw(out, sp::PLAYLIST_BOTTOM_TILE, x, by);
            x += 25;
        }
        skin.draw(out, sp::PLAYLIST_BOTTOM_LEFT_CORNER, 0, by);
        if st.pl_size.0 > 2 {
            skin.draw(out, sp::PLAYLIST_VISUALIZER_BACKGROUND, g.w - 225, by);
        }
        skin.draw(out, sp::PLAYLIST_BOTTOM_RIGHT_CORNER, g.w - 150, by);

        // Scrollbar handle.
        let track = g.scroll_track();
        let max_scroll = st.playlist.len().saturating_sub(g.visible_rows());
        let frac = if max_scroll == 0 {
            0.0
        } else {
            st.playlist.scroll as f64 / max_scroll as f64
        };
        let hy = track.y + ((track.h - 18) as f64 * frac).floor() as i32;
        let handle = if self.pressed == Some(Widget::Scrollbar) {
            sp::PLAYLIST_SCROLL_HANDLE_SELECTED
        } else {
            sp::PLAYLIST_SCROLL_HANDLE
        };
        skin.draw(out, handle, track.x, hy);

        // Track list.
        self.render_tracks(st, out, &g);

        // Running time and mini time.
        let (sel, sel_ok) = st.playlist.selected_duration();
        let (tot, tot_ok) = st.playlist.total_duration();
        let text = format!(
            "{}{}/{}{}",
            format_long_time(sel),
            if sel_ok { "" } else { "+" },
            format_long_time(tot),
            if tot_ok { "" } else { "+" }
        );
        draw_text_clipped(
            st,
            out,
            &format!("{text:<18}"),
            g.w - 150 + 7,
            by + 10,
            Rect::new(g.w - 150 + 7, by + 10, 90, 6),
        );
        let blink = st.status() != crate::audio::Status::Paused
            || (st.started.elapsed().as_millis() / 1000).is_multiple_of(2);
        draw_mini_time(st, out, g.w - 150 + 66, by + 23, blink);

        // Open pop-up menu.
        if let Some(m) = self.open_menu {
            let skin = &st.skin;
            let items = m.items();
            let top = g.menu_item_rect(m, 0);
            let bar = m.bar();
            skin.draw(out, bar, top.x - 3, top.y);
            for (i, (normal, hot, _)) in items.iter().enumerate() {
                let r = g.menu_item_rect(m, i);
                let hl = self.menu_hover == Some(i);
                skin.draw(out, if hl { *hot } else { *normal }, r.x, r.y);
            }
        }
    }

    fn render_tracks(&self, st: &State, out: &mut Bitmap, g: &Geometry) {
        let list = g.list_rect();
        let style = &st.skin.playlist;
        let n = st.playlist.len();
        let rows = g.visible_rows();
        let fonts = &st.list_fonts;
        // Duration column width.
        let mut dur_w = 0;
        for i in st.playlist.scroll..(st.playlist.scroll + rows).min(n) {
            if let Some(d) = st.playlist.entries[i].duration {
                dur_w = dur_w.max(fonts.measure(&format_time(d), FONT_PX, Weight::Regular));
            }
        }
        for row in 0..rows {
            let idx = st.playlist.scroll + row;
            if idx >= n {
                break;
            }
            let e = &st.playlist.entries[idx];
            let y = LIST_TOP + LIST_PAD_TOP + row as i32 * ROW_H;
            if e.selected {
                out.fill_rect(list.x, y, list.w, ROW_H, style.selected_bg);
            }
            let color = if Some(idx) == st.playlist.current {
                style.current
            } else {
                style.normal
            };
            let ty = y;
            let dur_x = list.x + list.w - 3;
            let title_clip_w = list.w - dur_w - 8;
            let label = if st.show_numbers {
                format!("{}. {}", idx + 1, e.title)
            } else {
                e.title.clone()
            };
            let label = fonts.ellipsize(&label, FONT_PX, Weight::Regular, title_clip_w.max(10));
            fonts.draw(
                out,
                &label,
                list.x,
                ty,
                FONT_PX,
                Weight::Regular,
                color,
                Rect::new(list.x, y, title_clip_w + 2, ROW_H),
                !st.smooth_text,
            );
            if let Some(d) = e.duration {
                let t = format_time(d);
                let tw = fonts.measure(&t, FONT_PX, Weight::Regular);
                fonts.draw(
                    out,
                    &t,
                    dur_x - tw,
                    ty,
                    FONT_PX,
                    Weight::Regular,
                    color,
                    Rect::new(list.x, y, list.w, ROW_H),
                    !st.smooth_text,
                );
            }
        }
    }

    fn render_shade(&mut self, st: &mut State, out: &mut Bitmap) {
        let g = Geometry::of(st);
        let selected = st.focused == Some(WinKind::Playlist);
        let skin = &st.skin;
        let mut x = 0;
        while x < g.w {
            skin.draw(out, sp::PLAYLIST_SHADE_BACKGROUND, x, 0);
            x += 25;
        }
        skin.draw(out, sp::PLAYLIST_SHADE_BACKGROUND_LEFT, 0, 0);
        skin.draw(
            out,
            if selected {
                sp::PLAYLIST_SHADE_BACKGROUND_RIGHT_SELECTED
            } else {
                sp::PLAYLIST_SHADE_BACKGROUND_RIGHT
            },
            g.w - 50,
            0,
        );
        if self.pressed_here(Widget::Shade) {
            skin.draw(out, sp::PLAYLIST_EXPAND_SELECTED, g.w - 21, 3);
        }
        if self.pressed_here(Widget::Close) {
            skin.draw(out, sp::PLAYLIST_CLOSE_SELECTED, g.w - 11, 3);
        }
        let (name, time) = match st.playlist.current_entry() {
            Some(e) => (
                format!("{}. {}", st.playlist.current.unwrap_or(0) + 1, e.title),
                e.duration.map(format_time).unwrap_or_default(),
            ),
            None => ("[No file]".to_string(), String::new()),
        };
        let max_chars = ((205 + st.pl_size.0 * 25) / 5) as usize;
        let name = if name.chars().count() > max_chars {
            let mut s: String = name.chars().take(max_chars - 1).collect();
            s.push('\u{2026}');
            s
        } else {
            name
        };
        draw_text_clipped(st, out, &name, 5, 4, Rect::new(5, 4, g.w - 70, 6));
        let tw = time.chars().count() as i32 * 5;
        draw_text_clipped(
            st,
            out,
            &time,
            g.w - 30 - tw,
            4,
            Rect::new(g.w - 30 - tw, 4, tw, 6),
        );
    }

    pub fn mouse(&mut self, st: &mut State, ev: MouseEvent) -> Vec<WmRequest> {
        let mut wm = Vec::new();
        let g = Geometry::of(st);
        match ev {
            MouseEvent::Down {
                x,
                y,
                button: Button::Right,
                ..
            } => {
                self.open_menu = None;
                match self.hit(st, x, y) {
                    Widget::Row(i) => {
                        if !st.playlist.entries[i].selected {
                            st.playlist.click(i);
                        }
                        st.push(Cmd::ShowMenu(
                            MenuId::PlaylistContext,
                            x,
                            y,
                            WinKind::Playlist,
                        ));
                    }
                    Widget::ListBlank => st.push(Cmd::ShowMenu(
                        MenuId::PlaylistContext,
                        x,
                        y,
                        WinKind::Playlist,
                    )),
                    _ => st.push(Cmd::ShowMenu(MenuId::Main, x, y, WinKind::Playlist)),
                }
                st.dirty = true;
            }
            MouseEvent::Down {
                x,
                y,
                button: Button::Left,
                double,
                mods,
            } => {
                let w = self.hit(st, x, y);
                // A click outside an open menu closes it.
                if let Some(m) = self.open_menu {
                    match w {
                        Widget::MenuItem(mm, i) if mm == m => {
                            self.pressed = Some(w);
                            self.hover_inside = true;
                            self.menu_hover = Some(i);
                            st.dirty = true;
                            return wm;
                        }
                        _ => {
                            self.open_menu = None;
                            self.menu_hover = None;
                            st.dirty = true;
                            if w == Widget::MenuButton(m) {
                                return wm;
                            }
                        }
                    }
                }
                if double && w == Widget::TitleBar {
                    st.push(Cmd::ToggleShade(WinKind::Playlist));
                    return wm;
                }
                match w {
                    Widget::TitleBar | Widget::Body => {
                        wm.push(WmRequest::BeginMove);
                        return wm;
                    }
                    Widget::Resize => {
                        wm.push(WmRequest::BeginResize);
                        return wm;
                    }
                    _ => {}
                }
                self.pressed = Some(w);
                self.hover_inside = true;
                match w {
                    Widget::Row(i) => {
                        let dbl = double
                            || matches!(self.last_click, Some((j, t)) if j == i && t.elapsed().as_millis() < 400);
                        self.last_click = Some((i, Instant::now()));
                        if dbl && !mods.shift && !mods.ctrl {
                            st.playlist.click(i);
                            st.push(Cmd::PlayIndex(i));
                            self.pressed = None;
                        } else if mods.shift {
                            st.playlist.shift_click(i);
                            self.pressed = None;
                        } else if mods.ctrl {
                            st.playlist.ctrl_click(i);
                            self.pressed = None;
                        } else {
                            if !st.playlist.entries[i].selected {
                                st.playlist.click(i);
                            }
                            self.drag_rows = Some((y, 0));
                        }
                    }
                    Widget::ListBlank => {
                        st.playlist.select_none();
                    }
                    Widget::Scrollbar => {
                        let track = g.scroll_track();
                        let max_scroll = st.playlist.len().saturating_sub(g.visible_rows());
                        let frac = if max_scroll == 0 {
                            0.0
                        } else {
                            st.playlist.scroll as f64 / max_scroll as f64
                        };
                        let hy = ((track.h - 18) as f64 * frac).floor() as i32;
                        let rel = y - track.y;
                        self.scroll_grab = if rel >= hy && rel < hy + 18 {
                            rel - hy
                        } else {
                            9
                        };
                        self.drag_scroll(st, y);
                    }
                    Widget::MenuButton(m) => {
                        // The menu opens over the button: its last item sits under the mouse.
                        self.open_menu = Some(m);
                        self.menu_hover = Some(m.items().len() - 1);
                    }
                    _ => {}
                }
                st.dirty = true;
            }
            MouseEvent::Move { x, y } => {
                if let Some(m) = self.open_menu {
                    let hov = match self.hit(st, x, y) {
                        Widget::MenuItem(mm, i) if mm == m => Some(i),
                        _ => None,
                    };
                    if hov != self.menu_hover {
                        self.menu_hover = hov;
                        st.dirty = true;
                    }
                }
                let Some(p) = self.pressed else { return wm };
                let inside = self.hit(st, x, y) == p;
                if inside != self.hover_inside {
                    self.hover_inside = inside;
                    st.dirty = true;
                }
                match p {
                    Widget::Row(_) => {
                        if let Some((y0, moved)) = self.drag_rows {
                            let want = (y - y0).div_euclid(ROW_H);
                            if want != moved {
                                let applied =
                                    st.playlist.move_selected((want - moved) as isize) as i32;
                                self.drag_rows = Some((y0, moved + applied));
                                st.dirty = true;
                            }
                        }
                    }
                    Widget::Scrollbar => self.drag_scroll(st, y),
                    _ => {}
                }
            }
            MouseEvent::Up {
                x,
                y,
                button: Button::Left,
            } => {
                let Some(p) = self.pressed.take() else {
                    return wm;
                };
                self.drag_rows = None;
                let now = self.hit(st, x, y);
                let inside = now == p;
                st.dirty = true;
                match p {
                    Widget::MenuButton(m) => {
                        // Press-drag-release onto an item triggers it directly.
                        if let Widget::MenuItem(mm, i) = now
                            && mm == m
                        {
                            self.trigger(st, m, i, x, y);
                        }
                    }
                    Widget::MenuItem(m, i) if inside => self.trigger(st, m, i, x, y),
                    _ if !inside => {}
                    Widget::Shade => st.push(Cmd::ToggleShade(WinKind::Playlist)),
                    Widget::Close => st.push(Cmd::CloseWindow(WinKind::Playlist)),
                    Widget::MiniPrev => st.push(Cmd::Previous),
                    Widget::MiniPlay => st.push(Cmd::Play),
                    Widget::MiniPause => st.push(Cmd::Pause),
                    Widget::MiniStop => st.push(Cmd::Stop),
                    Widget::MiniNext => st.push(Cmd::Next),
                    Widget::MiniEject => st.push(Cmd::Eject),
                    Widget::MiniTime => st.push(Cmd::ToggleTimeMode),
                    Widget::ScrollUp => st.push(Cmd::PlScrollBy(-4)),
                    Widget::ScrollDown => st.push(Cmd::PlScrollBy(4)),
                    _ => {}
                }
            }
            MouseEvent::Wheel { delta, .. } => st.push(Cmd::PlScrollBy(-delta * 3)),
            _ => {}
        }
        wm
    }

    fn trigger(&mut self, st: &mut State, m: PopMenu, i: usize, x: i32, y: i32) {
        self.open_menu = None;
        self.menu_hover = None;
        let item = m.items()[i].2;
        let cmd = match item {
            PopItem::AddUrl => Cmd::PlAddUrl,
            PopItem::AddDir => Cmd::PlAddFolder,
            PopItem::AddFile => Cmd::PlAddFiles,
            PopItem::RemoveMisc => Cmd::ShowMenu(MenuId::PlaylistRemove, x, y, WinKind::Playlist),
            PopItem::RemoveAll => Cmd::PlClear,
            PopItem::Crop => Cmd::PlCrop,
            PopItem::RemoveSelected => Cmd::PlRemoveSelected,
            PopItem::Invert => Cmd::PlInvertSelection,
            PopItem::SelectNone => Cmd::PlSelectNone,
            PopItem::SelectAll => Cmd::PlSelectAll,
            PopItem::Sort => Cmd::ShowMenu(MenuId::PlaylistSort, x, y, WinKind::Playlist),
            PopItem::FileInfo => Cmd::FileInfo,
            PopItem::MiscOptions => {
                Cmd::ShowMenu(MenuId::PlaylistMiscOptions, x, y, WinKind::Playlist)
            }
            PopItem::NewList => Cmd::PlNew,
            PopItem::SaveList => Cmd::PlSave,
            PopItem::LoadList => Cmd::PlLoad,
        };
        st.push(cmd);
    }

    fn drag_scroll(&mut self, st: &mut State, y: i32) {
        let g = Geometry::of(st);
        let track = g.scroll_track();
        let max_scroll = st.playlist.len().saturating_sub(g.visible_rows());
        let travel = (track.h - 18).max(1);
        let pos = (y - track.y - self.scroll_grab).clamp(0, travel);
        st.playlist.scroll = ((pos as f64 / travel as f64) * max_scroll as f64).round() as usize;
        st.dirty = true;
    }

    pub fn close_menus(&mut self, st: &mut State) {
        if self.open_menu.take().is_some() {
            st.dirty = true;
        }
        self.pressed = None;
        self.drag_rows = None;
    }

    /// Row index under a window y coordinate (for drops), clamped to the list.
    pub fn row_at(&self, st: &State, y: i32) -> usize {
        let row = ((y - LIST_TOP - LIST_PAD_TOP) as f64 / ROW_H as f64)
            .round()
            .max(0.0) as usize;
        (st.playlist.scroll + row).min(st.playlist.len())
    }
}

/// Running times switch to h:mm:ss from one hour on.
pub fn format_long_time(secs: f64) -> String {
    let s = secs.max(0.0).floor() as u64;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
    } else {
        format_time(secs)
    }
}

fn tile_right_aligned(
    skin: &crate::skin::Skin,
    out: &mut Bitmap,
    tile: Sprite,
    x: i32,
    w: i32,
    y: i32,
) {
    if w <= 0 {
        return;
    }
    // Tiles are anchored at the right edge of the area.
    let mut tx = x + w - 25;
    while tx > x - 25 {
        let ox = (x - tx).max(0);
        skin.draw_part(out, tile, ox, 0, 25 - ox, 20, tx + ox, y);
        tx -= 25;
    }
}

pub fn ensure_visible(st: &mut State, idx: usize) {
    let g = Geometry::of(st);
    let rows = g.visible_rows().max(1);
    if idx < st.playlist.scroll {
        st.playlist.scroll = idx;
    } else if idx >= st.playlist.scroll + rows {
        st.playlist.scroll = idx + 1 - rows;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{Engine, Source};
    use crate::skin::Skin;

    fn state(rows: usize, size: (i32, i32)) -> State {
        let mut st = State::new(Engine::headless(), Skin::builtin());
        st.pl_size = size;
        let items = (0..rows)
            .map(|i| {
                (
                    Source::File(format!("/m/{i}.mp3").into()),
                    Some(format!("Track {i}")),
                    Some(60.0),
                )
            })
            .collect();
        st.playlist.append(items);
        st
    }

    #[test]
    fn geometry_and_hits() {
        let st = state(30, (0, 4));
        let g = Geometry::of(&st);
        assert_eq!((g.w, g.h), (275, 232));
        assert_eq!(g.visible_rows(), 13);
        let w = PlWin::new();
        assert_eq!(w.hit(&st, 50, LIST_TOP + LIST_PAD_TOP + 1), Widget::Row(0));
        assert_eq!(
            w.hit(&st, 50, LIST_TOP + LIST_PAD_TOP + ROW_H * 2 + 1),
            Widget::Row(2)
        );
        assert_eq!(w.hit(&st, 20, g.h - 25), Widget::MenuButton(PopMenu::Add));
        assert_eq!(
            w.hit(&st, g.w - 35, g.h - 25),
            Widget::MenuButton(PopMenu::List)
        );
        assert_eq!(w.hit(&st, g.w - 5, g.h - 5), Widget::Resize);
        assert_eq!(w.hit(&st, g.w - 7, 6), Widget::Close);
        assert_eq!(w.hit(&st, 100, 5), Widget::TitleBar);
    }

    #[test]
    fn menus_open_over_their_button() {
        let st = state(1, (2, 0));
        let g = Geometry::of(&st);
        for m in [
            PopMenu::Add,
            PopMenu::Remove,
            PopMenu::Select,
            PopMenu::Misc,
            PopMenu::List,
        ] {
            let last = m.items().len() - 1;
            assert_eq!(g.menu_item_rect(m, last), g.menu_button(m));
            assert!(g.menu_item_rect(m, 0).y >= 0);
        }
    }

    #[test]
    fn scrolling_keeps_selection_visible() {
        let mut st = state(40, (0, 0));
        ensure_visible(&mut st, 30);
        let rows = Geometry::of(&st).visible_rows();
        assert!(st.playlist.scroll <= 30 && 30 < st.playlist.scroll + rows);
        ensure_visible(&mut st, 0);
        assert_eq!(st.playlist.scroll, 0);
    }

    #[test]
    fn long_running_times() {
        assert_eq!(format_long_time(59.0), "0:59");
        assert_eq!(format_long_time(7200.0), "2:00:00");
    }
}
