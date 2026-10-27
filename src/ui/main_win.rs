//! The main window (275x116) and its window-shade form (275x14).

use std::time::Instant;

use crate::audio::Status;
use crate::gfx::{Bitmap, Rect};
use crate::skin::CursorKind;
use crate::skin::sprites::{self as sp, Sheet, Sprite};
use crate::state::{Focus, State, marquee_loop_text, time_digits};
use crate::ui::vis::VisMode;
use crate::ui::{Button, Cmd, MenuId, MouseEvent, WinKind, WmRequest};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Widget {
    Options,
    Minimize,
    Shade,
    Close,
    ClutterO,
    ClutterA,
    ClutterI,
    ClutterD,
    ClutterV,
    Time,
    Vis,
    Marquee,
    Volume,
    Balance,
    EqButton,
    PlButton,
    Position,
    Prev,
    Play,
    Pause,
    Stop,
    Next,
    Eject,
    Shuffle,
    Repeat,
    About,
    TitleBar,
    Body,
}

const R: fn(i32, i32, i32, i32) -> Rect = Rect::new;

fn layout_normal() -> [(Widget, Rect); 26] {
    [
        (Widget::Options, R(6, 3, 9, 9)),
        (Widget::Minimize, R(244, 3, 9, 9)),
        (Widget::Shade, R(254, 3, 9, 9)),
        (Widget::Close, R(264, 3, 9, 9)),
        (Widget::ClutterO, R(10, 25, 8, 8)),
        (Widget::ClutterA, R(10, 33, 8, 7)),
        (Widget::ClutterI, R(10, 40, 8, 7)),
        (Widget::ClutterD, R(10, 47, 8, 8)),
        (Widget::ClutterV, R(10, 55, 8, 7)),
        (Widget::Time, R(36, 26, 63, 13)),
        (Widget::Vis, R(24, 43, 76, 16)),
        (Widget::Marquee, R(111, 21, 154, 12)),
        (Widget::Volume, R(107, 57, 68, 13)),
        (Widget::Balance, R(177, 57, 38, 13)),
        (Widget::EqButton, R(219, 58, 23, 12)),
        (Widget::PlButton, R(242, 58, 23, 12)),
        (Widget::Position, R(16, 72, 248, 10)),
        (Widget::Prev, R(16, 88, 23, 18)),
        (Widget::Play, R(39, 88, 23, 18)),
        (Widget::Pause, R(62, 88, 23, 18)),
        (Widget::Stop, R(85, 88, 23, 18)),
        (Widget::Next, R(108, 88, 22, 18)),
        (Widget::Eject, R(136, 89, 22, 16)),
        (Widget::Shuffle, R(164, 89, 47, 15)),
        (Widget::Repeat, R(210, 89, 28, 15)),
        (Widget::About, R(253, 91, 13, 15)),
    ]
}

fn layout_shade() -> [(Widget, Rect); 13] {
    [
        (Widget::Options, R(6, 3, 9, 9)),
        (Widget::Minimize, R(244, 3, 9, 9)),
        (Widget::Shade, R(254, 3, 9, 9)),
        (Widget::Close, R(264, 3, 9, 9)),
        (Widget::Vis, R(79, 5, 38, 5)),
        (Widget::Time, R(127, 4, 30, 6)),
        (Widget::Prev, R(169, 2, 7, 10)),
        (Widget::Play, R(176, 2, 10, 10)),
        (Widget::Pause, R(186, 2, 9, 10)),
        (Widget::Stop, R(195, 2, 9, 10)),
        (Widget::Next, R(204, 2, 10, 10)),
        (Widget::Eject, R(215, 2, 10, 10)),
        (Widget::Position, R(226, 4, 17, 7)),
    ]
}

pub const VOLUME_TRAVEL: i32 = 51;
pub const BALANCE_TRAVEL: i32 = 24;
pub const POSITION_TRAVEL: i32 = 218;
const SHADE_POSITION_TRAVEL: i32 = 14;

#[derive(Default)]
pub struct MainWin {
    pressed: Option<Widget>,
    hover_inside: bool,
    /// Grab offset inside a slider thumb.
    grab: i32,
    drag_start_x: i32,
    /// Title-bar buttons are painted from their own sprites once used.
    clicked: [bool; 4],
    paused_at: Option<Instant>,
    last_status: Option<Status>,
}

