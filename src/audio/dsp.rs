//! Signal processing: the 10-band equalizer and a windowed-sinc resampler.

use std::f64::consts::PI;

pub const EQ_FREQS: [f64; 10] = [60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0, 14000.0, 16000.0];
/// Range of the equalizer sliders in dB (the scale printed on the window).
pub const EQ_RANGE_DB: f64 = 12.0;

/// Equalizer settings as the user sees them. Slider values are 0..=63 with
/// 0 at the top (+12 dB) and 63 at the bottom (-12 dB), like the classic EQF format.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqSettings {
    pub enabled: bool,
    pub preamp: u8,
    pub bands: [u8; 10],
}

impl Default for EqSettings {
    fn default() -> Self {
        EqSettings { enabled: false, preamp: 31, bands: [31; 10] }
    }
}

pub fn slider_to_db(v: u8) -> f64 {
    // 0 -> +12, 31 -> 0 (the default position), 63 -> -12
    let v = v.min(63) as f64;
    if v <= 31.0 { (31.0 - v) / 31.0 * EQ_RANGE_DB } else { (31.0 - v) / 32.0 * EQ_RANGE_DB }
}

#[derive(Clone, Copy, Default)]
struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    // state per channel
    z: [[f64; 2]; 2],
}

impl Biquad {
    fn peaking(rate: f64, freq: f64, q: f64, gain_db: f64) -> Biquad {
        let a = 10f64.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / rate;
        let alpha = w0.sin() / (2.0 * q);
        let cos = w0.cos();
        let a0 = 1.0 + alpha / a;
        Biquad {
            b0: (1.0 + alpha * a) / a0,
            b1: (-2.0 * cos) / a0,
            b2: (1.0 - alpha * a) / a0,
            a1: (-2.0 * cos) / a0,
            a2: (1.0 - alpha / a) / a0,
            z: [[0.0; 2]; 2],
        }
    }

    #[inline]
    fn run(&mut self, ch: usize, x: f64) -> f64 {
        // Transposed direct form II.
        let z = &mut self.z[ch];
        let y = self.b0 * x + z[0];
        z[0] = self.b1 * x - self.a1 * y + z[1];
        z[1] = self.b2 * x - self.a2 * y;
        y
    }
}

pub struct Equalizer {
    rate: f64,
    settings: EqSettings,
    filters: Vec<Biquad>,
    preamp_gain: f64,
}

impl Equalizer {
    pub fn new(rate: u32, settings: EqSettings) -> Equalizer {
        let mut e = Equalizer { rate: rate as f64, settings, filters: Vec::new(), preamp_gain: 1.0 };
        e.rebuild();
        e
    }

    pub fn set(&mut self, settings: EqSettings) {
        if settings != self.settings {
            let old = std::mem::take(&mut self.filters);
            self.settings = settings;
            self.rebuild();
            // Keep filter state across parameter changes to avoid clicks.
            for (n, o) in self.filters.iter_mut().zip(old.iter()) {
                n.z = o.z;
            }
        }
    }

    fn rebuild(&mut self) {
        self.filters.clear();
        self.preamp_gain = 10f64.powf(slider_to_db(self.settings.preamp) / 20.0);
        for (i, &f) in EQ_FREQS.iter().enumerate() {
            if f >= self.rate * 0.45 {
                continue;
            }
            let db = slider_to_db(self.settings.bands[i]);
            self.filters.push(Biquad::peaking(self.rate, f, 1.4, db));
        }
    }

    /// Process interleaved stereo frames in place.
    pub fn process(&mut self, frames: &mut [[f32; 2]]) {
        if !self.settings.enabled {
            return;
        }
        for fr in frames.iter_mut() {
            for ch in 0..2 {
                let mut x = fr[ch] as f64 * self.preamp_gain;
                for f in self.filters.iter_mut() {
                    x = f.run(ch, x);
                }
                fr[ch] = x as f32;
            }
        }
    }
}

/// Band-limited resampler using a Blackman-windowed sinc kernel with a
/// precomputed polyphase table and linear interpolation between phases.
pub struct Resampler {
    step: f64,
    half_taps: usize,
    phases: usize,
    table: Vec<f32>,
    buf: Vec<[f32; 2]>,
    pos: f64,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Resampler {
        let step = in_rate as f64 / out_rate as f64;
        let half_taps = 16usize;
        let phases = 512usize;
        let cutoff = if step > 1.0 { 0.97 / step } else { 0.97 };
        let taps = half_taps * 2;
        let mut table = vec![0f32; (phases + 1) * taps];
        for p in 0..=phases {
            let frac = p as f64 / phases as f64;
            let mut row = vec![0f64; taps];
            let mut sum = 0.0;
            for (k, v) in row.iter_mut().enumerate() {
                // Tap k corresponds to input offset (k - half_taps + 1) from floor(pos).
                let x = (k as f64 - half_taps as f64 + 1.0) - frac;
                let sinc = if x.abs() < 1e-9 { 1.0 } else { (PI * cutoff * x).sin() / (PI * cutoff * x) };
                let n = (x + half_taps as f64) / (2.0 * half_taps as f64);
                let w = if (0.0..=1.0).contains(&n) {
                    0.42 - 0.5 * (2.0 * PI * n).cos() + 0.08 * (4.0 * PI * n).cos()
                } else {
                    0.0
                };
                *v = sinc * w;
                sum += *v;
            }
            for (k, v) in row.iter().enumerate() {
                table[p * taps + k] = (v / sum) as f32;
            }
        }
        Resampler {
            step,
            half_taps,
            phases,
            table,
            buf: vec![[0.0; 2]; half_taps - 1],
            pos: (half_taps - 1) as f64,
        }
    }

