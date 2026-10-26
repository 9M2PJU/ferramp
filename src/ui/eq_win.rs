//! The graphical equalizer window (275x116) and its shade form.

use crate::gfx::{Bitmap, Rect};
use crate::skin::CursorKind;
use crate::skin::sprites as sp;
use crate::state::{Focus, State};
use crate::ui::{Button, Cmd, MenuId, MouseEvent, WinKind, WmRequest};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Widget {
    Shade,
    Close,
    On,
    Auto,
    Presets,
    Preamp,
    Band(usize),
    Max,
    Zero,
    Min,
    ShadeVolume,
    ShadeBalance,
    TitleBar,
    Body,
}

pub const BAND_X: [i32; 10] = [78, 96, 114, 132, 150, 168, 186, 204, 222, 240];
const SLIDER_Y: i32 = 38;
const SLIDER_H: i32 = 63;
const THUMB_TRAVEL: i32 = 51; // 62 - 11

#[derive(Default)]
pub struct EqWin {
    pressed: Option<Widget>,
    hover_inside: bool,
    grab: i32,
    /// Dragging across the band area changes every band the mouse passes.
    sweep: bool,
}

impl EqWin {
    pub fn new() -> EqWin {
        EqWin::default()
    }

    pub fn hit(&self, st: &State, x: i32, y: i32) -> Widget {
        let r = Rect::new;
        if st.eqw.shade {
            return if r(254, 3, 9, 9).contains(x, y) {
                Widget::Shade
            } else if r(264, 3, 9, 9).contains(x, y) {
                Widget::Close
            } else if r(61, 4, 97, 6).contains(x, y) {
                Widget::ShadeVolume
            } else if r(164, 4, 43, 6).contains(x, y) {
                Widget::ShadeBalance
            } else {
                Widget::TitleBar
            };
        }
        let fixed = [
            (Widget::Shade, r(254, 3, 9, 9)),
            (Widget::Close, r(264, 3, 9, 9)),
            (Widget::On, r(14, 18, 26, 12)),
            (Widget::Auto, r(40, 18, 32, 12)),
            (Widget::Presets, r(217, 18, 44, 12)),
            (Widget::Preamp, r(21, SLIDER_Y, 14, SLIDER_H)),
            (Widget::Max, r(45, 36, 22, 8)),
            (Widget::Zero, r(45, 64, 22, 8)),
            (Widget::Min, r(45, 95, 22, 8)),
        ];
        if let Some((w, _)) = fixed.iter().find(|(_, rr)| rr.contains(x, y)) {
            return *w;
        }
        for (i, bx) in BAND_X.iter().enumerate() {
            if r(*bx, SLIDER_Y, 14, SLIDER_H).contains(x, y) {
                return Widget::Band(i);
            }
        }
        if y < 14 { Widget::TitleBar } else { Widget::Body }
    }

    pub fn cursor(&self, st: &State, x: i32, y: i32) -> CursorKind {
        match self.pressed.unwrap_or_else(|| self.hit(st, x, y)) {
            Widget::Close => CursorKind::EqClose,
            Widget::Preamp | Widget::Band(_) => CursorKind::EqSlid,
            Widget::TitleBar | Widget::ShadeVolume | Widget::ShadeBalance => CursorKind::EqTitle,
            _ => CursorKind::EqNormal,
        }
    }

    fn pressed_here(&self, w: Widget) -> bool {
        self.pressed == Some(w) && self.hover_inside
    }