impl MainWin {
    pub fn new() -> MainWin {
        MainWin::default()
    }

    pub fn hit(&self, st: &State, x: i32, y: i32) -> Widget {
        let found = if st.main.shade {
            layout_shade().iter().find(|(_, r)| r.contains(x, y)).map(|(w, _)| *w)
        } else {
            layout_normal().iter().find(|(_, r)| r.contains(x, y)).map(|(w, _)| *w)
        };
        match found {
            Some(w) => w,
            None if st.main.shade || y < 14 => Widget::TitleBar,
            None => Widget::Body,
        }
    }

    pub fn cursor(&self, st: &State, x: i32, y: i32) -> CursorKind {
        let w = if let Some(p) = self.pressed { p } else { self.hit(st, x, y) };
        match (st.main.shade, w) {
            (false, Widget::Options) => CursorKind::MainMenu,
            (true, Widget::Options) => CursorKind::MMenu,
            (_, Widget::Minimize) => CursorKind::Min,
            (_, Widget::Close) => CursorKind::Close,
            (_, Widget::Shade) => CursorKind::WinBut,
            (false, Widget::Position) => CursorKind::PosBar,
            (true, Widget::Position) => CursorKind::WSPosBar,
            (_, Widget::Volume | Widget::Balance) => CursorKind::VolBal,
            (_, Widget::Marquee) => CursorKind::SongName,
            (false, Widget::TitleBar) => CursorKind::TitleBar,
            (true, _) => CursorKind::WSNormal,
            _ => CursorKind::Normal,
        }
    }

    /// Tooltip for the control under the mouse (as in the original:
    /// specific texts for a few controls, the program name elsewhere).
    pub fn tooltip(&self, st: &State, x: i32, y: i32) -> String {
        let generic = concat!("Ferramp ", env!("CARGO_PKG_VERSION"));
        match self.hit(st, x, y) {
            Widget::Options => "Ferramp Menu".into(),
            Widget::Minimize => "Minimize".into(),
            Widget::Shade => "Toggle Windowshade Mode".into(),
            Widget::Close => "Close".into(),
            Widget::ClutterO if !st.main.shade => "Options Menu".into(),
            Widget::ClutterA if !st.main.shade => "Toggle Always-On-Top".into(),
            Widget::ClutterI if !st.main.shade => "File Info Box".into(),
            Widget::ClutterD if !st.main.shade => "Toggle Doublesize Mode".into(),
            Widget::ClutterV if !st.main.shade => "Visualization Menu".into(),
            Widget::Marquee => "Song Title (right-click for options)".into(),
            _ => generic.into(),
        }
    }

    // ------------------------------------------------------------------
    // Rendering

    pub fn render(&mut self, st: &mut State, out: &mut Bitmap) {
        let status = st.status();
        if self.last_status != Some(status) {
            if status == Status::Paused {
                self.paused_at = Some(Instant::now());
            }
            self.last_status = Some(status);
        }
        if st.main.shade {
            self.render_shade(st, out);
        } else {
            self.render_normal(st, out);
        }
    }

    fn pressed_here(&self, w: Widget) -> bool {
        self.pressed == Some(w) && self.hover_inside
    }

    fn blink_visible(&self, st: &State) -> bool {
        if st.status() != Status::Paused {
            return true;
        }
        let since = self.paused_at.map(|t| t.elapsed().as_millis()).unwrap_or(0);
        (since / 1000) % 2 == 0
    }

    fn title_buttons(&self, st: &State, out: &mut Bitmap, shade: bool) {
        let skin = &st.skin;
        let selected = st.focused == Some(WinKind::Main);
        let draw = |out: &mut Bitmap, i: usize, w: Widget, normal: Sprite, down: Sprite, x: i32| {
            if !selected {
                return;
            }
            if self.pressed_here(w) {
                skin.draw(out, down, x, 3);
            } else if self.clicked[i] {
                skin.draw(out, normal, x, 3);
            }
        };
        draw(out, 0, Widget::Options, sp::MAIN_OPTIONS_BUTTON, sp::MAIN_OPTIONS_BUTTON_DEPRESSED, 6);
        draw(out, 1, Widget::Minimize, sp::MAIN_MINIMIZE_BUTTON, sp::MAIN_MINIMIZE_BUTTON_DEPRESSED, 244);
        if shade {
            if self.pressed_here(Widget::Shade) {
                skin.draw(out, sp::MAIN_SHADE_BUTTON_SELECTED_DEPRESSED, 254, 3);
            } else if selected {
                skin.draw(out, sp::MAIN_SHADE_BUTTON_SELECTED, 254, 3);
            }
        } else {
            draw(out, 2, Widget::Shade, sp::MAIN_SHADE_BUTTON, sp::MAIN_SHADE_BUTTON_DEPRESSED, 254);
        }
        draw(out, 3, Widget::Close, sp::MAIN_CLOSE_BUTTON, sp::MAIN_CLOSE_BUTTON_DEPRESSED, 264);
    }

