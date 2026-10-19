//! Streaming mono windowed-sinc resampler for arbitrary rate pairs.
//! Only created when the device rate differs from the 48 kHz DSP rate, so the
//! common case costs nothing (v1.x always ran a resampler).

use std::f64::consts::PI;

const HALF_TAPS: usize = 16;
const TAPS: usize = HALF_TAPS * 2;
const PHASES: usize = 256;

pub struct Resampler {
    /// `PHASES + 1` rows of `TAPS` coefficients, row p = fractional offset p / PHASES.
    table: Vec<f32>,
    step: f64,
    pos: f64,
    buf: Vec<f32>,
}

impl Resampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        let ratio = to_rate as f64 / from_rate as f64;
        let cutoff = ratio.min(1.0) * 0.94;
        let mut table = vec![0f32; (PHASES + 1) * TAPS];
        for p in 0..=PHASES {
            let frac = p as f64 / PHASES as f64;
            let row = &mut table[p * TAPS..(p + 1) * TAPS];
            let mut sum = 0.0;
            for (k, c) in row.iter_mut().enumerate() {
                // Tap k sits at input offset k - (HALF_TAPS - 1) from the integer index.
                let x = k as f64 - (HALF_TAPS as f64 - 1.0) - frac;
                let sinc = if x.abs() < 1e-9 {
                    1.0
                } else {
                    (PI * cutoff * x).sin() / (PI * cutoff * x)
                };
                // Blackman window over the span of the kernel.
                let w = {
                    let t = (x / HALF_TAPS as f64 + 1.0) / 2.0;
                    if (0.0..=1.0).contains(&t) {
                        0.42 - 0.5 * (2.0 * PI * t).cos() + 0.08 * (4.0 * PI * t).cos()
                    } else {
                        0.0
                    }
                };
                let v = cutoff * sinc * w;
                *c = v as f32;
                sum += v;
            }
            // Unity DC gain for every phase.
            for c in row.iter_mut() {
                *c /= sum as f32;
            }
        }
        Resampler {
            table,
            step: from_rate as f64 / to_rate as f64,
            pos: (HALF_TAPS - 1) as f64,
            buf: vec![0.0; HALF_TAPS - 1],
        }
    }

    /// Feeds `input` and appends every output sample it allows to `out`.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        self.buf.extend_from_slice(input);
        loop {
            let i = self.pos as usize;
            if i + HALF_TAPS >= self.buf.len() {
                break;
            }
            let frac = (self.pos - i as f64) * PHASES as f64;
            let p = frac as usize;
            let t = (frac - p as f64) as f32;
            let r0 = &self.table[p * TAPS..(p + 1) * TAPS];
            let r1 = &self.table[(p + 1) * TAPS..(p + 2) * TAPS];
            let src = &self.buf[i + 1 - HALF_TAPS..i + 1 + HALF_TAPS];
            let mut acc = 0.0f32;
            for k in 0..TAPS {
                let c = r0[k] + (r1[k] - r0[k]) * t;
                acc += c * src[k];
            }
            out.push(acc);
            self.pos += self.step;
        }
        // Drop input that no future output can reach.
        let keep_from = (self.pos as usize + 1).saturating_sub(HALF_TAPS);
        if keep_from > 0 {
            self.buf.drain(..keep_from);
            self.pos -= keep_from as f64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(from: u32, to: u32, freq: f64) -> Vec<f32> {
        let mut r = Resampler::new(from, to);
        let input: Vec<f32> = (0..from as usize)
            .map(|n| (2.0 * PI * freq * n as f64 / from as f64).sin() as f32)
            .collect();
        let mut out = Vec::new();
        for chunk in input.chunks(441) {
            r.process(chunk, &mut out);
        }
        out
    }

    #[test]
    fn output_length_tracks_ratio() {
        for (from, to) in [
            (44_100, 48_000),
            (48_000, 44_100),
            (16_000, 48_000),
            (96_000, 48_000),
        ] {
            let out = run(from, to, 440.0);
            // Output lags by the kernel's look-ahead (HALF_TAPS input samples).
            let lag = HALF_TAPS as f64 * to as f64 / from as f64;
            assert!(
                (out.len() as f64 - to as f64).abs() <= lag + 2.0,
                "{from}->{to}: {}",
                out.len()
            );
        }
    }

    #[test]
    fn passband_tone_keeps_amplitude() {
        let out = run(44_100, 48_000, 1000.0);
        let peak = out[4000..].iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        assert!((peak - 1.0).abs() < 0.02, "{peak}");
    }

    #[test]
    fn downsampling_rejects_aliases() {
        // 30 kHz cannot exist at 48 kHz and must be filtered out.
        let out = run(96_000, 48_000, 30_000.0);
        let peak = out[4000..].iter().fold(0.0f32, |m, &v| m.max(v.abs()));
        assert!(peak < 0.01, "{peak}");
    }
}
