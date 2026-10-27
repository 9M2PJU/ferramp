//! Player state shared by all windows.

use std::time::{Duration, Instant};

use crate::audio::dsp::EqSettings;
use crate::audio::{Engine, Status};
use crate::playlist::{Playlist, format_time};
use crate::skin::Skin;
use crate::text::Fonts;
use crate::ui::vis::{VisMode, VisOptions, Visualizer};
use crate::ui::{Cmd, WinKind};

pub const MARQUEE_CHARS: usize = 31;
pub const MARQUEE_STEP: Duration = Duration::from_millis(220);
pub const PL_SEG_W: i32 = 25;
pub const PL_SEG_H: i32 = 29;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinGeom {
    pub x: i32,
    pub y: i32,
    pub visible: bool,
    pub shade: bool,
}

#[derive(Clone, Debug, Default)]
pub struct NowPlaying {
    pub duration: Option<f64>,
    pub kbps: u32,
    pub khz: u32,
    pub channels: u32,
    pub stream_name: Option<String>,
    pub stream_title: Option<String>,
}

/// Something the user is adjusting; its text replaces the song title.
#[derive(Clone, Debug, PartialEq)]
pub enum Focus {
    Volume,
    Balance,
    Position(f64),
    Eq(Option<usize>),
    Double,
    /// A fixed text, such as the clutter bar's button names.
    Text(&'static str),
}

pub struct State {
    pub engine: Engine,
    pub playlist: Playlist,
    pub skin: Skin,
    pub fonts: Fonts,

    pub volume: u8,
    pub balance: i8,
    pub shuffle: bool,
    pub repeat: bool,
    pub time_remaining: bool,
    pub doubled: bool,
    pub always_on_top: bool,
    pub easy_move: bool,
    pub autoscroll: bool,
    pub vis_mode: VisMode,
    pub vis_opts: VisOptions,
    pub eq: EqSettings,
    pub eq_auto: bool,
    pub stop_after_current: bool,

    pub main: WinGeom,
    pub eqw: WinGeom,
    pub pl: WinGeom,
    /// Extra playlist size in 25x29 segments.
    pub pl_size: (i32, i32),
    pub focused: Option<WinKind>,
    pub llama: bool,

    pub now: NowPlaying,
    /// Opening or buffering a track (shown by the work indicator).
    pub working: bool,
    pub focus: Option<Focus>,
    pub message: Option<(String, Instant)>,
    pub marquee_step: usize,
    pub marquee_drag: i32,
    pub marquee_dragging: bool,
    pub marquee_resume: Instant,
    last_step: Instant,
    pub started: Instant,
    pub error: Option<String>,

    pub vis: Visualizer,
    pub vis_samples: Vec<f32>,
    pub fade: Option<(Instant, u8)>,

    pub queue: Vec<Cmd>,
    pub dirty: bool,
}

impl State {
    pub fn new(engine: Engine, skin: Skin) -> State {
        let now = Instant::now();
        State {
            engine,
            playlist: Playlist::new(),
            skin,
            fonts: Fonts::new(),
            volume: 78,
            balance: 0,
            shuffle: false,
            repeat: false,
            time_remaining: false,
            doubled: false,
            always_on_top: false,
            easy_move: true,
            autoscroll: true,
            vis_mode: VisMode::Analyzer,
            vis_opts: VisOptions::default(),
            eq: EqSettings::default(),
            eq_auto: false,
            stop_after_current: false,
            main: WinGeom {
                x: 100,
                y: 100,
                visible: true,
                shade: false,
            },
            eqw: WinGeom {
                x: 100,
                y: 216,
                visible: true,
                shade: false,
            },
            pl: WinGeom {
                x: 100,
                y: 332,
                visible: true,
                shade: false,
            },
            pl_size: (0, 0),
            focused: Some(WinKind::Main),
            llama: false,
            now: NowPlaying::default(),
            working: false,
            focus: None,
            message: None,
            marquee_step: 0,
            marquee_drag: 0,
            marquee_dragging: false,
            marquee_resume: now,
            last_step: now,
            started: now,
            error: None,
            vis: Visualizer::new(),
            vis_samples: Vec::new(),
            fade: None,
            queue: Vec::new(),
            dirty: true,
        }
    }

    pub fn status(&self) -> Status {
        self.engine.status()
    }

    pub fn geom(&self, w: WinKind) -> &WinGeom {
        match w {
            WinKind::Main => &self.main,
            WinKind::Eq => &self.eqw,
            WinKind::Playlist => &self.pl,
        }
    }

    pub fn geom_mut(&mut self, w: WinKind) -> &mut WinGeom {
        match w {
            WinKind::Main => &mut self.main,
            WinKind::Eq => &mut self.eqw,
            WinKind::Playlist => &mut self.pl,
        }
    }

    /// Window size in skin pixels (before double-size scaling).
    pub fn base_size(&self, w: WinKind) -> (i32, i32) {
        match w {
            WinKind::Main => (275, if self.main.shade { 14 } else { 116 }),
            WinKind::Eq => (275, if self.eqw.shade { 14 } else { 116 }),
            WinKind::Playlist => {
                let width = 275 + self.pl_size.0 * PL_SEG_W;
                (
                    width,
                    if self.pl.shade {
                        14
                    } else {
                        116 + self.pl_size.1 * PL_SEG_H
                    },
                )
            }
        }
    }