    fn render_normal(&mut self, st: &mut State, out: &mut Bitmap) {
        let status = st.status();
        let selected = st.focused == Some(WinKind::Main);
        {
            let skin = &st.skin;
            skin.draw(out, sp::MAIN_WINDOW_BACKGROUND, 0, 0);
            let title = match (st.llama, selected) {
                (false, true) => sp::MAIN_TITLE_BAR_SELECTED,
                (false, false) => sp::MAIN_TITLE_BAR,
                (true, true) => sp::MAIN_EASTER_EGG_TITLE_BAR_SELECTED,
                (true, false) => sp::MAIN_EASTER_EGG_TITLE_BAR,
            };
            skin.draw(out, title, 0, 0);
        }
        self.title_buttons(st, out, false);
        let skin = &st.skin;

        // Clutter bar.
        skin.draw(out, sp::MAIN_CLUTTER_BAR_BACKGROUND, 10, 22);
        if self.pressed_here(Widget::ClutterO) {
            skin.draw(out, sp::MAIN_CLUTTER_BAR_BUTTON_O_SELECTED, 10, 25);
        }
        if self.pressed_here(Widget::ClutterA) || st.always_on_top {
            skin.draw(out, sp::MAIN_CLUTTER_BAR_BUTTON_A_SELECTED, 10, 33);
        }
        if self.pressed_here(Widget::ClutterI) {
            skin.draw(out, sp::MAIN_CLUTTER_BAR_BUTTON_I_SELECTED, 10, 40);
        }
        if self.pressed_here(Widget::ClutterD) || st.doubled {
            skin.draw(out, sp::MAIN_CLUTTER_BAR_BUTTON_D_SELECTED, 10, 47);
        }
        if self.pressed_here(Widget::ClutterV) {
            skin.draw(out, sp::MAIN_CLUTTER_BAR_BUTTON_V_SELECTED, 10, 55);
        }

        // Play status. While a track is loaded the status and minus-sign
        // areas are cleared to the visualiser's dotted background first.
        if status != Status::Stopped {
            vis_dots(out, &st.skin.viscolors, 24, 28, 22, 9);
            vis_dots(out, &st.skin.viscolors, 36, 26, 10, 13);
        }
        match status {
            Status::Playing if st.working => skin.draw(out, sp::MAIN_WORKING_INDICATOR, 24, 28),
            Status::Playing => {
                skin.draw(out, sp::MAIN_PLAYING_INDICATOR, 26, 28);
                skin.draw(out, sp::MAIN_NOT_WORKING_INDICATOR, 24, 28);
            }
            Status::Paused => skin.draw(out, sp::MAIN_PAUSED_INDICATOR, 26, 28),
            Status::Stopped => {
                // The indicator column left of the status icon shows the
                // visualiser's dotted background while stopped.
                vis_dots(out, &st.skin.viscolors, 24, 28, 2, 9);
                skin.draw(out, sp::MAIN_STOPPED_INDICATOR, 26, 28);
            }
        }

        // Time.
        if status != Status::Stopped && self.blink_visible(st) {
            let remaining = st.time_remaining && st.duration().is_some();
            let secs = if remaining { st.duration().unwrap_or(0.0) - st.position() } else { st.position() };
            let d = time_digits(secs);
            let ex = skin.uses_nums_ex();
            if ex {
                skin.draw(out, if remaining { sp::MINUS_SIGN_EX } else { sp::NO_MINUS_SIGN_EX }, 38, 26);
            } else {
                skin.draw(out, if remaining { sp::MINUS_SIGN } else { sp::NO_MINUS_SIGN }, 38, 32);
            }
            for (i, x) in [48, 60, 78, 90].iter().enumerate() {
                skin.draw(out, sp::digit(d[i] as i32, ex), *x, 26);
            }
        }

        // Song title, bitrate, sample rate, channels.
        let text = st.marquee_text();
        let looped = marquee_loop_text(&text);
        let off = st.marquee_offset(&text);
        draw_text_clipped(st, out, &looped, 111 - off, 27, Rect::new(111, 27, 154, 6));
        if status != Status::Stopped {
            let kbps = format!("{:>3}", st.engine.shared.kbps.load(std::sync::atomic::Ordering::Relaxed).min(999));
            let kbps = if kbps.trim() == "0" && st.now.kbps > 0 { format!("{:>3}", st.now.kbps.min(999)) } else { kbps };
            draw_text_clipped(st, out, &kbps, 111, 43, Rect::new(111, 43, 15, 6));
            let khz = format!("{:>2}", st.now.khz.min(99));
            draw_text_clipped(st, out, &khz, 156, 43, Rect::new(156, 43, 10, 6));
        }
        let skin = &st.skin;
        let (mono, stereo) = match status {
            Status::Stopped => (false, false),
            _ => (st.now.channels == 1, st.now.channels >= 2),
        };
        skin.draw(out, if mono { sp::MAIN_MONO_SELECTED } else { sp::MAIN_MONO }, 212, 41);
        skin.draw(out, if stereo { sp::MAIN_STEREO_SELECTED } else { sp::MAIN_STEREO }, 239, 41);

        // Volume.
        let vi = volume_frame(st.volume);
        skin.draw_part(out, sp::MAIN_VOLUME_BACKGROUND, 0, vi * 15, 68, 13, 107, 57);
        let vx = volume_thumb_x(st.volume);
        skin.draw(
            out,
            if self.pressed == Some(Widget::Volume) { sp::MAIN_VOLUME_THUMB_SELECTED } else { sp::MAIN_VOLUME_THUMB },
            107 + vx,
            58,
        );

        // Balance (falls back to the volume art when the skin has no balance.bmp).
        let (bg, thumb, thumb_on) = balance_sprites(st);
        let bi = balance_frame(st.balance);
        skin.draw_part(out, bg, 0, bi * 15, 38, 13, 177, 57);
        skin.draw(out, if self.pressed == Some(Widget::Balance) { thumb_on } else { thumb }, 177 + balance_thumb_x(st.balance), 58);

        // EQ / PL toggles.
        let eq_on = st.eqw.visible;
        let pl_on = st.pl.visible;
        let eq_sprite = match (eq_on, self.pressed_here(Widget::EqButton)) {
            (false, false) => sp::MAIN_EQ_BUTTON,
            (true, false) => sp::MAIN_EQ_BUTTON_SELECTED,
            (false, true) => sp::MAIN_EQ_BUTTON_DEPRESSED,
            (true, true) => sp::MAIN_EQ_BUTTON_DEPRESSED_SELECTED,
        };
        skin.draw(out, eq_sprite, 219, 58);
        let pl_sprite = match (pl_on, self.pressed_here(Widget::PlButton)) {
            (false, false) => sp::MAIN_PLAYLIST_BUTTON,
            (true, false) => sp::MAIN_PLAYLIST_BUTTON_SELECTED,
            (false, true) => sp::MAIN_PLAYLIST_BUTTON_DEPRESSED,
            (true, true) => sp::MAIN_PLAYLIST_BUTTON_DEPRESSED_SELECTED,
        };
        skin.draw(out, pl_sprite, 242, 58);

        // Seek bar: only shown while a seekable track is playing or paused.
        if status != Status::Stopped
            && let Some(frac) = self.position_fraction(st)
        {
            skin.draw(out, sp::MAIN_POSITION_SLIDER_BACKGROUND, 16, 72);
            let x = (frac * POSITION_TRAVEL as f64).floor() as i32;
            let thumb = if self.pressed == Some(Widget::Position) {
                sp::MAIN_POSITION_SLIDER_THUMB_SELECTED
            } else {
                sp::MAIN_POSITION_SLIDER_THUMB
            };
            skin.draw(out, thumb, 16 + x, 72);
        }

        // Transport buttons.
        let buttons = [
            (Widget::Prev, sp::MAIN_PREVIOUS_BUTTON, sp::MAIN_PREVIOUS_BUTTON_ACTIVE, 16, 88),
            (Widget::Play, sp::MAIN_PLAY_BUTTON, sp::MAIN_PLAY_BUTTON_ACTIVE, 39, 88),
            (Widget::Pause, sp::MAIN_PAUSE_BUTTON, sp::MAIN_PAUSE_BUTTON_ACTIVE, 62, 88),
            (Widget::Stop, sp::MAIN_STOP_BUTTON, sp::MAIN_STOP_BUTTON_ACTIVE, 85, 88),
            (Widget::Next, sp::MAIN_NEXT_BUTTON, sp::MAIN_NEXT_BUTTON_ACTIVE, 108, 88),
            (Widget::Eject, sp::MAIN_EJECT_BUTTON, sp::MAIN_EJECT_BUTTON_ACTIVE, 136, 89),
        ];
        for (w, normal, active, x, y) in buttons {
            skin.draw(out, if self.pressed_here(w) { active } else { normal }, x, y);
        }
        let shuffle = match (st.shuffle, self.pressed_here(Widget::Shuffle)) {
            (false, false) => sp::MAIN_SHUFFLE_BUTTON,
            (false, true) => sp::MAIN_SHUFFLE_BUTTON_DEPRESSED,
            (true, false) => sp::MAIN_SHUFFLE_BUTTON_SELECTED,
            (true, true) => sp::MAIN_SHUFFLE_BUTTON_SELECTED_DEPRESSED,
        };
        skin.draw(out, shuffle, 164, 89);
        let repeat = match (st.repeat, self.pressed_here(Widget::Repeat)) {
            (false, false) => sp::MAIN_REPEAT_BUTTON,
            (false, true) => sp::MAIN_REPEAT_BUTTON_DEPRESSED,
            (true, false) => sp::MAIN_REPEAT_BUTTON_SELECTED,
            (true, true) => sp::MAIN_REPEAT_BUTTON_SELECTED_DEPRESSED,
        };
        skin.draw(out, repeat, 210, 89);

        // Visualisation.
        draw_vis(st, out, 24, 43, false);
    }

