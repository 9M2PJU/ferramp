//! Settings persistence (an INI file like the original's) and EQF preset files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::audio::dsp::EqSettings;

pub fn config_dir() -> PathBuf {
    if let Ok(d) = std::env::var("FERRAMP_CONFIG_DIR")
        && !d.is_empty()
    {
        return PathBuf::from(d);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ferramp")
}

pub fn skins_dir() -> PathBuf {
    config_dir().join("Skins")
}

/// Flat key/value settings with typed accessors.
#[derive(Default, Clone, Debug)]
pub struct Settings {
    pub values: BTreeMap<String, String>,
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        let mut s = Settings::default();
        if let Ok(text) = std::fs::read_to_string(path) {
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with('[') || line.starts_with(';') || line.is_empty() {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    s.values.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        s
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut out = String::from("[Ferramp]\n");
        for (k, v) in &self.values {
            out.push_str(&format!("{k}={v}\n"));
        }
        let tmp = path.with_extension("ini.tmp");
        std::fs::write(&tmp, out)?;
        std::fs::rename(tmp, path)
    }

    pub fn get_i32(&self, k: &str, default: i32) -> i32 {
        self.values
            .get(k)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub fn get_bool(&self, k: &str, default: bool) -> bool {
        self.values
            .get(k)
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(default)
    }

    pub fn get_str(&self, k: &str) -> Option<&str> {
        self.values
            .get(k)
            .map(|s| s.as_str())
            .filter(|s| !s.is_empty())
    }

    pub fn set(&mut self, k: &str, v: impl ToString) {
        self.values.insert(k.to_string(), v.to_string());
    }

    pub fn set_bool(&mut self, k: &str, v: bool) {
        self.set(k, if v { 1 } else { 0 });
    }

    pub fn eq(&self) -> EqSettings {
        let mut e = EqSettings {
            enabled: self.get_bool("eq_enabled", false),
            ..Default::default()
        };
        e.preamp = self.get_i32("eq_preamp", 31).clamp(0, 63) as u8;
        if let Some(b) = self.get_str("eq_bands") {
            for (i, v) in b.split(',').take(10).enumerate() {
                if let Ok(n) = v.trim().parse::<u8>() {
                    e.bands[i] = n.min(63);
                }
            }
        }
        e
    }

    pub fn set_eq(&mut self, e: &EqSettings) {
        self.set_bool("eq_enabled", e.enabled);
        self.set("eq_preamp", e.preamp);
        self.set(
            "eq_bands",
            e.bands
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
    }
}

const EQF_HEADER: &[u8] = b"Winamp EQ library file v1.1\x1a!--";

/// Parse an .eqf / .q1 library: (name, preamp, bands) with 0 = +12 dB.
pub fn parse_eqf(data: &[u8]) -> Option<Vec<(String, u8, [u8; 10])>> {
    let rest = data.strip_prefix(EQF_HEADER)?;
    let mut out = Vec::new();
    for chunk in rest.chunks(257 + 11) {
        if chunk.len() < 257 + 11 {
            break;
        }
        let name_end = chunk[..257].iter().position(|&b| b == 0).unwrap_or(257);
        let name: String = chunk[..name_end].iter().map(|&b| b as char).collect();
        let v = &chunk[257..];
        let mut bands = [31u8; 10];
        for i in 0..10 {
            bands[i] = v[i].min(63);
        }
        out.push((name, v[10].min(63), bands));
    }
    Some(out)
}

pub fn write_eqf(name: &str, preamp: u8, bands: &[u8; 10]) -> Vec<u8> {
    let mut out = EQF_HEADER.to_vec();
    let mut n = [0u8; 257];
    for (i, b) in name.bytes().take(256).enumerate() {
        n[i] = b;
    }
    out.extend_from_slice(&n);
    out.extend(bands.iter().map(|b| (*b).min(63)));
    out.push(preamp.min(63));
    out
}

/// Write a whole preset library (several presets) in EQF format.
pub fn write_eqf_library(presets: &[(String, u8, [u8; 10])]) -> Vec<u8> {
    let mut out = EQF_HEADER.to_vec();
    for (name, preamp, bands) in presets {
        out.extend_from_slice(&write_eqf(name, *preamp, bands)[EQF_HEADER.len()..]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eqf_round_trip() {
        let bytes = write_eqf("Mine", 20, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 63]);
        let p = parse_eqf(&bytes).unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].0, "Mine");
        assert_eq!(p[0].1, 20);
        assert_eq!(p[0].2[9], 63);
        assert!(parse_eqf(b"nope").is_none());
        let lib = write_eqf_library(&[("A".into(), 1, [2; 10]), ("B".into(), 3, [4; 10])]);
        let p = parse_eqf(&lib).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[1].0, "B");
    }

    #[test]
    fn settings_round_trip() {
        let dir = std::env::temp_dir().join(format!("ferramp-test-{}", std::process::id()));
        let path = dir.join("ferramp.ini");
        let mut s = Settings::default();
        s.set("volume", 42);
        s.set_bool("shuffle", true);
        let mut e = EqSettings::default();
        e.bands[3] = 7;
        s.set_eq(&e);
        s.save(&path).unwrap();
        let l = Settings::load(&path);
        assert_eq!(l.get_i32("volume", 0), 42);
        assert!(l.get_bool("shuffle", false));
        assert_eq!(l.eq().bands[3], 7);
        let _ = std::fs::remove_dir_all(dir);
    }
}
