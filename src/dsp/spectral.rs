//! Lightweight spectral noise reduction: a Wiener filter on a 50 %-overlap
//! STFT, with the noise floor tracked per bin by its running minimum.
//!
//! It has no model, so it costs a fraction of RNNoise. It learns steady noise
//! (fans, hum, hiss) within a second or two but cannot tell keystrokes or
//! voices in the background from speech.

use std::sync::Arc;

use realfft::num_complex::Complex;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

use super::denoise::FRAME;

const FFT: usize = FRAME * 2;
const BINS: usize = FFT / 2 + 1;
/// Power smoothing used for the noise tracker.
const SMOOTH: f32 = 0.7;
/// How fast the noise floor may rise while the signal stays above it (~3 dB/s).
const RISE: f32 = 1.0069;
/// The running minimum sits below the mean noise power; scale it back up.
const MIN_BIAS: f32 = 2.0;
/// Decision-directed a-priori SNR weight.
const DD: f32 = 0.98;
/// Never attenuate a bin by more than this (-20 dB) to limit musical noise.
const GAIN_FLOOR: f32 = 0.1;
/// Frames used to seed the noise estimate.
const SEED_FRAMES: u32 = 10;

pub struct Spectral {
    fft: Arc<dyn RealToComplex<f32>>,
    ifft: Arc<dyn ComplexToReal<f32>>,
    window: Vec<f32>,
    /// The previous frame's input followed by the current one.
    history: Vec<f32>,
    time: Vec<f32>,
    spec: Vec<Complex<f32>>,
    scratch_fwd: Vec<Complex<f32>>,
    scratch_inv: Vec<Complex<f32>>,
    /// Second half of the last synthesis frame, added to the next one.
    tail: Vec<f32>,
    smoothed: Vec<f32>,
    noise: Vec<f32>,
    clean_prev: Vec<f32>,
    gains: Vec<f32>,
    frames: u32,
}

impl Spectral {
    /// Output lags the input by one hop (the overlap-add).
    pub const LATENCY: usize = FRAME;

    pub fn new() -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT);
        let ifft = planner.plan_fft_inverse(FFT);
        // sqrt of a periodic Hann on both sides: the product sums to 1 at 50 % overlap.
        let window = (0..FFT)
            .map(|n| {
                (0.5 - 0.5 * (2.0 * std::f32::consts::PI * n as f32 / FFT as f32).cos()).sqrt()
            })
            .collect();
        Spectral {
            time: fft.make_input_vec(),
            spec: fft.make_output_vec(),
            scratch_fwd: fft.make_scratch_vec(),
            scratch_inv: ifft.make_scratch_vec(),
            fft,
            ifft,
            window,
            history: vec![0.0; FFT],
            tail: vec![0.0; FRAME],
            smoothed: vec![0.0; BINS],
            noise: vec![0.0; BINS],
            clean_prev: vec![0.0; BINS],
            gains: vec![1.0; BINS],
            frames: 0,
        }
    }

    /// Replaces `frame` with the denoised signal; returns a rough speech probability.
    pub fn process(&mut self, frame: &mut [f32]) -> f32 {
        self.history.copy_within(FRAME.., 0);
        self.history[FRAME..].copy_from_slice(frame);
        for ((t, h), w) in self.time.iter_mut().zip(&self.history).zip(&self.window) {
            *t = h * w;
        }
        if self
            .fft
            .process_with_scratch(&mut self.time, &mut self.spec, &mut self.scratch_fwd)
            .is_err()
        {
            return 0.0;
        }

        self.frames = self.frames.saturating_add(1);
        let seeding = self.frames <= SEED_FRAMES;
        let (mut sig, mut noise) = (0.0f32, 0.0f32);
        for k in 0..BINS {
            let p = self.spec[k].norm_sqr();
            let s = &mut self.smoothed[k];
            *s = if self.frames == 1 {
                p
            } else {
                SMOOTH * *s + (1.0 - SMOOTH) * p
            };
            let n = &mut self.noise[k];
            if seeding {
                // Average of the first frames, so the filter starts sensibly.
                *n += (*s - *n) / self.frames as f32;
            } else if *s < *n {
                *n = *s;
            } else {
                *n = *n * RISE + 1e-12;
            }
            let n = (*n * MIN_BIAS).max(1e-12);
            let gamma = p / n;
            let xi = DD * self.clean_prev[k] / n + (1.0 - DD) * (gamma - 1.0).max(0.0);
            self.gains[k] = xi / (1.0 + xi);
            // 300 Hz – 3.4 kHz decides the speech estimate.
            if (6..=68).contains(&k) {
                sig += p;
                noise += n;
            }
        }
        // A little smoothing across bins tames isolated "musical" tones.
        let mut prev = self.gains[0];
        for k in 0..BINS {
            let next = self.gains[(k + 1).min(BINS - 1)];
            let cur = self.gains[k];
            let g = (0.25 * prev + 0.5 * cur + 0.25 * next).max(GAIN_FLOOR);
            prev = cur;
            self.spec[k] *= g;
            self.clean_prev[k] = self.spec[k].norm_sqr();
        }
        // DC and Nyquist must be real for the inverse transform.
        self.spec[0].im = 0.0;
        self.spec[BINS - 1].im = 0.0;
        if self
            .ifft
            .process_with_scratch(&mut self.spec, &mut self.time, &mut self.scratch_inv)
            .is_err()
        {
            return 0.0;
        }

        let norm = 1.0 / FFT as f32;
        #[allow(clippy::needless_range_loop)]
        for i in 0..FRAME {
            frame[i] = self.tail[i] + self.time[i] * self.window[i] * norm;
            self.tail[i] = self.time[FRAME + i] * self.window[FRAME + i] * norm;
        }
        let snr_db = 10.0 * (sig / noise.max(1e-12)).max(1e-6).log10();
        ((snr_db - 3.0) / 12.0).clamp(0.0, 1.0)
    }
}