    fn render_shade(&mut self, st: &mut State, out: &mut Bitmap) {
        let status = st.status();
        let selected = st.focused == Some(WinKind::Main);
        st.skin.draw(out, if selected { sp::MAIN_SHADE_BACKGROUND_SELECTED } else { sp::MAIN_SHADE_BACKGROUND }, 0, 0);
        self.title_buttons(st, out, true);
        // Mini time.
        draw_mini_time(st, out, 127, 4, self.blink_visible(st));
        // Seek bar.
        let skin = &st.skin;
        skin.draw(out, sp::MAIN_SHADE_POSITION_BACKGROUND, 226, 4);
        if status != Status::Stopped
            && let Some(frac) = self.position_fraction(st)
        {
            let thumb = if frac <= 0.33 {
                sp::MAIN_SHADE_POSITION_THUMB_LEFT
            } else if frac >= 0.66 {
                sp::MAIN_SHADE_POSITION_THUMB_RIGHT
            } else {
                sp::MAIN_SHADE_POSITION_THUMB
            };
            skin.draw(out, thumb, 226 + (frac * SHADE_POSITION_TRAVEL as f64).round() as i32, 4);
        }
        draw_vis(st, out, 79, 5, true);
    }

    fn position_fraction(&self, st: &State) -> Option<f64> {
        let d = st.duration()?;
        if d <= 0.0 {
            return None;
        }
        if let Some(Focus::Position(p)) = st.focus {
            return Some((p / d).clamp(0.0, 1.0));
        }
        Some((st.position().floor() / d).clamp(0.0, 1.0))
    }