    /// Double size applies to the main and equalizer windows.
    pub fn scale_of(&self, w: WinKind) -> i32 {
        if self.doubled && w != WinKind::Playlist {
            2
        } else {
            1
        }
    }

    pub fn position(&self) -> f64 {
        match self.status() {
            Status::Stopped => 0.0,
            _ => {
                let p = self.engine.position();
                match self.now.duration {
                    Some(d) => p.min(d),
                    None => p,
                }
            }
        }
    }

    pub fn duration(&self) -> Option<f64> {
        self.now.duration
    }

    pub fn current_title(&self) -> Option<String> {
        let e = self.playlist.current_entry()?;
        if let Some(t) = &self.now.stream_title
            && self.status() != Status::Stopped
        {
            return Some(t.clone());
        }
        Some(e.title.clone())
    }

    /// "N. Title (m:ss)" as scrolled in the main window.
    pub fn media_text(&self) -> Option<String> {
        let idx = self.playlist.current?;
        let title = self.current_title()?;
        let dur = self.playlist.current_entry().and_then(|e| e.duration).or(
            if self.status() != Status::Stopped {
                self.now.duration
            } else {
                None
            },
        );
        Some(match dur {
            Some(d) => format!("{}. {} ({})", idx + 1, title, format_time(d)),
            None => format!("{}. {}", idx + 1, title),
        })
    }

    pub fn marquee_text(&self) -> String {
        if let Some((m, until)) = &self.message
            && Instant::now() < *until
        {
            return m.clone();
        }
        match &self.focus {
            Some(Focus::Volume) => return crate::ui::volume_text(self.volume),
            Some(Focus::Balance) => return crate::ui::balance_text(self.balance),
            Some(Focus::Position(p)) => {
                if let Some(d) = self.duration() {
                    let pct = if d > 0.0 {
                        (p / d * 100.0).round() as i32
                    } else {
                        0
                    };
                    return format!("Seek to: {}/{} ({}%)", format_time(*p), format_time(d), pct);
                }
            }
            Some(Focus::Eq(band)) => {
                let v = match band {
                    None => self.eq.preamp,
                    Some(i) => self.eq.bands[*i],
                };
                return crate::ui::eq_text(*band, v);
            }
            Some(Focus::Double) => {
                return format!(
                    "{} doublesize mode",
                    if self.doubled { "Disable" } else { "Enable" }
                );
            }
            Some(Focus::Text(t)) => {
                if *t == "always-on-top" {
                    return format!(
                        "{} always-on-top",
                        if self.always_on_top {
                            "Disable"
                        } else {
                            "Enable"
                        }
                    );
                }
                return t.to_string();
            }
            None => {}
        }
        if let Some(e) = &self.error
            && self.status() == Status::Stopped
        {
            return e.clone();
        }
        self.media_text()
            .unwrap_or_else(|| concat!("Ferramp ", env!("CARGO_PKG_VERSION")).to_string())
    }

    pub fn show_message(&mut self, text: impl Into<String>, secs: f64) {
        self.message = Some((text.into(), Instant::now() + Duration::from_secs_f64(secs)));
        self.dirty = true;
    }

    /// Advance timers; returns true if something visible changed.
    pub fn tick(&mut self, now: Instant) -> bool {
        let mut changed = false;
        if self.autoscroll
            && !self.marquee_dragging
            && now >= self.marquee_resume
            && now.duration_since(self.last_step) >= MARQUEE_STEP
        {
            self.last_step = now;
            self.marquee_step = self.marquee_step.wrapping_add(1);
            changed = true;
        }
        if let Some((_, until)) = &self.message
            && now >= *until
        {
            self.message = None;
            changed = true;
        }
        changed
    }

    /// Pixel offset of the scrolling title.
    pub fn marquee_offset(&self, text: &str) -> i32 {
        let len = text.chars().count();
        if len < MARQUEE_CHARS {
            return 0;
        }
        let cycle = ((len + 7) * 5) as i64;
        let off = self.marquee_step as i64 * 5 + self.marquee_drag as i64;
        (((off % cycle) + cycle) % cycle) as i32
    }

    pub fn push(&mut self, cmd: Cmd) {
        self.queue.push(cmd);
    }
}

/// Text to scroll: long titles loop with a separator, short ones are padded.
pub fn marquee_loop_text(text: &str) -> String {
    if text.chars().count() >= MARQUEE_CHARS {
        format!("{text}  ***  {text}")
    } else {
        let mut s = text.to_string();
        while s.chars().count() < MARQUEE_CHARS {
            s.push(' ');
        }
        s
    }
}

/// Digits for the time display: (minus, [m1, m2, s1, s2]).
pub fn time_digits(secs: f64) -> [u32; 4] {
    let s = secs.max(0.0).floor() as u64;
    let mut m = s / 60;
    if m > 99 {
        m %= 100;
    }
    let sec = s % 60;
    [
        (m / 10) as u32,
        (m % 10) as u32,
        (sec / 10) as u32,
        (sec % 10) as u32,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits() {
        assert_eq!(time_digits(0.0), [0, 0, 0, 0]);
        assert_eq!(time_digits(754.9), [1, 2, 3, 4]);
    }

    #[test]
    fn marquee_padding() {
        assert_eq!(marquee_loop_text("abc").chars().count(), MARQUEE_CHARS);
        let long = "x".repeat(40);
        assert_eq!(marquee_loop_text(&long), format!("{long}  ***  {long}"));
    }
}