    pub fn render(&mut self, st: &mut State, out: &mut Bitmap) {
        let skin = &st.skin;
        let selected = st.focused == Some(WinKind::Eq);
        if st.eqw.shade {
            skin.draw(out, if selected { sp::EQ_SHADE_BACKGROUND_SELECTED } else { sp::EQ_SHADE_BACKGROUND }, 0, 0);
            // Volume and balance thumbs (left/centre/right variants).
            let vx = (st.volume as f64 / 100.0 * 94.0).round() as i32;
            let vs = if st.volume <= 33 {
                sp::EQ_SHADE_VOLUME_SLIDER_LEFT
            } else if st.volume >= 66 {
                sp::EQ_SHADE_VOLUME_SLIDER_RIGHT
            } else {
                sp::EQ_SHADE_VOLUME_SLIDER_CENTER
            };
            skin.draw(out, vs, 61 + vx, 4);
            let bx = ((st.balance as f64 + 100.0) / 200.0 * 40.0).round() as i32;
            let bs = if st.balance < -33 {
                sp::EQ_SHADE_BALANCE_SLIDER_LEFT
            } else if st.balance > 33 {
                sp::EQ_SHADE_BALANCE_SLIDER_RIGHT
            } else {
                sp::EQ_SHADE_BALANCE_SLIDER_CENTER
            };
            skin.draw(out, bs, 164 + bx, 4);
            if self.pressed_here(Widget::Shade) {
                skin.draw(out, sp::EQ_MINIMIZE_BUTTON_ACTIVE, 254, 3);
            }
            if self.pressed_here(Widget::Close) && selected {
                skin.draw(out, sp::EQ_SHADE_CLOSE_BUTTON_ACTIVE, 264, 3);
            }
            return;
        }
        skin.draw(out, sp::EQ_WINDOW_BACKGROUND, 0, 0);
        skin.draw(out, if selected { sp::EQ_TITLE_BAR_SELECTED } else { sp::EQ_TITLE_BAR }, 0, 0);
        if self.pressed_here(Widget::Shade) {
            if st.skin.has_own(crate::skin::sprites::Sheet::EqEx) || st.skin.path.is_none() {
                skin.draw(out, sp::EQ_MAXIMIZE_BUTTON_ACTIVE, 254, 3);
            } else {
                skin.draw(out, sp::EQ_MAXIMIZE_BUTTON_ACTIVE_FALLBACK, 254, 3);
            }
        }
        if self.pressed_here(Widget::Close) {
            skin.draw(out, sp::EQ_CLOSE_BUTTON_ACTIVE, 264, 3);
        }
        let on = match (st.eq.enabled, self.pressed_here(Widget::On)) {
            (false, false) => sp::EQ_ON_BUTTON,
            (false, true) => sp::EQ_ON_BUTTON_DEPRESSED,
            (true, false) => sp::EQ_ON_BUTTON_SELECTED,
            (true, true) => sp::EQ_ON_BUTTON_SELECTED_DEPRESSED,
        };
        skin.draw(out, on, 14, 18);
        let auto = match (st.eq_auto, self.pressed_here(Widget::Auto)) {
            (false, false) => sp::EQ_AUTO_BUTTON,
            (false, true) => sp::EQ_AUTO_BUTTON_DEPRESSED,
            (true, false) => sp::EQ_AUTO_BUTTON_SELECTED,
            (true, true) => sp::EQ_AUTO_BUTTON_SELECTED_DEPRESSED,
        };
        skin.draw(out, auto, 40, 18);
        skin.draw(out, if self.pressed_here(Widget::Presets) { sp::EQ_PRESETS_BUTTON_SELECTED } else { sp::EQ_PRESETS_BUTTON }, 217, 18);

        // Graph.
        skin.draw(out, sp::EQ_GRAPH_BACKGROUND, 86, 17);
        draw_graph(st, out, 86, 17);

        // Sliders.
        let band = |out: &mut Bitmap, x: i32, value: u8, active: bool| {
            let level = (63 - value.min(63)) as f64 / 63.0;
            let n = (level * 27.0).round() as i32;
            let (fx, fy) = (n % 14, n / 14);
            skin.draw_part(out, sp::EQ_SLIDER_BACKGROUND, fx * 15, fy * 65, 14, 63, x, SLIDER_Y);
            let ty = (value.min(63) as f64 / 63.0 * THUMB_TRAVEL as f64).floor() as i32;
            skin.draw(out, if active { sp::EQ_SLIDER_THUMB_SELECTED } else { sp::EQ_SLIDER_THUMB }, x + 1, SLIDER_Y + ty);
        };
        band(out, 21, st.eq.preamp, self.pressed == Some(Widget::Preamp));
        for (i, bx) in BAND_X.iter().enumerate() {
            band(out, *bx, st.eq.bands[i], self.pressed == Some(Widget::Band(i)));
        }
    }