    // ------------------------------------------------------------------
    // Input

    pub fn mouse(&mut self, st: &mut State, ev: MouseEvent) -> Vec<WmRequest> {
        let mut wm = Vec::new();
        match ev {
            MouseEvent::Down { x, y, button: Button::Right, .. } => {
                let w = self.hit(st, x, y);
                let menu = match w {
                    Widget::Vis | Widget::ClutterV => MenuId::Vis,
                    Widget::Marquee => MenuId::SongTitle,
                    Widget::Time => MenuId::Time,
                    _ => MenuId::Main,
                };
                st.push(Cmd::ShowMenu(menu, x, y, WinKind::Main));
            }
            MouseEvent::Down { x, y, button: Button::Left, double, .. } => {
                let w = self.hit(st, x, y);
                if double && matches!(w, Widget::TitleBar) {
                    st.push(Cmd::ToggleShade(WinKind::Main));
                    return wm;
                }
                if double && matches!(w, Widget::Marquee) {
                    st.push(Cmd::FileInfo);
                    return wm;
                }
                match w {
                    Widget::TitleBar => {
                        wm.push(WmRequest::BeginMove);
                        return wm;
                    }
                    // EasyMove: drag the window from anywhere.
                    Widget::Body => {
                        if st.easy_move {
                            wm.push(WmRequest::BeginMove);
                        }
                        return wm;
                    }
                    _ => {}
                }
                if st.easy_move && matches!(w, Widget::Time | Widget::Vis) {
                    wm.push(WmRequest::BeginMove);
                    return wm;
                }
                self.pressed = Some(w);
                self.hover_inside = true;
                match w {
                    Widget::Options => {
                        self.clicked[0] = true;
                        st.push(Cmd::ShowMenu(MenuId::Main, 6, 14, WinKind::Main));
                    }
                    Widget::ClutterO => {
                        st.focus = Some(Focus::Text("Options menu"));
                        st.push(Cmd::ShowMenu(MenuId::Options, 18, 25, WinKind::Main));
                    }
                    Widget::ClutterV => {
                        st.focus = Some(Focus::Text("Visualization menu"));
                        st.push(Cmd::ShowMenu(MenuId::Vis, 18, 55, WinKind::Main));
                    }
                    Widget::ClutterA => st.focus = Some(Focus::Text("always-on-top")),
                    Widget::ClutterI => st.focus = Some(Focus::Text("File info box")),
                    Widget::ClutterD => st.focus = Some(Focus::Double),
                    Widget::Marquee => {
                        st.marquee_dragging = true;
                        self.drag_start_x = x + st.marquee_drag;
                    }
                    Widget::Volume => {
                        st.focus = Some(Focus::Volume);
                        self.grab = slider_grab(x - 107, volume_thumb_x(st.volume), 14);
                        self.drag_volume(st, x);
                    }
                    Widget::Balance => {
                        st.focus = Some(Focus::Balance);
                        self.grab = slider_grab(x - 177, balance_thumb_x(st.balance), 14);
                        self.drag_balance(st, x);
                    }
                    Widget::Position => {
                        if st.duration().is_some() && st.status() != Status::Stopped {
                            let (x0, travel, tw) =
                                if st.main.shade { (226, SHADE_POSITION_TRAVEL, 3) } else { (16, POSITION_TRAVEL, 29) };
                            let cur = self.position_fraction(st).unwrap_or(0.0);
                            self.grab = slider_grab(x - x0, (cur * travel as f64).round() as i32, tw);
                            self.drag_position(st, x);
                        } else {
                            self.pressed = None;
                        }
                    }
                    _ => {}
                }
                st.dirty = true;
            }
            MouseEvent::Move { x, y } => {
                if let Some(p) = self.pressed {
                    let inside = self.hit(st, x, y) == p;
                    if inside != self.hover_inside {
                        self.hover_inside = inside;
                        st.dirty = true;
                    }
                    match p {
                        Widget::Volume => self.drag_volume(st, x),
                        Widget::Balance => self.drag_balance(st, x),
                        Widget::Position => self.drag_position(st, x),
                        Widget::Marquee => {
                            st.marquee_drag = self.drag_start_x - x;
                            st.dirty = true;
                        }
                        _ => {}
                    }
                }
            }
            MouseEvent::Up { x, y, button: Button::Left } => {
                let Some(p) = self.pressed.take() else { return wm };
                let inside = self.hit(st, x, y) == p;
                st.dirty = true;
                match p {
                    Widget::Volume | Widget::Balance => st.focus = None,
                    Widget::ClutterO | Widget::ClutterV => st.focus = None,
                    Widget::ClutterA | Widget::ClutterI if !inside => st.focus = None,
                    Widget::Position => {
                        if let Some(Focus::Position(t)) = st.focus.take() {
                            st.push(Cmd::SeekTo(t));
                        }
                    }
                    Widget::Marquee => {
                        st.marquee_dragging = false;
                        st.marquee_resume = Instant::now() + std::time::Duration::from_secs(1);
                    }
                    Widget::ClutterD => {
                        st.focus = None;
                        if inside {
                            st.push(Cmd::ToggleDouble);
                        }
                    }
                    _ if !inside => {}
                    Widget::Minimize => {
                        self.clicked[1] = true;
                        st.push(Cmd::Minimize);
                    }
                    Widget::Shade => {
                        self.clicked[2] = true;
                        st.push(Cmd::ToggleShade(WinKind::Main));
                    }
                    Widget::Close => {
                        self.clicked[3] = true;
                        st.push(Cmd::Exit);
                    }
                    Widget::ClutterA => {
                        st.focus = None;
                        st.push(Cmd::ToggleAlwaysOnTop);
                    }
                    Widget::ClutterI => {
                        st.focus = None;
                        st.push(Cmd::FileInfo);
                    }
                    Widget::Time => st.push(Cmd::ToggleTimeMode),
                    Widget::Vis => st.push(Cmd::CycleVis),
                    Widget::EqButton => st.push(Cmd::ToggleWindow(WinKind::Eq)),
                    Widget::PlButton => st.push(Cmd::ToggleWindow(WinKind::Playlist)),
                    Widget::Prev => st.push(Cmd::Previous),
                    Widget::Play => st.push(Cmd::Play),
                    Widget::Pause => st.push(Cmd::Pause),
                    Widget::Stop => st.push(Cmd::Stop),
                    Widget::Next => st.push(Cmd::Next),
                    Widget::Eject => st.push(Cmd::Eject),
                    Widget::Shuffle => st.push(Cmd::ToggleShuffle),
                    Widget::Repeat => st.push(Cmd::ToggleRepeat),
                    Widget::About => st.push(Cmd::About),
                    _ => {}
                }
            }
            MouseEvent::Wheel { delta, .. } => {
                st.push(Cmd::VolumeBy(delta * 2));
            }
            _ => {}
        }
        wm
    }

