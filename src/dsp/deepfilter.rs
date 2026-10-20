//! DeepFilterNet3 (via `libDF` on the pure-Rust `tract` runtime).
//!
//! Full-band and clearly better than RNNoise on non-steady noise, at roughly
//! five times its CPU. The model is not `Send`, so it lives on its own thread:
//! the capture thread hands it each frame and picks up the previous frame's
//! result, which adds one frame of latency but keeps the inference (and its
//! allocations) off the audio callback. Loading takes ~150 ms; until then the
//! signal passes through unprocessed.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::Thread;
use std::time::Duration;

use df::tract::{DfParams, DfTract, RuntimeParams};
use ndarray::Array2;

use super::denoise::FRAME;

const LOADING: u8 = 0;
const READY: u8 = 1;
const FAILED: u8 = 2;
/// Output frames carry the speech probability after the samples.
const OUT_FRAME: usize = FRAME + 1;
/// Upper bound for the pre-gain given to the model (+50 dB, the normal gain limit).
const MAX_LEVEL_GAIN: f32 = 316.0;

struct Shared {
    state: AtomicU8,
    stop: AtomicBool,
}

pub struct DeepFilter {
    shared: Arc<Shared>,
    worker: Option<Thread>,
    to_worker: rtrb::Producer<f32>,
    from_worker: rtrb::Consumer<f32>,
    /// Frames handed to the worker / results taken back (used or skipped).
    sent: u64,
    received: u64,
}

impl DeepFilter {
    /// STFT (one hop), the model's two-frame lookahead and the hand-off frame.
    pub const LATENCY: usize = FRAME * 4;

    pub fn new() -> Self {
        let shared = Arc::new(Shared {
            state: AtomicU8::new(LOADING),
            stop: AtomicBool::new(false),
        });
        let (to_worker, input) = rtrb::RingBuffer::new(FRAME * 8);
        let (output, from_worker) = rtrb::RingBuffer::new(OUT_FRAME * 8);
        let s = shared.clone();
        let worker = std::thread::Builder::new()
            .name("deepfilter".into())
            .spawn(move || run(&s, input, output))
            .ok()
            .map(|h| h.thread().clone());
        if worker.is_none() {
            shared.state.store(FAILED, Ordering::Relaxed);
        }
        DeepFilter {
            shared,
            worker,
            to_worker,
            from_worker,
            sent: 0,
            received: 0,
        }
    }

    /// Replaces `frame` with the denoised signal and returns the speech
    /// probability, or leaves it alone and returns `None` when no result is
    /// ready (still loading, or the worker fell behind).
    ///
    /// `level_gain` is the gain the chain applies later. The model judges
    /// speech by absolute level and mutes a quiet microphone entirely (it
    /// treats -50 dBFS speech as silence), so it sees the signal at the level
    /// the listener will hear and the result is scaled back afterwards.
    pub fn process(&mut self, frame: &mut [f32], level_gain: f32) -> Option<f32> {
        let g = level_gain.clamp(1.0, MAX_LEVEL_GAIN);
        if self.shared.state.load(Ordering::Acquire) != READY {
            return None;
        }
        if let Ok(mut chunk) = self.to_worker.write_chunk(FRAME) {
            let (a, b) = chunk.as_mut_slices();
            for (d, s) in a.iter_mut().chain(b).zip(frame.iter()) {
                *d = *s * g;
            }
            chunk.commit_all();
            self.sent += 1;
            if let Some(w) = &self.worker {
                w.unpark();
            }
        }
        // The result for the previous frame; anything older came too late.
        let want = self.sent.checked_sub(2)?;
        while self.received < want && self.from_worker.slots() >= OUT_FRAME {
            if let Ok(c) = self.from_worker.read_chunk(OUT_FRAME) {
                c.commit_all();
            }
            self.received += 1;
        }
        if self.received != want {
            return None;
        }
        let chunk = self.from_worker.read_chunk(OUT_FRAME).ok()?;
        self.received += 1;
        let (a, b) = chunk.as_slices();
        let all = a.iter().chain(b);
        // The gain may have moved since this frame went in; at slider speed
        // that is a fraction of a dB for one frame.
        for (d, s) in frame.iter_mut().zip(all.clone()) {
            *d = *s / g;
        }
        let vad = all.last().copied().unwrap_or(0.0);
        chunk.commit_all();
        Some(vad)
    }

    #[cfg(test)]
    pub fn ready(&self) -> bool {
        self.shared.state.load(Ordering::Acquire) != LOADING
    }
}

impl Drop for DeepFilter {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(w) = &self.worker {
            w.unpark();
        }
    }
}

fn run(shared: &Shared, mut input: rtrb::Consumer<f32>, mut output: rtrb::Producer<f32>) {
    let model = DfTract::new(DfParams::default(), &RuntimeParams::default_with_ch(1));
    let mut model = match model {
        Ok(m) if m.hop_size == FRAME && m.sr == super::SAMPLE_RATE as usize => m,
        Ok(_) => {
            eprintln!("DeepFilterNet: unexpected model frame size");
            shared.state.store(FAILED, Ordering::Release);
            return;
        }
        Err(e) => {
            eprintln!("DeepFilterNet: {e}");
            shared.state.store(FAILED, Ordering::Release);
            return;
        }
    };
    shared.state.store(READY, Ordering::Release);
    let mut noisy = Array2::<f32>::zeros((1, FRAME));
    let mut enhanced = Array2::<f32>::zeros((1, FRAME));
    while !shared.stop.load(Ordering::Relaxed) {
        while let Ok(chunk) = input.read_chunk(FRAME) {
            let (a, b) = chunk.as_slices();
            let buf = noisy.as_slice_mut().expect("contiguous");
            buf[..a.len()].copy_from_slice(a);
            buf[a.len()..].copy_from_slice(b);
            chunk.commit_all();
            // Local SNR runs from -15 dB (silence) to 35 dB; speech sits well above 0.
            let vad = match model.process(noisy.view(), enhanced.view_mut()) {
                Ok(lsnr) => (lsnr / 15.0).clamp(0.0, 1.0),
                Err(_) => {
                    enhanced.assign(&noisy);
                    0.0
                }
            };
            if let Ok(mut out) = output.write_chunk(OUT_FRAME) {
                let samples = enhanced.as_slice().expect("contiguous");
                let (a, b) = out.as_mut_slices();
                for (d, s) in a.iter_mut().chain(b).zip(samples.iter().chain([&vad])) {
                    *d = *s;
                }
                out.commit_all();
            }
        }
        std::thread::park_timeout(Duration::from_millis(100));
    }
}
