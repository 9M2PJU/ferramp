//! Playlist model: entries, selection, ordering, shuffle and playlist files.

use std::path::{Path, PathBuf};

use crate::audio::Source;

#[derive(Clone, Debug)]
pub struct Entry {
    pub id: u64,
    pub source: Source,
    /// Title as shown in the list ("Artist - Title" or the file name).
    pub title: String,
    /// True once tags were read (or a playlist file provided a title).
    pub has_info: bool,
    pub duration: Option<f64>,
    pub selected: bool,
}

impl Entry {
    pub fn path(&self) -> Option<&Path> {
        match &self.source {
            Source::File(p) => Some(p),
            Source::Url(_) => None,
        }
    }

    pub fn location(&self) -> String {
        match &self.source {
            Source::File(p) => p.to_string_lossy().into_owned(),
            Source::Url(u) => u.clone(),
        }
    }
}

/// Remove line breaks and other control characters (tags and stream
/// headers are untrusted and titles end up in playlist files).
pub fn clean_title(t: &str) -> String {
    t.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn default_title(source: &Source) -> String {
    match source {
        Source::File(p) => p
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| p.to_string_lossy().into_owned()),
        Source::Url(u) => u.clone(),
    }
}

#[derive(Default)]
pub struct Playlist {
    pub entries: Vec<Entry>,
    /// Index of the entry being played (or that will play on "Play").
    pub current: Option<usize>,
    /// First visible row in the editor.
    pub scroll: usize,
    anchor: Option<usize>,
    next_id: u64,
    /// Shuffle order (indices) and our position in it.
    shuffle_order: Vec<usize>,
    shuffle_pos: usize,
    rng: u64,
}