    fn drag_volume(&mut self, st: &mut State, x: i32) {
        let pos = (x - 107 - self.grab).clamp(0, VOLUME_TRAVEL);
        let v = (pos as f64 / VOLUME_TRAVEL as f64 * 100.0).round() as u8;
        if v != st.volume {
            st.push(Cmd::SetVolume(v));
        }
        st.dirty = true;
    }

    fn drag_balance(&mut self, st: &mut State, x: i32) {
        let pos = (x - 177 - self.grab).clamp(0, BALANCE_TRAVEL);
        let mut b = ((pos as f64 / BALANCE_TRAVEL as f64) * 200.0 - 100.0).round() as i32;
        // The centre is sticky.
        if b.abs() < 13 {
            b = 0;
        }
        let b = b.clamp(-100, 100) as i8;
        if b != st.balance {
            st.push(Cmd::SetBalance(b));
        }
        st.dirty = true;
    }

    fn drag_position(&mut self, st: &mut State, x: i32) {
        let Some(d) = st.duration() else { return };
        let (x0, travel) = if st.main.shade { (226, SHADE_POSITION_TRAVEL) } else { (16, POSITION_TRAVEL) };
        let pos = (x - x0 - self.grab).clamp(0, travel);
        let t = pos as f64 / travel as f64 * d;
        st.focus = Some(Focus::Position(t));
        st.dirty = true;
    }

