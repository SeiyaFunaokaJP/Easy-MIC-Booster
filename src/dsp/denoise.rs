//! RNNoise (via the pure-Rust `nnnoiseless` port) with a wet/dry mix.
//!
//! v1.x shipped the xiph rnnoise 0.2 C library with its large model, built
//! without SIMD; that was most of its CPU use. nnnoiseless runs the original
//! lighter model and costs nothing at all while disabled because the state is
//! only created on first use.

use nnnoiseless::DenoiseState;

pub const FRAME: usize = DenoiseState::FRAME_SIZE;
const SCALE: f32 = 32768.0;

pub struct Denoiser {
    state: Option<Box<DenoiseState<'static>>>,
    input: [f32; FRAME],
    output: [f32; FRAME],
    /// Dry signal delayed by one frame to line up with RNNoise's output.
    dry: [f32; FRAME],
    vad: f32,
    warm: bool,
}

impl Default for Denoiser {
    fn default() -> Self {
        Denoiser {
            state: None,
            input: [0.0; FRAME],
            output: [0.0; FRAME],
            dry: [0.0; FRAME],
            vad: 0.0,
            warm: false,
        }
    }
}

impl Denoiser {
    /// Speech probability of the last frame (0..1).
    pub fn vad(&self) -> f32 {
        self.vad
    }

    /// Latency added while enabled, in samples.
    #[cfg(test)]
    pub const LATENCY: usize = FRAME;

    pub fn reset(&mut self) {
        self.state = None;
        self.warm = false;
        self.dry = [0.0; FRAME];
    }

    /// Processes exactly one 480-sample frame at 48 kHz in place.
    /// `strength` 1.0 = fully denoised, 0.0 = dry (but still one frame late,
    /// so toggling strength never shifts the timing).
    pub fn process(&mut self, frame: &mut [f32], strength: f32) {
        debug_assert_eq!(frame.len(), FRAME);
        let state = self.state.get_or_insert_with(DenoiseState::new);
        for (d, s) in self.input.iter_mut().zip(frame.iter()) {
            *d = *s * SCALE;
        }
        self.vad = state.process_frame(&mut self.output, &self.input);
        let first = !self.warm;
        self.warm = true;

        let wet = strength.clamp(0.0, 1.0);
        let dry = 1.0 - wet;
        #[allow(clippy::needless_range_loop)]
        for i in 0..FRAME {
            let delayed = self.dry[i];
            self.dry[i] = frame[i];
            frame[i] = if first {
                0.0
            } else {
                self.output[i] / SCALE * wet + delayed * dry
            };
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

    #[test]
    fn dry_path_is_aligned_with_rnnoise_output() {
        // A signal so quiet that RNNoise treats it as silence and only resynthesizes
        // it: the output is then a delayed copy, which reveals the latency.
        let mut seed = 12345u32;
        let input: Vec<f32> = (0..FRAME * 40)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((seed >> 8) as f32 / (1 << 24) as f32 - 0.5) * 2e-6
            })
            .collect();

        let run = |strength: f32| -> Vec<f32> {
            let mut d = Denoiser::default();
            let mut out = input.clone();
            for f in out.chunks_mut(FRAME) {
                d.process(f, strength);
            }
            out
        };
        assert_eq!(best_lag(&input, &run(1.0), FRAME * 2), Denoiser::LATENCY);
        assert_eq!(best_lag(&input, &run(0.0), FRAME * 2), Denoiser::LATENCY);
    }
}
