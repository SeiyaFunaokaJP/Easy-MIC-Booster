//! RNNoise (via the pure-Rust `nnnoiseless` port).
//!
//! v1.x shipped the xiph rnnoise 0.2 C library with its large model, built
//! without SIMD; that was most of its CPU use. nnnoiseless runs the original
//! lighter model.

use nnnoiseless::DenoiseState;

use super::denoise::FRAME;

const SCALE: f32 = 32768.0;

pub struct Rnnoise {
    state: Box<DenoiseState<'static>>,
    input: [f32; FRAME],
    output: [f32; FRAME],
    warm: bool,
}

impl Rnnoise {
    /// Output lags the input by one frame.
    pub const LATENCY: usize = FRAME;

    pub fn new() -> Self {
        Rnnoise {
            state: DenoiseState::new(),
            input: [0.0; FRAME],
            output: [0.0; FRAME],
            warm: false,
        }
    }

    /// Replaces `frame` with the denoised signal; returns the speech probability.
    pub fn process(&mut self, frame: &mut [f32]) -> f32 {
        for (d, s) in self.input.iter_mut().zip(frame.iter()) {
            *d = *s * SCALE;
        }
        let vad = self.state.process_frame(&mut self.output, &self.input);
        // The very first frame is the model's start-up transient.
        let first = !std::mem::replace(&mut self.warm, true);
        for (d, s) in frame.iter_mut().zip(&self.output) {
            *d = if first { 0.0 } else { *s / SCALE };
        }
        vad
    }
}