    pub fn wants_animation(&self, st: &State) -> bool {
        st.status() != Status::Stopped || st.marquee_text().chars().count() >= crate::state::MARQUEE_CHARS
    }
}

/// Visualiser background: colour 0 with colour 1 dots on even screen pixels.
fn vis_dots(out: &mut Bitmap, colors: &[u32; 24], x0: i32, y0: i32, w: i32, h: i32) {
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            out.set(x, y, if x % 2 == 0 && y % 2 == 0 { colors[1] } else { colors[0] });
        }
    }
}

/// Where inside a slider thumb the mouse grabbed it: the existing offset when
/// the click lands on the thumb, otherwise centre the thumb on the click.
fn slider_grab(rel_x: i32, thumb_x: i32, thumb_w: i32) -> i32 {
    if rel_x >= thumb_x && rel_x < thumb_x + thumb_w { rel_x - thumb_x } else { thumb_w / 2 }
}

pub fn volume_thumb_x(v: u8) -> i32 {
    (v as f64 / 100.0 * VOLUME_TRAVEL as f64).round() as i32
}

pub fn volume_frame(v: u8) -> i32 {
    (((v as f64 / 100.0) * 28.0).round() as i32 - 1).clamp(0, 27)
}

pub fn balance_thumb_x(b: i8) -> i32 {
    ((b as f64 + 100.0) / 200.0 * BALANCE_TRAVEL as f64).round() as i32
}

