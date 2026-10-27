//! The classic spectrum analyser and oscilloscope (76x16 in the main window,
//! 38x5 in window-shade mode). Behaviour follows the analysis of the original
//! visualiser published by the Webamp project (MIT): a 1024-point FFT with an
//! envelope and log-equalisation, bars with falloff and falling peaks.

use std::f32::consts::PI;

use crate::gfx::Bitmap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisMode {
    Analyzer,
    Oscilloscope,
    Off,
}

impl VisMode {
    pub fn next(self) -> VisMode {
        match self {
            VisMode::Analyzer => VisMode::Oscilloscope,
            VisMode::Oscilloscope => VisMode::Off,
            VisMode::Off => VisMode::Analyzer,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            VisMode::Analyzer => "analyzer",
            VisMode::Oscilloscope => "oscilloscope",
            VisMode::Off => "off",
        }
    }

    pub fn parse(s: &str) -> VisMode {
        match s {
            "oscilloscope" => VisMode::Oscilloscope,
            "off" => VisMode::Off,
            _ => VisMode::Analyzer,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalyzerStyle {
    Normal,
    Fire,
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscStyle {
    Dots,
    Lines,
    Solid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisOptions {
    pub analyzer: AnalyzerStyle,
    pub osc: OscStyle,
    pub peaks: bool,
    pub wide_bands: bool,
    /// Bar falloff speed index 0..=4 (slower..faster).
    pub falloff: u8,
    /// Peak falloff speed index 0..=4.
    pub peak_falloff: u8,
}

impl Default for VisOptions {
    fn default() -> Self {
        VisOptions {
            analyzer: AnalyzerStyle::Normal,
            osc: OscStyle::Lines,
            peaks: true,
            wide_bands: true,
            falloff: 2,
            peak_falloff: 1,
        }
    }
}

struct Fft {
    bitrev: Vec<usize>,
    envelope: Vec<f32>,
    equalize: Vec<f32>,
    cossin: Vec<(f32, f32)>,
    re: Vec<f32>,
    im: Vec<f32>,
}

impl Fft {
    fn new() -> Fft {
        let samples_in = 1024usize;
        let nfreq = 1024usize;
        let mut bitrev: Vec<usize> = (0..nfreq).collect();
        let mut j = 0usize;
        for i in 0..nfreq {
            if j > i {
                bitrev.swap(i, j);
            }
            let mut m = nfreq >> 1;
            while m >= 1 && j >= m {
                j -= m;
                m >>= 1;
            }
            j += m;
        }
        let mult = 2.0 * PI / samples_in as f32;
        let envelope = (0..samples_in)
            .map(|i| 0.5 + 0.5 * (i as f32 * mult - PI / 2.0).sin())
            .collect();
        let mut equalize = Vec::with_capacity(nfreq / 2);
        let mut bias = 0.04f32;
        for i in 0..nfreq / 2 {
            let inv = (9.0 - bias) / (nfreq / 2) as f32;
            equalize.push((1.0 + bias + (i + 1) as f32 * inv).log10());
            bias /= 1.0025;
        }
        let mut cossin = Vec::new();
        let mut size = 2;
        while size <= nfreq {
            let theta = -2.0 * std::f64::consts::PI / size as f64;
            cossin.push((theta.cos() as f32, theta.sin() as f32));
            size <<= 1;
        }
        Fft {
            bitrev,
            envelope,
            equalize,
            cossin,
            re: vec![0.0; nfreq],
            im: vec![0.0; nfreq],
        }
    }

    fn run(&mut self, wave: &[f32], out: &mut [f32]) {
        let n = self.re.len();
        for i in 0..n {
            let idx = self.bitrev[i];
            self.re[i] = if idx < wave.len() {
                wave[idx] * self.envelope[idx]
            } else {
                0.0
            };
        }
        self.im.fill(0.0);
        let mut size = 2;
        let mut t = 0;
        while size <= n {
            let (wpr, wpi) = self.cossin[t];
            let (mut wr, mut wi) = (1.0f32, 0.0f32);
            let half = size >> 1;
            for m in 0..half {
                let mut i = m;
                while i < n {
                    let j = i + half;
                    let tr = wr * self.re[j] - wi * self.im[j];
                    let ti = wr * self.im[j] + wi * self.re[j];
                    self.re[j] = self.re[i] - tr;
                    self.im[j] = self.im[i] - ti;
                    self.re[i] += tr;
                    self.im[i] += ti;
                    i += size;
                }
                let tmp = wr;
                wr = wr * wpr - wi * wpi;
                wi = wi * wpr + tmp * wpi;
            }
            size <<= 1;
            t += 1;
        }
        for (i, o) in out.iter_mut().enumerate() {
            *o = (self.re[i] * self.re[i] + self.im[i] * self.im[i]).sqrt() * self.equalize[i];
        }
    }
}

pub struct Visualizer {
    fft: Fft,
    wave: Vec<f32>,
    spectrum: Vec<f32>,
    sample: [f32; 76],
    falloff_h: [f32; 76],
    peaks: [f32; 76],
    peak_speed: [f32; 76],
    last_y: i32,
}

impl Default for Visualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Visualizer {
    pub fn new() -> Visualizer {
        Visualizer {
            fft: Fft::new(),
            wave: vec![0.0; 1024],
            spectrum: vec![0.0; 512],
            sample: [0.0; 76],
            falloff_h: [0.0; 76],
            peaks: [0.0; 76],
            peak_speed: [0.0; 76],
            last_y: 0,
        }
    }

    pub fn heights(&self) -> Vec<i32> {
        self.falloff_h
            .iter()
            .step_by(4)
            .map(|h| h.round() as i32)
            .collect()
    }

    pub fn reset(&mut self) {
        self.falloff_h = [0.0; 76];
        self.peaks = [0.0; 76];
        self.peak_speed = [0.0; 76];
    }

    /// Feed the latest mono samples (most recent last). Call once per frame.
    pub fn analyze(&mut self, samples: &[f32], small: bool, opts: &VisOptions) {
        let n = samples.len();
        for i in 0..1024 {
            let s = if n >= 1024 {
                samples[n - 1024 + i]
            } else {
                0.0
            };
            // The original works on 8-bit samples scaled by 1/24.
            self.wave[i] = (s * 127.0).round().clamp(-128.0, 127.0) / 24.0;
        }
        self.fft.run(&self.wave, &mut self.spectrum);
        let target = if small { 40 } else { 75 };
        let max_h = if small { 5.0 } else { 15.0 };
        let scale = 0.91f32;
        let maxf = 512.0f32;
        let log_max = maxf.log10();
        for x in 0..target {
            let lin = x as f32 / (target - 1) as f32 * (maxf - 1.0);
            let logi = 10f32.powf(log_max * x as f32 / (target - 1) as f32);
            let idx = (1.0 - scale) * lin + scale * logi;
            let i1 = (idx.floor() as usize).min(511);
            let i2 = (idx.ceil() as usize).min(511);
            self.sample[x] = if i1 == i2 {
                self.spectrum[i1]
            } else {
                let f2 = idx - i1 as f32;
                (1.0 - f2) * self.spectrum[i1] + f2 * self.spectrum[i2]
            };
        }
        let width = if small { 37 } else { 75 };
        let falloff = [3.0, 6.0, 12.0, 16.0, 32.0][opts.falloff.min(4) as usize];
        let peak_falloff = [1.05, 1.1, 1.2, 1.4, 1.6][opts.peak_falloff.min(4) as usize];
        for x in 0..width {
            let data = if opts.wide_bands {
                let c = x & !3;
                (self.sample[c]
                    + self.sample[(c + 1).min(75)]
                    + self.sample[(c + 2).min(75)]
                    + self.sample[(c + 3).min(75)])
                    / 4.0
            } else {
                self.sample[x]
            };
            // The original stores the value in a 16-bit integer.
            let data = (data.trunc()).min(max_h);
            self.peaks[x] = self.peaks[x].min(max_h * 256.0);
            self.falloff_h[x] -= falloff / 16.0;
            if self.falloff_h[x] <= data {
                self.falloff_h[x] = data;
            }
            if self.peaks[x] <= (self.falloff_h[x] * 256.0).round() {
                self.peaks[x] = self.falloff_h[x] * 256.0;
                self.peak_speed[x] = 3.0;
            }
            self.peaks[x] = (self.peaks[x] - self.peak_speed[x].round()).trunc();
            self.peak_speed[x] *= peak_falloff;
            if self.peaks[x] <= 0.0 {
                self.peaks[x] = 0.0;
            }
        }
    }

    /// Draw the analyser into `dst` at (x, y). `colors` are the 24 viscolors.
    pub fn draw_analyzer(
        &self,
        dst: &mut Bitmap,
        x0: i32,
        y0: i32,
        small: bool,
        colors: &[u32; 24],
        opts: &VisOptions,
    ) {
        let h = if small { 5 } else { 16 };
        let push_down = if small { 0 } else { 2 };
        let width = if small { 37 } else { 75 };
        let small_cols = [colors[17], colors[14], colors[11], colors[8], colors[4]];
        let bar_color = |row: i32| -> u32 {
            if small {
                small_cols[(row.clamp(0, 4)) as usize]
            } else {
                colors[(row + 2 - push_down).clamp(0, 23) as usize]
            }
        };
        for x in 0..width {
            if opts.wide_bands && (x & 3) == 3 {
                continue;
            }
            let bar_h = (self.falloff_h[x].round() as i32 - push_down).max(0);
            let top = h - bar_h;
            for y in top.max(0)..h {
                let c = match opts.analyzer {
                    AnalyzerStyle::Normal => bar_color(y),
                    // Fire: colours start at the top of the bar.
                    AnalyzerStyle::Fire => bar_color(y - top),
                    // Line: the bar has one colour picked by its height.
                    AnalyzerStyle::Line => bar_color(top),
                };
                dst.set(x0 + x as i32, y0 + y, c);
            }
            if opts.peaks && !small {
                let mut peak = (self.peaks[x] / 256.0).trunc() as i32;
                if peak < 1 {
                    peak = -3;
                }
                let py = h - (peak + 1 - push_down);
                if (0..h).contains(&py) {
                    dst.set(x0 + x as i32, y0 + py, colors[23]);
                }
            }
        }
    }

    /// Draw the oscilloscope from the latest samples.
    pub fn draw_oscilloscope(
        &mut self,
        dst: &mut Bitmap,
        samples: &[f32],
        x0: i32,
        y0: i32,
        small: bool,
        colors: &[u32; 24],
        opts: &VisOptions,
    ) {
        let render_h = if small { 5 } else { 16 };
        let push_down = if small { 0 } else { 2 };
        let n = samples.len();
        let src = if n >= 576 {
            &samples[n - 576..]
        } else {
            samples
        };
        if src.is_empty() {
            return;
        }
        let slice = (src.len() / 75).max(1);
        let limit = if small { 38 } else { 75 };
        for x in 0..=75usize {
            if x >= limit {
                break;
            }
            let s = src[(x * slice).min(src.len() - 1)];
            let byte = ((s * 127.0).round() + 128.0).clamp(0.0, 255.0);
            let mut y = ((byte / 16.0) * 2.0).round() as i32 - 9;
            if small {
                y -= 5;
                y = (((y + 11) as f32 / 16.0) * 5.0).round() as i32 - 2;
            }
            y = y.clamp(0, render_h - 1);
            let v = y;
            if x == 0 {
                self.last_y = y;
            }
            let mut top = y;
            let mut bottom = self.last_y;
            self.last_y = y;
            match opts.osc {
                OscStyle::Solid => {
                    let mid = if small { 2 } else { 8 };
                    if y >= mid {
                        top = mid;
                        bottom = y;
                    } else {
                        top = y;
                        bottom = if small { 2 } else { 7 };
                    }
                    if x == 0 && small {
                        top = y;
                        bottom = y;
                    }
                }
                OscStyle::Dots => {
                    top = y;
                    bottom = y;
                }
                OscStyle::Lines => {
                    if bottom < top {
                        std::mem::swap(&mut top, &mut bottom);
                        if !small {
                            top += 1;
                        }
                    }
                }
            }
            let ci = if small {
                0
            } else {
                match v {
                    14.. => 4,
                    12..=13 => 3,
                    10..=11 => 2,
                    8..=9 => 1,
                    6..=7 => 0,
                    4..=5 => 1,
                    2..=3 => 2,
                    _ => 3,
                }
            };
            for yy in top..=bottom {
                let py = yy + push_down;
                if py < (if small { 5 } else { 16 }) {
                    dst.set(x0 + x as i32, y0 + py, colors[18 + ci]);
                }
            }
        }
    }
}

/// Background for the visualiser area: colour 0 with a dot grid of colour 1.
pub fn draw_background(
    dst: &mut Bitmap,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    colors: &[u32; 24],
    grid: bool,
) {
    dst.fill_rect(x0, y0, w, h, colors[0]);
    if grid {
        for y in (1..h).step_by(2) {
            for x in (0..w).step_by(2) {
                dst.set(x0 + x, y0 + y, colors[1]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzer_reacts_to_tone() {
        let mut v = Visualizer::new();
        let samples: Vec<f32> = (0..2048).map(|i| (i as f32 * 0.2).sin() * 0.8).collect();
        let opts = VisOptions::default();
        for _ in 0..3 {
            v.analyze(&samples, false, &opts);
        }
        assert!(v.falloff_h.iter().any(|&h| h > 5.0));
        let silent = vec![0.0f32; 2048];
        for _ in 0..200 {
            v.analyze(&silent, false, &opts);
        }
        assert!(v.falloff_h.iter().all(|&h| h <= 0.0));
    }

    #[test]
    fn modes_cycle() {
        assert_eq!(VisMode::Analyzer.next(), VisMode::Oscilloscope);
        assert_eq!(VisMode::Off.next(), VisMode::Analyzer);
    }
}
