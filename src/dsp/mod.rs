//! The processing chain. Runs on the capture thread in 480-sample blocks at
//! 48 kHz mono: noise suppression → EQ → gain → noise gate → limiter.

pub mod biquad;
pub mod denoise;
pub mod dynamics;
pub mod resample;

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use realfft::{RealFftPlanner, RealToComplex};

use crate::config::{EqBand, MAX_EQ_POINTS};
use biquad::{Biquad, Coeffs};
use denoise::Denoiser;
use dynamics::{db_to_lin, Limiter, NoiseGate};

pub const SAMPLE_RATE: u32 = 48_000;
pub const BLOCK: usize = denoise::FRAME;
pub const FFT_SIZE: usize = 2048;

/// What the UI wants the chain to do. Swapped atomically as a whole.
#[derive(Clone, Debug, PartialEq)]
pub struct DspParams {
    pub boost: bool,
    pub gain_db: f32,
    pub bands: Vec<EqBand>,
    pub noise_suppression: bool,
    pub suppression_strength: f32,
    pub gate_db: f32,
    pub limiter_ceiling_db: f32,
}

impl Default for DspParams {
    fn default() -> Self {
        DspParams {
            boost: false,
            gain_db: 0.0,
            bands: Vec::new(),
            noise_suppression: false,
            suppression_strength: 1.0,
            gate_db: dynamics::GATE_OFF_DB,
            limiter_ceiling_db: -1.0,
        }
    }
}

/// An f32 in an atomic. Meters store "max since the UI last looked".
#[derive(Default)]
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn store(&self, v: f32) {
        self.0.store(v.to_bits(), Ordering::Relaxed)
    }
    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
    /// For non-negative values the bit patterns sort like the floats.
    pub fn fetch_max(&self, v: f32) {
        self.0.fetch_max(v.max(0.0).to_bits(), Ordering::Relaxed);
    }
    pub fn take(&self) -> f32 {
        f32::from_bits(self.0.swap(0, Ordering::Relaxed))
    }
    pub fn fetch_min(&self, v: f32) {
        self.0.fetch_min(v.max(0.0).to_bits(), Ordering::Relaxed);
    }
    pub fn take_min(&self) -> f32 {
        f32::from_bits(self.0.swap(1f32.to_bits(), Ordering::Relaxed))
    }
}

#[derive(Default)]
pub struct Meters {
    /// Raw microphone peak (linear).
    pub input_peak: AtomicF32,
    /// Final output peak (linear).
    pub output_peak: AtomicF32,
    /// Noise gate detector level (dBFS, post-gain).
    pub gate_level_db: AtomicF32,
    pub gate_open: AtomicBool,
    /// Smallest limiter gain since last read (1.0 = idle).
    pub limiter_gain: AtomicF32,
    /// Speech probability from RNNoise.
    pub vad: AtomicF32,
}

/// Shared between the audio callbacks and the UI.
pub struct DspShared {
    params: ArcSwap<DspParams>,
    version: AtomicU64,
    pub meters: Meters,
    /// Output spectrum in dBFS, `FFT_SIZE / 2` bins. Only filled while the UI is visible.
    pub spectrum: Mutex<Vec<f32>>,
    pub spectrum_version: AtomicU64,
    pub ui_visible: AtomicBool,
}

impl Default for DspShared {
    fn default() -> Self {
        let m = Meters::default();
        m.limiter_gain.store(1.0);
        DspShared {
            params: ArcSwap::from_pointee(DspParams::default()),
            version: AtomicU64::new(1),
            meters: m,
            spectrum: Mutex::new(vec![-120.0; FFT_SIZE / 2]),
            spectrum_version: AtomicU64::new(0),
            ui_visible: AtomicBool::new(true),
        }
    }
}

impl DspShared {
    pub fn set_params(&self, p: DspParams) {
        if **self.params.load() != p {
            self.params.store(Arc::new(p));
            self.version.fetch_add(1, Ordering::Release);
        }
    }

    pub fn params(&self) -> Arc<DspParams> {
        self.params.load_full()
    }
}