pub fn balance_frame(b: i8) -> i32 {
    ((b.unsigned_abs() as f64 / 100.0) * 27.0).floor() as i32
}

fn balance_sprites(st: &State) -> (Sprite, Sprite, Sprite) {
    if !st.skin.has_own(Sheet::Balance) && st.skin.has_own(Sheet::Volume) {
        (
            Sprite { sheet: Sheet::Volume, x: 0, y: 0, w: 38, h: 420 },
            sp::MAIN_VOLUME_THUMB,
            sp::MAIN_VOLUME_THUMB_SELECTED,
        )
    } else {
        (sp::MAIN_BALANCE_BACKGROUND, sp::MAIN_BALANCE_THUMB, sp::MAIN_BALANCE_THUMB_ACTIVE)
    }
}

/// Text in the skin font, clipped to `clip`.
pub fn draw_text_clipped(st: &State, out: &mut Bitmap, text: &str, x: i32, y: i32, clip: Rect) {
    let Some(font) = st.skin.sheet(Sheet::Text) else { return };
    let mut cx = x;
    for c in text.chars() {
        if cx + 5 > clip.x && cx < clip.x + clip.w
            && let Some(s) = sp::text_char(c)
        {
            out.blit_clipped(font, s.x, s.y, s.w, s.h, cx, y, clip);
        }
        cx += 5;
    }
}

/// The 5-character mini time used in window-shade mode and the playlist.
pub fn draw_mini_time(st: &State, out: &mut Bitmap, x: i32, y: i32, visible: bool) {
    let cells = [1, 7, 12, 20, 25];
    for cx in cells {
        draw_text_clipped(st, out, " ", x + cx, y, Rect::new(x + cx, y, 5, 6));
    }
    if st.status() == Status::Stopped || !visible {
        return;
    }
    let remaining = st.time_remaining && st.duration().is_some();
    let secs = if remaining { st.duration().unwrap_or(0.0) - st.position() } else { st.position() };
    let d = time_digits(secs);
    let chars = [if remaining { '-' } else { ' ' }, digit_char(d[0]), digit_char(d[1]), digit_char(d[2]), digit_char(d[3])];
    for (cx, ch) in cells.iter().zip(chars) {
        draw_text_clipped(st, out, &ch.to_string(), x + cx, y, Rect::new(x + cx, y, 5, 6));
    }
}

fn digit_char(d: u32) -> char {
    char::from_digit(d, 10).unwrap_or('0')
}

pub fn draw_vis(st: &mut State, out: &mut Bitmap, x: i32, y: i32, small: bool) {
    if st.status() == Status::Stopped || st.vis_mode == VisMode::Off {
        return;
    }
    let (w, h) = if small { (38, 5) } else { (76, 16) };
    crate::ui::vis::draw_background(out, x, y, w, h, &st.skin.viscolors, !small);
    let colors = st.skin.viscolors;
    let opts = st.vis_opts;
    match st.vis_mode {
        VisMode::Analyzer => st.vis.draw_analyzer(out, x, y, small, &colors, &opts),
        VisMode::Oscilloscope => {
            let samples = std::mem::take(&mut st.vis_samples);
            st.vis.draw_oscilloscope(out, &samples, x, y, small, &colors, &opts);
            st.vis_samples = samples;
        }
        VisMode::Off => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_math() {
        assert_eq!(volume_thumb_x(0), 0);
        assert_eq!(volume_thumb_x(100), 51);
        assert_eq!(balance_thumb_x(0), 12);
        assert_eq!(balance_thumb_x(-100), 0);
        assert_eq!(balance_frame(100), 27);
        assert_eq!(volume_frame(100), 27);
        assert_eq!(volume_frame(78), 21);
        assert_eq!(slider_grab(10, 5, 14), 5);
        assert_eq!(slider_grab(40, 5, 14), 7);
    }
}
