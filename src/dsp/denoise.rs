//! Noise suppression with a wet/dry mix. Runs one of three methods (see
//! [`NoiseMethod`]); each is created on first use and dropped when another
//! method is picked, so the unused ones cost nothing.

use crate::config::NoiseMethod;

#[cfg(deepfilter)]
use super::deepfilter::DeepFilter;
use super::rnnoise::Rnnoise;
use super::spectral::Spectral;

/// Samples per block, 10 ms at 48 kHz. Every method works in these.
pub const FRAME: usize = 480;

#[cfg(deepfilter)]
const MAX_LATENCY: usize = DeepFilter::LATENCY;
#[cfg(not(deepfilter))]
const MAX_LATENCY: usize = Rnnoise::LATENCY;

enum Engine {
    None,
    Light(Box<Spectral>),
    Standard(Box<Rnnoise>),
    #[cfg(deepfilter)]
    HighQuality(Box<DeepFilter>),
}

pub struct Denoiser {
    engine: Engine,
    method: NoiseMethod,
    /// The last `MAX_LATENCY + FRAME` input samples, newest last, for the dry path.
    history: Vec<f32>,
    vad: f32,
}

impl Default for Denoiser {
    fn default() -> Self {
        Denoiser {
            engine: Engine::None,
            method: NoiseMethod::Standard,
            history: vec![0.0; MAX_LATENCY + FRAME],
            vad: 0.0,
        }
    }
}

impl Denoiser {
    /// Speech probability of the last frame (0..1).
    pub fn vad(&self) -> f32 {
        self.vad
    }

    /// Latency added while `method` is running, in samples.
    pub fn latency(method: NoiseMethod) -> usize {
        match method.effective() {
            NoiseMethod::Light => Spectral::LATENCY,
            NoiseMethod::Standard => Rnnoise::LATENCY,
            #[cfg(deepfilter)]
            NoiseMethod::HighQuality => DeepFilter::LATENCY,
            #[cfg(not(deepfilter))]
            NoiseMethod::HighQuality => unreachable!(),
        }
    }

    pub fn reset(&mut self) {
        self.engine = Engine::None;
        self.history.fill(0.0);
        self.vad = 0.0;
    }

    /// Processes exactly one `FRAME` at 48 kHz in place.
    /// `strength` 1.0 = fully denoised, 0.0 = dry (but still delayed by the
    /// method's latency, so moving the slider never shifts the timing).
    /// `level_gain` is the linear gain applied after noise suppression; see
    /// `DeepFilter::process` for why the level matters.
    pub fn process(
        &mut self,
        frame: &mut [f32],
        strength: f32,
        method: NoiseMethod,
        level_gain: f32,
    ) {
        debug_assert_eq!(frame.len(), FRAME);
        let method = method.effective();
        if method != self.method || matches!(self.engine, Engine::None) {
            self.method = method;
            self.engine = match method {
                NoiseMethod::Light => Engine::Light(Box::new(Spectral::new())),
                NoiseMethod::Standard => Engine::Standard(Box::new(Rnnoise::new())),
                #[cfg(deepfilter)]
                NoiseMethod::HighQuality => Engine::HighQuality(Box::new(DeepFilter::new())),
                #[cfg(not(deepfilter))]
                NoiseMethod::HighQuality => unreachable!(),
            };
        }

        self.history.copy_within(FRAME.., 0);
        let n = self.history.len();
        self.history[n - FRAME..].copy_from_slice(frame);
        let latency = Self::latency(method);
        let dry = &self.history[n - FRAME - latency..n - latency];

        let vad = match &mut self.engine {
            Engine::None => None,
            Engine::Light(e) => Some(e.process(frame)),
            Engine::Standard(e) => Some(e.process(frame)),
            #[cfg(deepfilter)]
            Engine::HighQuality(e) => e.process(frame, level_gain),
        };
        let Some(vad) = vad else {
            // Not ready yet: pass the dry signal through with the same timing.
            frame.copy_from_slice(dry);
            self.vad = 0.0;
            return;
        };
        self.vad = vad;
        let wet = strength.clamp(0.0, 1.0);
        for (s, d) in frame.iter_mut().zip(dry) {
            *s = *s * wet + d * (1.0 - wet);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Finds the lag at which `out` best matches `inp`.
    fn best_lag(inp: &[f32], out: &[f32], max_lag: usize) -> usize {
        (0..max_lag)
            .max_by(|&a, &b| {
                let c = |lag: usize| -> f64 {
                    (lag..out.len())
                        .map(|i| (out[i] * inp[i - lag]) as f64)
                        .sum()
                };
                c(a).partial_cmp(&c(b)).unwrap()
            })
            .unwrap()
    }

    fn run(input: &[f32], strength: f32, method: NoiseMethod) -> Vec<f32> {
        let mut d = Denoiser::default();
        let mut out = input.to_vec();
        for f in out.chunks_mut(FRAME) {
            d.process(f, strength, method, 1.0);
        }
        out
    }

    #[test]
    fn rnnoise_dry_path_is_aligned_with_its_output() {
        // A signal so quiet that RNNoise treats it as silence and only resynthesizes
        // it: the output is then a delayed copy, which reveals the latency.
        let mut seed = 12345u32;
        let input: Vec<f32> = (0..FRAME * 40)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((seed >> 8) as f32 / (1 << 24) as f32 - 0.5) * 2e-6
            })
            .collect();
        let latency = Denoiser::latency(NoiseMethod::Standard);
        for strength in [1.0, 0.0] {
            let out = run(&input, strength, NoiseMethod::Standard);
            assert_eq!(best_lag(&input, &out, FRAME * 2), latency);
        }
    }