struct SpectrumTap {
    ring: Vec<f32>,
    pos: usize,
    blocks: u32,
    fft: Arc<dyn RealToComplex<f32>>,
    window: Vec<f32>,
    input: Vec<f32>,
    output: Vec<realfft::num_complex::Complex<f32>>,
    scratch: Vec<realfft::num_complex::Complex<f32>>,
    mags: Vec<f32>,
}

impl SpectrumTap {
    fn new() -> Self {
        let fft = RealFftPlanner::<f32>::new().plan_fft_forward(FFT_SIZE);
        let window: Vec<f32> = (0..FFT_SIZE)
            .map(|n| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * n as f32 / FFT_SIZE as f32).cos())
            .collect();
        SpectrumTap {
            ring: vec![0.0; FFT_SIZE],
            pos: 0,
            blocks: 0,
            input: fft.make_input_vec(),
            output: fft.make_output_vec(),
            scratch: fft.make_scratch_vec(),
            mags: vec![-120.0; FFT_SIZE / 2],
            fft,
            window,
        }
    }

    fn push(&mut self, block: &[f32], shared: &DspShared) {
        for &s in block {
            self.ring[self.pos] = s;
            self.pos = (self.pos + 1) % FFT_SIZE;
        }
        self.blocks += 1;
        // ~33 updates per second.
        if self.blocks % 3 != 0 {
            return;
        }
        for i in 0..FFT_SIZE {
            self.input[i] = self.ring[(self.pos + i) % FFT_SIZE] * self.window[i];
        }
        if self
            .fft
            .process_with_scratch(&mut self.input, &mut self.output, &mut self.scratch)
            .is_err()
        {
            return;
        }
        // Hann window coherent gain 0.5 → full-scale sine reads 0 dBFS.
        let norm = 2.0 / (FFT_SIZE as f32 * 0.5);
        for (m, c) in self.mags.iter_mut().zip(&self.output) {
            *m = 20.0 * (c.norm() * norm).max(1e-6).log10();
        }
        if let Ok(mut s) = shared.spectrum.try_lock() {
            s.copy_from_slice(&self.mags);
            shared.spectrum_version.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub struct Processor {
    shared: Arc<DspShared>,
    seen_version: u64,
    params: Arc<DspParams>,
    filters: [Biquad; MAX_EQ_POINTS],
    filter_count: usize,
    gain: f32,
    denoiser: Denoiser,
    gate: NoiseGate,
    limiter: Limiter,
    spectrum: SpectrumTap,
}

impl Processor {
    pub fn new(shared: Arc<DspShared>) -> Self {
        let sr = SAMPLE_RATE as f32;
        let params = shared.params();
        let mut p = Processor {
            shared,
            seen_version: 0,
            params,
            filters: [Biquad::default(); MAX_EQ_POINTS],
            filter_count: 0,
            gain: 1.0,
            denoiser: Denoiser::default(),
            gate: NoiseGate::new(sr),
            limiter: Limiter::new(sr),
            spectrum: SpectrumTap::new(),
        };
        p.refresh_params();
        p.gain = p.target_gain();
        p
    }

    fn target_gain(&self) -> f32 {
        if self.params.boost {
            db_to_lin(self.params.gain_db)
        } else {
            1.0
        }
    }

    fn refresh_params(&mut self) {
        let v = self.shared.version.load(Ordering::Acquire);
        if v == self.seen_version {
            return;
        }
        self.seen_version = v;
        self.params = self.shared.params();
        let sr = SAMPLE_RATE as f64;
        let mut n = 0;
        if self.params.boost {
            for band in self.params.bands.iter().take(MAX_EQ_POINTS) {
                if let Some(c) = Coeffs::for_band(band, sr) {
                    self.filters[n].set(c);
                    n += 1;
                }
            }
        }
        // Filters that come into use start from silence instead of stale state.
        for f in &mut self.filters[self.filter_count.min(n)..n] {
            f.reset();
        }
        self.filter_count = n;
        if !self.params.noise_suppression {
            self.denoiser.reset();
        }
    }

    /// Processes one block of `BLOCK` samples in place.
    pub fn process(&mut self, block: &mut [f32]) {
        debug_assert_eq!(block.len(), BLOCK);
        self.refresh_params();
        let m = &self.shared.meters;
        m.input_peak.fetch_max(peak(block));

        if self.params.noise_suppression {
            self.denoiser
                .process(block, self.params.suppression_strength);
            m.vad.store(self.denoiser.vad());
        }

        if self.filter_count > 0 {
            let filters = &mut self.filters[..self.filter_count];
            for s in block.iter_mut() {
                let mut x = *s as f64;
                for f in filters.iter_mut() {
                    x = f.process(x);
                }
                *s = x as f32;
            }
        }

        // Linear ramp to the new gain over the block: no zipper noise while dragging.
        let target = self.target_gain();
        let step = (target - self.gain) / BLOCK as f32;
        for s in block.iter_mut() {
            self.gain += step;
            *s *= self.gain;
        }
        self.gain = target;

        let open = self.gate.process(block, self.params.gate_db);
        m.gate_open.store(open, Ordering::Relaxed);
        m.gate_level_db.store(self.gate.level_db());

        let lg = self.limiter.process(block, self.params.limiter_ceiling_db);
        m.limiter_gain.fetch_min(lg);
        m.output_peak.fetch_max(peak(block));

        if self.shared.ui_visible.load(Ordering::Relaxed) {
            self.spectrum.push(block, &self.shared);
        }
    }
}

fn peak(buf: &[f32]) -> f32 {
    buf.iter().fold(0.0f32, |m, &v| m.max(v.abs()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(amp: f32) -> Vec<f32> {
        (0..BLOCK).map(|n| (n as f32 * 0.13).sin() * amp).collect()
    }

    #[test]
    fn boost_applies_gain_and_limiter_caps_it() {
        let shared = Arc::new(DspShared::default());
        shared.set_params(DspParams {
            boost: true,
            gain_db: 40.0,
            ..Default::default()
        });
        let mut p = Processor::new(shared.clone());
        let mut out = 0.0f32;
        for _ in 0..20 {
            let mut b = tone(0.1);
            p.process(&mut b);
            out = out.max(peak(&b));
        }
        assert!(out <= db_to_lin(-1.0) + 1e-6);
        assert!(out > 0.85);
    }

    /// `cargo test --release -- --ignored --nocapture bench_chain`
    #[test]
    #[ignore]
    fn bench_chain() {
        let secs = 60;
        let input: Vec<f32> = (0..SAMPLE_RATE as usize * secs)
            .map(|n| {
                (n as f32 * 0.031).sin() * 0.2 + ((n * 7919 % 1000) as f32 / 1000.0 - 0.5) * 0.02
            })
            .collect();
        for ns in [false, true] {
            let shared = Arc::new(DspShared::default());
            shared.ui_visible.store(false, Ordering::Relaxed);
            shared.set_params(DspParams {
                boost: true,
                gain_db: 32.0,
                noise_suppression: ns,
                gate_db: -50.0,
                bands: vec![
                    EqBand::peaking(3000.0, 4.0, 1.0),
                    EqBand::peaking(200.0, -3.0, 1.0),
                ],
                ..Default::default()
            });
            let mut p = Processor::new(shared);
            let mut buf = input.clone();
            let t = std::time::Instant::now();
            for b in buf.chunks_mut(BLOCK) {
                p.process(b);
            }
            let cpu = t.elapsed().as_secs_f64() / secs as f64 * 100.0;
            println!("noise suppression {ns}: {cpu:.2}% of one core");
        }
    }

    #[test]
    fn boost_off_is_unity() {
        let shared = Arc::new(DspShared::default());
        shared.set_params(DspParams {
            boost: false,
            gain_db: 30.0,
            ..Default::default()
        });
        let mut p = Processor::new(shared);
        let mut last = Vec::new();
        for _ in 0..4 {
            last = tone(0.2);
            p.process(&mut last);
        }
        let pk = peak(&last);
        assert!((pk - 0.2).abs() < 0.01, "{pk}");
    }

    #[test]
    fn param_changes_are_picked_up() {
        let shared = Arc::new(DspShared::default());
        let mut p = Processor::new(shared.clone());
        shared.set_params(DspParams {
            boost: true,
            gain_db: -20.0,
            ..Default::default()
        });
        let mut b = tone(0.5);
        for _ in 0..3 {
            b = tone(0.5);
            p.process(&mut b);
        }
        assert!(peak(&b) < 0.06);
    }
}
