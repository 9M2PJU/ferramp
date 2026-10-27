//! Tag and duration reading for playlist entries, done on a worker thread.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use lofty::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct TrackInfo {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub track_no: Option<u32>,
    pub duration: Option<f64>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
}

impl TrackInfo {
    /// "Artist - Title", falling back to whichever part exists.
    pub fn display_title(&self) -> Option<String> {
        match (&self.artist, &self.title) {
            (Some(a), Some(t)) if !a.is_empty() && !t.is_empty() => Some(format!("{a} - {t}")),
            (_, Some(t)) if !t.is_empty() => Some(t.clone()),
            _ => None,
        }
    }
}

pub fn read(path: &Path) -> Option<TrackInfo> {
    let tagged = lofty::read_from_path(path).ok()?;
    let props = tagged.properties();
    let mut info = TrackInfo {
        duration: Some(props.duration().as_secs_f64()).filter(|d| *d > 0.0),
        bitrate: props.audio_bitrate().or(props.overall_bitrate()),
        sample_rate: props.sample_rate(),
        channels: props.channels(),
        ..Default::default()
    };
    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
        let clean = |s: Option<std::borrow::Cow<'_, str>>| {
            s.map(|v| v.trim().trim_matches('\0').to_string())
                .filter(|v| !v.is_empty())
        };
        info.artist = clean(tag.artist());
        info.title = clean(tag.title());
        info.album = clean(tag.album());
        info.genre = clean(tag.genre());
        info.comment = clean(tag.comment());
        info.track_no = tag.track();
        info.year = tag.date().map(|d| d.year as u32);
    }
    Some(info)
}

/// Background reader: send (id, path), receive (id, info).
pub struct InfoWorker {
    tx: Sender<(u64, PathBuf)>,
    pub rx: Receiver<(u64, Option<TrackInfo>)>,
}

impl InfoWorker {
    pub fn new(wake: impl Fn() + Send + 'static) -> InfoWorker {
        let (tx, jobs) = mpsc::channel::<(u64, PathBuf)>();
        let (done_tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("tag-reader".into())
            .spawn(move || {
                while let Ok((id, path)) = jobs.recv() {
                    let info = read(&path);
                    if done_tx.send((id, info)).is_err() {
                        break;
                    }
                    wake();
                }
            })
            .expect("spawn tag reader");
        InfoWorker { tx, rx }
    }

    pub fn request(&self, id: u64, path: PathBuf) {
        let _ = self.tx.send((id, path));
    }
}

/// File extensions the player lists when adding folders or filtering dialogs.
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "mp2", "mp1", "ogg", "oga", "flac", "wav", "wave", "aif", "aiff", "aifc", "m4a", "m4b",
    "mp4", "aac", "alac", "caf", "mka", "webm",
];

pub fn is_audio_file(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn is_playlist_file(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "m3u" | "m3u8" | "pls"))
        .unwrap_or(false)
}