    /// A voiced, vowel-like test signal.
    fn voice(len: usize) -> Vec<f32> {
        (0..len)
            .map(|i| {
                let t = i as f32 / 48_000.0;
                let f0 = 140.0 + 40.0 * (t * 3.0).sin();
                let s: f32 = (1..12)
                    .map(|k| (2.0 * std::f32::consts::PI * f0 * k as f32 * t).sin() / k as f32)
                    .sum();
                s * 0.05
            })
            .collect()
    }

    #[test]
    fn light_dry_path_is_aligned_with_its_output() {
        let input = voice(FRAME * 60);
        let latency = Denoiser::latency(NoiseMethod::Light);
        for strength in [1.0, 0.0] {
            let out = run(&input, strength, NoiseMethod::Light);
            assert_eq!(best_lag(&input, &out, FRAME * 2), latency);
        }
    }

    #[test]
    fn light_reduces_steady_noise() {
        let mut seed = 7u32;
        let noise: Vec<f32> = (0..48_000 * 3)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((seed >> 8) as f32 / (1 << 24) as f32 - 0.5) * 0.02
            })
            .collect();
        let out = run(&noise, 1.0, NoiseMethod::Light);
        let rms = |x: &[f32]| (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt();
        let tail = 48_000 * 2;
        let reduction = 20.0 * (rms(&out[tail..]) / rms(&noise[tail..])).log10();
        assert!(reduction < -10.0, "{reduction} dB");
    }

    /// Runs DeepFilterNet in real time on `input`; returns the output and how
    /// many frames the model actually processed.
    #[cfg(deepfilter)]
    fn run_deepfilter(input: &[f32], gain: f32) -> (Vec<f32>, usize) {
        let mut d = Denoiser::default();
        // Wait for the model so the comparison sees real output.
        let mut probe = vec![0.0; FRAME];
        let start = std::time::Instant::now();
        loop {
            d.process(&mut probe, 1.0, NoiseMethod::HighQuality, 1.0);
            if let Engine::HighQuality(e) = &d.engine {
                if e.ready() {
                    break;
                }
            }
            assert!(start.elapsed().as_secs() < 30, "model never loaded");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // Paced like a capture thread. The dry fallback would line up too, so
        // callers also check that the worker's results were actually used.
        let mut out = input.to_vec();
        let mut processed = 0;
        for f in out.chunks_mut(FRAME) {
            d.process(f, 1.0, NoiseMethod::HighQuality, gain);
            processed += (d.vad() > 0.0) as usize;
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        (out, processed)
    }

    #[cfg(deepfilter)]
    #[test]
    fn deepfilter_dry_path_is_aligned_with_its_output() {
        let input = voice(FRAME * 300);
        let (out, processed) = run_deepfilter(&input, 1.0);
        assert!(processed > 250, "{processed} of 300 frames processed");
        let latency = Denoiser::latency(NoiseMethod::HighQuality);
        assert_eq!(best_lag(&input, &out, FRAME * 6), latency);
    }

    /// A quiet microphone (here -50 dBFS, needing +32 dB of gain) used to be
    /// muted outright because the model judged its speech as silence.
    #[cfg(deepfilter)]
    #[test]
    fn deepfilter_keeps_quiet_voice_when_gain_follows() {
        let input: Vec<f32> = voice(FRAME * 300).iter().map(|s| s * 0.06).collect();
        let (out, _) = run_deepfilter(&input, 40.0);
        let energy = |x: &[f32]| x.iter().map(|v| (v * v) as f64).sum::<f64>();
        let tail = FRAME * 100;
        let db = 10.0 * (energy(&out[tail..]) / energy(&input[tail..])).log10();
        assert!(db > -6.0, "voice attenuated by {db:.1} dB");
    }
}