    pub fn mouse(&mut self, st: &mut State, ev: MouseEvent) -> Vec<WmRequest> {
        let mut wm = Vec::new();
        match ev {
            MouseEvent::Down { x, y, button: Button::Right, .. } => {
                let _ = (x, y);
                st.push(Cmd::ShowMenu(MenuId::Main, x, y, WinKind::Eq));
            }
            MouseEvent::Down { x, y, button: Button::Left, double, .. } => {
                let w = self.hit(st, x, y);
                if double && w == Widget::TitleBar {
                    st.push(Cmd::ToggleShade(WinKind::Eq));
                    return wm;
                }
                match w {
                    Widget::TitleBar | Widget::Body => {
                        wm.push(WmRequest::BeginMove);
                        return wm;
                    }
                    _ => {}
                }
                self.pressed = Some(w);
                self.hover_inside = true;
                match w {
                    Widget::Presets => st.push(Cmd::ShowMenu(MenuId::EqPresets, 217, 30, WinKind::Eq)),
                    Widget::Preamp => {
                        let cur = slider_y(st.eq.preamp);
                        self.grab = grab(y - SLIDER_Y, cur);
                        st.focus = Some(Focus::Eq(None));
                        self.drag_slider(st, None, y);
                    }
                    Widget::Band(i) => {
                        let cur = slider_y(st.eq.bands[i]);
                        self.grab = grab(y - SLIDER_Y, cur);
                        self.sweep = true;
                        st.focus = Some(Focus::Eq(Some(i)));
                        self.drag_slider(st, Some(i), y);
                    }
                    Widget::ShadeVolume => {
                        st.focus = Some(Focus::Volume);
                        self.drag_shade_volume(st, x);
                    }
                    Widget::ShadeBalance => {
                        st.focus = Some(Focus::Balance);
                        self.drag_shade_balance(st, x);
                    }
                    _ => {}
                }
                st.dirty = true;
            }
            MouseEvent::Move { x, y } => {
                let Some(p) = self.pressed else { return wm };
                let now = self.hit(st, x, y);
                let inside = now == p;
                if inside != self.hover_inside {
                    self.hover_inside = inside;
                    st.dirty = true;
                }
                match p {
                    Widget::Preamp => self.drag_slider(st, None, y),
                    Widget::Band(i) => {
                        // Sweeping across bands moves each band to the mouse.
                        if self.sweep
                            && let Widget::Band(j) = now
                            && j != i
                        {
                            self.pressed = Some(Widget::Band(j));
                            self.grab = 5;
                            st.focus = Some(Focus::Eq(Some(j)));
                            self.drag_slider(st, Some(j), y);
                        } else {
                            self.drag_slider(st, Some(i), y);
                        }
                    }
                    Widget::ShadeVolume => self.drag_shade_volume(st, x),
                    Widget::ShadeBalance => self.drag_shade_balance(st, x),
                    _ => {}
                }
            }
            MouseEvent::Up { x, y, button: Button::Left } => {
                let Some(p) = self.pressed.take() else { return wm };
                self.sweep = false;
                let inside = self.hit(st, x, y) == p;
                st.dirty = true;
                match p {
                    Widget::Preamp | Widget::Band(_) | Widget::ShadeVolume | Widget::ShadeBalance => st.focus = None,
                    _ if !inside => {}
                    Widget::Shade => st.push(Cmd::ToggleShade(WinKind::Eq)),
                    Widget::Close => st.push(Cmd::CloseWindow(WinKind::Eq)),
                    Widget::On => st.push(Cmd::EqToggleOn),
                    Widget::Auto => st.push(Cmd::EqToggleAuto),
                    Widget::Max => st.push(Cmd::EqAll(0)),
                    Widget::Zero => st.push(Cmd::EqAll(31)),
                    Widget::Min => st.push(Cmd::EqAll(63)),
                    _ => {}
                }
            }
            MouseEvent::Wheel { delta, .. } => st.push(Cmd::VolumeBy(delta * 2)),
            _ => {}
        }
        wm
    }

    fn drag_slider(&mut self, st: &mut State, band: Option<usize>, y: i32) {
        let pos = (y - SLIDER_Y - self.grab).clamp(0, THUMB_TRAVEL);
        let mut v = (pos as f64 / THUMB_TRAVEL as f64 * 63.0).round() as u8;
        // Snap to the centre line.
        if (29..=34).contains(&v) {
            v = 31;
        }
        match band {
            None => st.push(Cmd::EqSetPreamp(v)),
            Some(i) => st.push(Cmd::EqSetBand(i, v)),
        }
    }

    fn drag_shade_volume(&mut self, st: &mut State, x: i32) {
        let pos = (x - 61 - 1).clamp(0, 94);
        st.push(Cmd::SetVolume((pos as f64 / 94.0 * 100.0).round() as u8));
    }

    fn drag_shade_balance(&mut self, st: &mut State, x: i32) {
        let pos = (x - 164 - 1).clamp(0, 40);
        let mut b = (pos as f64 / 40.0 * 200.0 - 100.0).round() as i32;
        if b.abs() < 13 {
            b = 0;
        }
        st.push(Cmd::SetBalance(b as i8));
    }

    pub fn cancel(&mut self, st: &mut State) {
        if self.pressed.take().is_some() {
            st.focus = None;
            st.dirty = true;
        }
    }
}