impl Playlist {
    pub fn new() -> Playlist {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9);
        Playlist {
            rng: seed | 1,
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn rand(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn make_entry(
        &mut self,
        source: Source,
        title: Option<String>,
        duration: Option<f64>,
    ) -> Entry {
        self.next_id += 1;
        let has_info = title.is_some();
        Entry {
            id: self.next_id,
            title: clean_title(&title.unwrap_or_else(|| default_title(&source))),
            has_info,
            source,
            duration,
            selected: false,
        }
    }

    /// Insert entries at `at` (clamped). Returns the ids of the new entries.
    pub fn insert(
        &mut self,
        at: usize,
        items: Vec<(Source, Option<String>, Option<f64>)>,
    ) -> Vec<u64> {
        let at = at.min(self.entries.len());
        let new: Vec<Entry> = items
            .into_iter()
            .map(|(s, t, d)| self.make_entry(s, t, d))
            .collect();
        let ids = new.iter().map(|e| e.id).collect();
        let n = new.len();
        self.entries.splice(at..at, new);
        if let Some(c) = self.current
            && c >= at
        {
            self.current = Some(c + n);
        }
        if self.current.is_none() && !self.entries.is_empty() {
            self.current = Some(0);
        }
        self.invalidate_shuffle();
        ids
    }

    pub fn append(&mut self, items: Vec<(Source, Option<String>, Option<f64>)>) -> Vec<u64> {
        let at = self.entries.len();
        self.insert(at, items)
    }

    pub fn index_of(&self, id: u64) -> Option<usize> {
        self.entries.iter().position(|e| e.id == id)
    }

    pub fn set_info(&mut self, id: u64, title: Option<String>, duration: Option<f64>) {
        if let Some(i) = self.index_of(id) {
            let e = &mut self.entries[i];
            if let Some(t) = title {
                e.title = clean_title(&t);
            }
            if duration.is_some() {
                e.duration = duration;
            }
            e.has_info = true;
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current = None;
        self.scroll = 0;
        self.anchor = None;
        self.invalidate_shuffle();
    }

    fn retain(&mut self, mut keep: impl FnMut(&Entry) -> bool) {
        let current_id = self.current.and_then(|c| self.entries.get(c)).map(|e| e.id);
        let before = self.current;
        self.entries.retain(|e| keep(e));
        self.current = match current_id.and_then(|id| self.index_of(id)) {
            Some(i) => Some(i),
            None if self.entries.is_empty() => None,
            None => Some(before.unwrap_or(0).min(self.entries.len() - 1)),
        };
        self.anchor = None;
        let max_scroll = self.entries.len().saturating_sub(1);
        self.scroll = self.scroll.min(max_scroll);
        self.invalidate_shuffle();
    }

    pub fn remove_selected(&mut self) {
        self.retain(|e| !e.selected);
    }

    pub fn crop(&mut self) {
        self.retain(|e| e.selected);
    }

    pub fn remove_dead_files(&mut self) {
        self.retain(|e| match &e.source {
            Source::File(p) => p.exists(),
            Source::Url(_) => true,
        });
    }

    pub fn select_all(&mut self) {
        self.entries.iter_mut().for_each(|e| e.selected = true);
    }

    pub fn select_none(&mut self) {
        self.entries.iter_mut().for_each(|e| e.selected = false);
    }

    pub fn invert_selection(&mut self) {
        self.entries
            .iter_mut()
            .for_each(|e| e.selected = !e.selected);
    }

    pub fn selected_count(&self) -> usize {
        self.entries.iter().filter(|e| e.selected).count()
    }

    /// Plain click: select only this row.
    pub fn click(&mut self, i: usize) {
        if i >= self.entries.len() {
            return;
        }
        self.select_none();
        self.entries[i].selected = true;
        self.anchor = Some(i);
    }

    pub fn ctrl_click(&mut self, i: usize) {
        if let Some(e) = self.entries.get_mut(i) {
            e.selected = !e.selected;
            self.anchor = Some(i);
        }
    }

    pub fn shift_click(&mut self, i: usize) {
        if i >= self.entries.len() {
            return;
        }
        let a = self.anchor.unwrap_or(i);
        let (lo, hi) = if a <= i { (a, i) } else { (i, a) };
        for (j, e) in self.entries.iter_mut().enumerate() {
            e.selected = j >= lo && j <= hi;
        }
    }

    /// Move the selected entries by `delta` rows, keeping their relative order.
    /// Returns the delta actually applied.
    pub fn move_selected(&mut self, delta: isize) -> isize {
        let sel: Vec<usize> = (0..self.entries.len())
            .filter(|&i| self.entries[i].selected)
            .collect();
        if sel.is_empty() || delta == 0 {
            return 0;
        }
        let first = sel[0] as isize;
        let last = *sel.last().unwrap() as isize;
        let n = self.entries.len() as isize;
        let delta = delta.clamp(-first, n - 1 - last);
        if delta == 0 {
            return 0;
        }
        let current_id = self.current.and_then(|c| self.entries.get(c)).map(|e| e.id);
        let mut order: Vec<Option<Entry>> = self.entries.drain(..).map(Some).collect();
        let mut result: Vec<Option<Entry>> = vec![None; order.len()];
        for &i in &sel {
            let j = (i as isize + delta) as usize;
            result[j] = order[i].take();
        }
        let mut rest = order.into_iter().flatten();
        for slot in result.iter_mut() {
            if slot.is_none() {
                *slot = rest.next();
            }
        }
        self.entries = result.into_iter().flatten().collect();
        self.current = current_id.and_then(|id| self.index_of(id));
        if let Some(a) = self.anchor {
            self.anchor = Some((a as isize + delta).clamp(0, n - 1) as usize);
        }
        self.invalidate_shuffle();
        delta
    }

    fn reorder(&mut self, mut f: impl FnMut(&mut Vec<Entry>)) {
        let current_id = self.current.and_then(|c| self.entries.get(c)).map(|e| e.id);
        f(&mut self.entries);
        self.current = current_id.and_then(|id| self.index_of(id));
        self.anchor = None;
        self.invalidate_shuffle();
    }

    pub fn reverse(&mut self) {
        self.reorder(|v| v.reverse());
    }

    pub fn randomize(&mut self) {
        let mut seeds: Vec<u64> = (0..self.entries.len()).map(|_| self.rand()).collect();
        self.reorder(|v| {
            let mut tagged: Vec<(u64, Entry)> = seeds.drain(..).zip(v.drain(..)).collect();
            tagged.sort_by_key(|t| t.0);
            v.extend(tagged.into_iter().map(|t| t.1));
        });
    }

    pub fn sort_by_title(&mut self) {
        self.reorder(|v| v.sort_by_cached_key(|e| e.title.to_lowercase()));
    }

    pub fn sort_by_filename(&mut self) {
        self.reorder(|v| {
            v.sort_by_cached_key(|e| match &e.source {
                Source::File(p) => p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default(),
                Source::Url(u) => u.to_lowercase(),
            })
        });
    }

    pub fn sort_by_path(&mut self) {
        self.reorder(|v| v.sort_by_cached_key(|e| e.location().to_lowercase()));
    }

    pub fn total_duration(&self) -> (f64, bool) {
        let mut total = 0.0;
        let mut complete = true;
        for e in &self.entries {
            match e.duration {
                Some(d) => total += d,
                None => complete = false,
            }
        }
        (total, complete)
    }

    pub fn selected_duration(&self) -> (f64, bool) {
        let mut total = 0.0;
        let mut complete = true;
        for e in self.entries.iter().filter(|e| e.selected) {
            match e.duration {
                Some(d) => total += d,
                None => complete = false,
            }
        }
        (total, complete)
    }

    fn invalidate_shuffle(&mut self) {
        self.shuffle_order.clear();
        self.shuffle_pos = 0;
    }

    fn ensure_shuffle(&mut self) {
        if self.shuffle_order.len() == self.entries.len() {
            return;
        }
        let mut order: Vec<usize> = (0..self.entries.len()).collect();
        for i in (1..order.len()).rev() {
            let j = (self.rand() % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        // Start the order at the current track.
        if let Some(c) = self.current
            && let Some(p) = order.iter().position(|&x| x == c)
        {
            order.swap(0, p);
        }
        self.shuffle_order = order;
        self.shuffle_pos = 0;
    }

    /// Index after the current one, honouring shuffle and repeat.
    /// `None` means the end of the list was reached without repeat.
    pub fn next_index(&mut self, shuffle: bool, repeat: bool) -> Option<usize> {
        let n = self.entries.len();
        if n == 0 {
            return None;
        }
        if shuffle {
            self.ensure_shuffle();
            if let Some(c) = self.current
                && self.shuffle_order.get(self.shuffle_pos) != Some(&c)
                && let Some(p) = self.shuffle_order.iter().position(|&x| x == c)
            {
                self.shuffle_pos = p;
            }
            if self.shuffle_pos + 1 < n {
                self.shuffle_pos += 1;
                return Some(self.shuffle_order[self.shuffle_pos]);
            }
            if !repeat {
                return None;
            }
            self.invalidate_shuffle();
            self.ensure_shuffle();
            return self.shuffle_order.first().copied();
        }
        match self.current {
            None => Some(0),
            Some(c) if c + 1 < n => Some(c + 1),
            Some(_) if repeat => Some(0),
            Some(_) => None,
        }
    }

    pub fn prev_index(&mut self, shuffle: bool) -> Option<usize> {
        let n = self.entries.len();
        if n == 0 {
            return None;
        }
        if shuffle {
            self.ensure_shuffle();
            if self.shuffle_pos > 0 {
                self.shuffle_pos -= 1;
                return Some(self.shuffle_order[self.shuffle_pos]);
            }
            return self.current.or(Some(0));
        }
        match self.current {
            None => Some(0),
            Some(0) => Some(0),
            Some(c) => Some(c - 1),
        }
    }

    pub fn current_entry(&self) -> Option<&Entry> {
        self.current.and_then(|c| self.entries.get(c))
    }

    // -- playlist files ----------------------------------------------------

    pub fn save_m3u(&self, path: &Path) -> std::io::Result<()> {
        let base = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let mut s = String::from("#EXTM3U\n");
        for e in &self.entries {
            // A path with a line break cannot be represented in M3U.
            if e.location().contains(['\n', '\r']) {
                continue;
            }
            let secs = e.duration.map(|d| d.round() as i64).unwrap_or(-1);
            s.push_str(&format!("#EXTINF:{},{}\n", secs, clean_title(&e.title)));
            match &e.source {
                Source::File(p) => {
                    let rel = p
                        .strip_prefix(&base)
                        .map(|r| r.to_path_buf())
                        .unwrap_or_else(|_| p.clone());
                    s.push_str(&rel.to_string_lossy());
                }
                Source::Url(u) => s.push_str(&clean_title(u)),
            }
            s.push('\n');
        }
        std::fs::write(path, s)
    }

    pub fn save_pls(&self, path: &Path) -> std::io::Result<()> {
        let mut s = String::from("[playlist]\n");
        for (i, e) in self.entries.iter().enumerate() {
            let n = i + 1;
            s.push_str(&format!("File{n}={}\n", clean_title(&e.location())));
            s.push_str(&format!("Title{n}={}\n", clean_title(&e.title)));
            s.push_str(&format!(
                "Length{n}={}\n",
                e.duration.map(|d| d.round() as i64).unwrap_or(-1)
            ));
        }
        s.push_str(&format!(
            "NumberOfEntries={}\nVersion=2\n",
            self.entries.len()
        ));
        std::fs::write(path, s)
    }
}

/// Read an M3U/M3U8/PLS file into (source, title, duration) items.
pub fn load_playlist_file(path: &Path) -> Vec<(Source, Option<String>, Option<f64>)> {
    let Ok(bytes) = std::fs::read(path) else {
        return vec![];
    };
    let text = crate::skin::decode_text(&bytes);
    let base = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ext == "pls"
        || text
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("[playlist]")
    {
        parse_pls(&text, &base)
    } else {
        parse_m3u(&text, &base)
    }
}

fn resolve(loc: &str, base: &Path) -> Source {
    let src = Source::parse(loc.trim());
    match src {
        Source::File(p) if p.is_relative() => {
            // Playlists written on Windows use backslashes.
            let fixed: PathBuf = loc.trim().replace('\\', "/").into();
            Source::File(base.join(fixed))
        }
        other => other,
    }
}

pub fn parse_m3u(text: &str, base: &Path) -> Vec<(Source, Option<String>, Option<f64>)> {
    let mut out = Vec::new();
    let mut pending: Option<(Option<f64>, Option<String>)> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("#EXTINF:") {
            let (dur, title) = rest.split_once(',').unwrap_or((rest, ""));
            let d = dur
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| *v >= 0.0);
            let t = Some(title.trim().to_string()).filter(|t| !t.is_empty());
            pending = Some((d, t));
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let (d, t) = pending.take().unwrap_or((None, None));
        out.push((resolve(line, base), t, d));
    }
    out
}

pub fn parse_pls(text: &str, base: &Path) -> Vec<(Source, Option<String>, Option<f64>)> {
    let ini = crate::skin::parse_ini(text);
    let Some(sec) = ini.get("playlist") else {
        return vec![];
    };
    let mut idx: Vec<usize> = sec
        .keys()
        .filter_map(|k| k.strip_prefix("file").and_then(|n| n.parse().ok()))
        .collect();
    idx.sort_unstable();
    idx.into_iter()
        .filter_map(|n| {
            let loc = sec.get(&format!("file{n}"))?;
            let title = sec
                .get(&format!("title{n}"))
                .cloned()
                .filter(|t| !t.is_empty());
            let dur = sec
                .get(&format!("length{n}"))
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| *v >= 0.0);
            Some((resolve(loc, base), title, dur))
        })
        .collect()
}

/// Format seconds as "m:ss" (minutes not capped), the way list durations are shown.
pub fn format_time(secs: f64) -> String {
    let s = secs.max(0.0).floor() as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(names: &[&str]) -> Vec<(Source, Option<String>, Option<f64>)> {
        names
            .iter()
            .map(|n| {
                (
                    Source::File(PathBuf::from(format!("/m/{n}.mp3"))),
                    None,
                    Some(10.0),
                )
            })
            .collect()
    }

    fn titles(p: &Playlist) -> Vec<String> {
        p.entries.iter().map(|e| e.title.clone()).collect()
    }

    #[test]
    fn insert_and_titles() {
        let mut p = Playlist::new();
        p.append(files(&["a", "b"]));
        p.insert(1, files(&["x"]));
        assert_eq!(titles(&p), ["a", "x", "b"]);
        assert_eq!(p.current, Some(0));
    }

    #[test]
    fn selection_and_removal_keep_current() {
        let mut p = Playlist::new();
        p.append(files(&["a", "b", "c", "d"]));
        p.current = Some(2);
        p.click(0);
        p.shift_click(1);
        assert_eq!(p.selected_count(), 2);
        p.remove_selected();
        assert_eq!(titles(&p), ["c", "d"]);
        assert_eq!(p.current, Some(0));
        p.ctrl_click(1);
        p.crop();
        assert_eq!(titles(&p), ["d"]);
    }

    #[test]
    fn move_selected_block() {
        let mut p = Playlist::new();
        p.append(files(&["a", "b", "c", "d", "e"]));
        p.entries[1].selected = true;
        p.entries[2].selected = true;
        assert_eq!(p.move_selected(2), 2);
        assert_eq!(titles(&p), ["a", "d", "e", "b", "c"]);
        assert_eq!(p.move_selected(5), 0);
        assert_eq!(p.move_selected(-10), -3);
        assert_eq!(titles(&p), ["b", "c", "a", "d", "e"]);
    }

    #[test]
    fn next_prev_repeat_and_shuffle() {
        let mut p = Playlist::new();
        p.append(files(&["a", "b", "c"]));
        p.current = Some(2);
        assert_eq!(p.next_index(false, false), None);
        assert_eq!(p.next_index(false, true), Some(0));
        assert_eq!(p.prev_index(false), Some(1));
        // Shuffle visits every track once before repeating.
        p.current = Some(0);
        let mut seen = vec![0];
        for _ in 0..2 {
            let n = p.next_index(true, false).unwrap();
            seen.push(n);
            p.current = Some(n);
        }
        seen.sort();
        assert_eq!(seen, [0, 1, 2]);
        assert_eq!(p.next_index(true, false), None);
        assert!(p.next_index(true, true).is_some());
    }

    #[test]
    fn m3u_and_pls_round_trip() {
        let m3u =
            "#EXTM3U\n#EXTINF:123,Artist - Song\nsongs\\one.mp3\nhttp://radio.example/stream\n";
        let items = parse_m3u(m3u, Path::new("/music"));
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].0,
            Source::File(PathBuf::from("/music/songs/one.mp3"))
        );
        assert_eq!(items[0].1.as_deref(), Some("Artist - Song"));
        assert_eq!(items[0].2, Some(123.0));
        assert_eq!(
            items[1].0,
            Source::Url("http://radio.example/stream".into())
        );

        let pls = "[playlist]\nFile1=/a.mp3\nTitle1=A\nLength1=5\nFile2=b.ogg\nNumberOfEntries=2\n";
        let items = parse_pls(pls, Path::new("/x"));
        assert_eq!(items[1].0, Source::File(PathBuf::from("/x/b.ogg")));
        assert_eq!(items[0].2, Some(5.0));
    }

    #[test]
    fn titles_are_single_line() {
        assert_eq!(clean_title("a\r\nhttp://evil\tb"), "a  http://evil b");
        let mut p = Playlist::new();
        p.append(vec![(
            Source::File(PathBuf::from("/x.mp3")),
            Some("A\nB".into()),
            None,
        )]);
        assert_eq!(p.entries[0].title, "A B");
    }

    #[test]
    fn time_format() {
        assert_eq!(format_time(0.0), "0:00");
        assert_eq!(format_time(61.9), "1:01");
        assert_eq!(format_time(3725.0), "62:05");
    }
}