    pub fn is_identity(&self) -> bool {
        (self.step - 1.0).abs() < 1e-12
    }

    pub fn process(&mut self, input: &[[f32; 2]], out: &mut Vec<[f32; 2]>) {
        if self.is_identity() {
            out.extend_from_slice(input);
            return;
        }
        self.buf.extend_from_slice(input);
        let taps = self.half_taps * 2;
        loop {
            let base = self.pos.floor() as usize;
            if base + self.half_taps >= self.buf.len() {
                break;
            }
            let frac = self.pos - base as f64;
            let pf = frac * self.phases as f64;
            let p0 = pf.floor() as usize;
            let t = (pf - p0 as f64) as f32;
            let r0 = &self.table[p0 * taps..(p0 + 1) * taps];
            let r1 = &self.table[(p0 + 1) * taps..(p0 + 2) * taps];
            let start = base + 1 - self.half_taps;
            let mut acc = [0f32; 2];
            for k in 0..taps {
                let c = r0[k] + (r1[k] - r0[k]) * t;
                let s = self.buf[start + k];
                acc[0] += s[0] * c;
                acc[1] += s[1] * c;
            }
            out.push(acc);
            self.pos += self.step;
        }
        // Drop consumed input, keeping enough history for the kernel.
        let keep_from = (self.pos.floor() as usize).saturating_sub(self.half_taps - 1);
        if keep_from > 0 {
            self.buf.drain(..keep_from);
            self.pos -= keep_from as f64;
        }
    }

    pub fn reset(&mut self) {
        self.buf.clear();
        self.buf.resize(self.half_taps - 1, [0.0; 2]);
        self.pos = (self.half_taps - 1) as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(rate: u32, freq: f64, n: usize) -> Vec<[f32; 2]> {
        (0..n)
            .map(|i| {
                let v = (2.0 * PI * freq * i as f64 / rate as f64).sin() as f32 * 0.5;
                [v, v]
            })
            .collect()
    }

    fn rms(v: &[[f32; 2]]) -> f64 {
        (v.iter().map(|f| (f[0] as f64).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    }

    #[test]
    fn slider_scale() {
        assert!((slider_to_db(0) - 12.0).abs() < 1e-9);
        assert!((slider_to_db(63) + 12.0).abs() < 1e-9);
        assert_eq!(slider_to_db(31), 0.0);
    }

    #[test]
    fn resampler_keeps_length_and_level() {
        let input = sine(44100, 1000.0, 44100);
        let mut r = Resampler::new(44100, 48000);
        let mut out = Vec::new();
        for chunk in input.chunks(1000) {
            r.process(chunk, &mut out);
        }
        let expected = 48000.0;
        assert!((out.len() as f64 - expected).abs() < 40.0, "{}", out.len());
        let level = rms(&out[2000..40000]);
        assert!((level - 0.5 / 2f64.sqrt()).abs() < 0.01, "{level}");
    }

    #[test]
    fn resampler_attenuates_above_nyquist_when_downsampling() {
        let input = sine(48000, 20000.0, 48000);
        let mut r = Resampler::new(48000, 22050);
        let mut out = Vec::new();
        r.process(&input, &mut out);
        assert!(rms(&out[1000..]) < 0.02);
    }

    #[test]
    fn eq_boosts_band() {
        let mut s = EqSettings { enabled: true, ..Default::default() };
        s.bands[4] = 0; // +12 dB at 1 kHz
        let mut eq = Equalizer::new(44100, s);
        let mut x = sine(44100, 1000.0, 44100);
        eq.process(&mut x);
        let gain = rms(&x[10000..]) / (0.5 / 2f64.sqrt());
        assert!(gain > 3.0 && gain < 4.5, "{gain}");
        // Disabled EQ is a no-op.
        let mut eq = Equalizer::new(44100, EqSettings { enabled: false, ..s });
        let mut y = sine(44100, 1000.0, 1000);
        let before = y.clone();
        eq.process(&mut y);
        assert_eq!(before, y);
    }
}