fn slider_y(v: u8) -> i32 {
    (v.min(63) as f64 / 63.0 * THUMB_TRAVEL as f64).floor() as i32
}

fn grab(rel_y: i32, thumb_y: i32) -> i32 {
    if rel_y >= thumb_y && rel_y < thumb_y + 11 { rel_y - thumb_y } else { 5 }
}

/// The curve through the ten band values, coloured from eqmain.bmp's colour strip.
pub fn draw_graph(st: &State, out: &mut Bitmap, gx: i32, gy: i32) {
    let skin = &st.skin;
    // Preamp line.
    let pre = ((st.eq.preamp.min(63) as f64 / 63.0) * 18.0).round() as i32;
    skin.draw(out, sp::EQ_PREAMP_LINE, gx, gy + pre);
    let xs: Vec<f64> = (0..10).map(|i| (i * 12) as f64).collect();
    let ys: Vec<f64> = st.eq.bands.iter().map(|&v| v.min(63) as f64 / 63.0 * 18.0).collect();
    let curve = spline(&xs, &ys);
    let mut last = ys[0].round() as i32;
    for (x, yv) in curve.iter().enumerate() {
        let y = (yv.round() as i32).clamp(0, 18);
        let top = y.min(last);
        let h = 1 + (last - y).abs();
        for yy in top..top + h {
            let c = skin.sprite_pixel(sp::EQ_GRAPH_LINE_COLORS, 0, yy);
            out.set(gx + 2 + x as i32, gy + yy, c);
        }
        last = y;
    }
}

/// Natural cubic spline evaluated at every integer x from xs[0] to xs[n-1].
pub fn spline(xs: &[f64], ys: &[f64]) -> Vec<f64> {
    let n = xs.len() - 1;
    // Solve for the derivatives ks of a natural spline (tridiagonal system).
    let mut a = vec![0.0; n + 1];
    let mut b = vec![0.0; n + 1];
    let mut c = vec![0.0; n + 1];
    let mut d = vec![0.0; n + 1];
    for i in 0..=n {
        if i == 0 {
            let h = xs[1] - xs[0];
            b[0] = 2.0 / h;
            c[0] = 1.0 / h;
            d[0] = 3.0 * (ys[1] - ys[0]) / (h * h);
        } else if i == n {
            let h = xs[n] - xs[n - 1];
            a[n] = 1.0 / h;
            b[n] = 2.0 / h;
            d[n] = 3.0 * (ys[n] - ys[n - 1]) / (h * h);
        } else {
            let h0 = xs[i] - xs[i - 1];
            let h1 = xs[i + 1] - xs[i];
            a[i] = 1.0 / h0;
            b[i] = 2.0 * (1.0 / h0 + 1.0 / h1);
            c[i] = 1.0 / h1;
            d[i] = 3.0 * ((ys[i] - ys[i - 1]) / (h0 * h0) + (ys[i + 1] - ys[i]) / (h1 * h1));
        }
    }
    // Thomas algorithm.
    for i in 1..=n {
        let m = a[i] / b[i - 1];
        b[i] -= m * c[i - 1];
        d[i] -= m * d[i - 1];
    }
    let mut k = vec![0.0; n + 1];
    k[n] = d[n] / b[n];
    for i in (0..n).rev() {
        k[i] = (d[i] - c[i] * k[i + 1]) / b[i];
    }
    let max_x = xs[n] as i32;
    let mut out = Vec::with_capacity(max_x as usize + 1);
    let mut i = 1;
    for x in 0..=max_x {
        let xf = x as f64;
        while xs[i] < xf {
            i += 1;
        }
        let h = xs[i] - xs[i - 1];
        let t = (xf - xs[i - 1]) / h;
        let aa = k[i - 1] * h - (ys[i] - ys[i - 1]);
        let bb = -k[i] * h + (ys[i] - ys[i - 1]);
        out.push((1.0 - t) * ys[i - 1] + t * ys[i] + t * (1.0 - t) * (aa * (1.0 - t) + bb * t));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_passes_through_points() {
        let xs: Vec<f64> = (0..10).map(|i| (i * 12) as f64).collect();
        let ys = vec![0.0, 18.0, 9.0, 3.0, 12.0, 9.0, 9.0, 0.0, 18.0, 5.0];
        let c = spline(&xs, &ys);
        assert_eq!(c.len(), 109);
        for (i, y) in ys.iter().enumerate() {
            assert!((c[i * 12] - y).abs() < 1e-9);
        }
        let flat = spline(&xs, &[9.0; 10]);
        assert!(flat.iter().all(|v| (v - 9.0).abs() < 1e-9));
    }
}
