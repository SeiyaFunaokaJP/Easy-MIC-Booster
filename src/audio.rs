//! Device enumeration and the capture → DSP → render streams.
//!
//! A supervisor thread owns the cpal streams (they are not `Send` everywhere)
//! and keeps the route alive: it restarts after stream errors, silent capture
//! (e.g. after sleep), and device or default-device changes.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, DeviceId, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig};

use crate::config::DEFAULT_DEVICE_ID;
use crate::dsp::resample::Resampler;
use crate::dsp::{DspShared, Processor, BLOCK, SAMPLE_RATE};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceList {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
    pub default_input: Option<String>,
    pub default_output: Option<String>,
}

impl DeviceList {
    pub fn input_name(&self, id: &str) -> Option<&str> {
        find_name(&self.inputs, id, self.default_input.as_deref())
    }
    pub fn output_name(&self, id: &str) -> Option<&str> {
        find_name(&self.outputs, id, self.default_output.as_deref())
    }
}

fn find_name<'a>(list: &'a [DeviceInfo], id: &str, default: Option<&str>) -> Option<&'a str> {
    let id = if id == DEFAULT_DEVICE_ID {
        default?
    } else {
        id
    };
    list.iter().find(|d| d.id == id).map(|d| d.name.as_str())
}

#[derive(Clone, Debug, PartialEq)]
pub enum EngineStatus {
    Idle,
    Starting,
    Running {
        input_rate: u32,
        output_rate: u32,
        latency_ms: f32,
    },
    /// Waiting for a device that is unplugged/disabled (`true` = input).
    MissingDevice {
        input: bool,
        output: bool,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub input_id: String,
    pub output_id: String,
}

enum Command {
    SetRoute(Option<Route>),
    RefreshDevices,
    Shutdown(mpsc::Sender<()>),
}

/// Handle to the supervisor thread; cheap to clone.
#[derive(Clone)]
pub struct Engine {
    tx: Sender<Command>,
    pub dsp: Arc<DspShared>,
    state: Arc<Mutex<(EngineStatus, DeviceList)>>,
    pub version: Arc<AtomicU64>,
}

impl Engine {
    /// `notify` is called (from the supervisor thread) whenever status or devices change.
    pub fn spawn(
        dsp: Arc<DspShared>,
        devices_dirty: Arc<AtomicBool>,
        notify: Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        let (tx, rx) = mpsc::channel();
        let devices = enumerate();
        let state = Arc::new(Mutex::new((EngineStatus::Idle, devices)));
        let version = Arc::new(AtomicU64::new(1));
        let engine = Engine {
            tx,
            dsp: dsp.clone(),
            state: state.clone(),
            version: version.clone(),
        };
        thread::Builder::new()
            .name("audio-supervisor".into())
            .spawn(move || Supervisor::new(dsp, state, version, devices_dirty, notify).run(rx))
            .expect("spawn audio supervisor");
        engine
    }

    pub fn set_route(&self, route: Option<Route>) {
        let _ = self.tx.send(Command::SetRoute(route));
    }

    pub fn refresh_devices(&self) {
        let _ = self.tx.send(Command::RefreshDevices);
    }

    pub fn status(&self) -> EngineStatus {
        self.state.lock().unwrap().0.clone()
    }

    pub fn devices(&self) -> DeviceList {
        self.state.lock().unwrap().1.clone()
    }

    /// Stops the streams and waits (briefly) for the supervisor to finish.
    pub fn shutdown(&self) {
        let (tx, rx) = mpsc::channel();
        if self.tx.send(Command::Shutdown(tx)).is_ok() {
            let _ = rx.recv_timeout(Duration::from_secs(2));
        }
    }
}

/// On Windows `name()` is only the endpoint description ("Microphone"); the
/// friendly name ("Microphone (HyperX QuadCast 2 S)", as shown by Windows and
/// v1.x) is the first extended line.
fn display_name(d: &cpal::DeviceDescription) -> String {
    if cfg!(windows) {
        if let Some(friendly) = d.extended().first() {
            return friendly.clone();
        }
    }
    d.name().to_string()
}

pub fn enumerate() -> DeviceList {
    let host = cpal::default_host();
    let collect = |devs: Result<_, _>| -> Vec<DeviceInfo> {
        let devs: Box<dyn Iterator<Item = Device>> = match devs {
            Ok(d) => Box::new(d),
            Err(_) => Box::new(std::iter::empty()),
        };
        let mut list: Vec<DeviceInfo> = devs
            .filter_map(|d| {
                let id = d.id().ok()?.1;
                let name = d
                    .description()
                    .map(|x| display_name(&x))
                    .unwrap_or_else(|_| id.clone());
                Some(DeviceInfo { id, name })
            })
            .collect();
        list.sort_by_key(|a| a.name.to_lowercase());
        list.dedup_by(|a, b| a.id == b.id);
        list
    };
    DeviceList {
        inputs: collect(
            host.input_devices()
                .map(|d| d.collect::<Vec<_>>().into_iter()),
        ),
        outputs: collect(
            host.output_devices()
                .map(|d| d.collect::<Vec<_>>().into_iter()),
        ),
        default_input: host
            .default_input_device()
            .and_then(|d| d.id().ok())
            .map(|i| i.1),
        default_output: host
            .default_output_device()
            .and_then(|d| d.id().ok())
            .map(|i| i.1),
    }
}

struct Running {
    _input: Stream,
    _output: Stream,
    input_id: String,
    output_id: String,
}

struct Supervisor {
    dsp: Arc<DspShared>,
    state: Arc<Mutex<(EngineStatus, DeviceList)>>,
    version: Arc<AtomicU64>,
    devices_dirty: Arc<AtomicBool>,
    notify: Arc<dyn Fn() + Send + Sync>,
    route: Option<Route>,
    running: Option<Running>,
    health: Arc<StreamHealth>,
    retry_at: Option<Instant>,
    retry_delay: Duration,
    last_poll: Instant,
}

#[derive(Default)]
struct StreamHealth {
    /// Milliseconds since `epoch` of the last capture callback.
    last_input_ms: AtomicU64,
    fault: Mutex<Option<String>>,
    epoch: Mutex<Option<Instant>>,
}

impl StreamHealth {
    fn now_ms(&self) -> u64 {
        self.epoch
            .lock()
            .unwrap()
            .map(|e| e.elapsed().as_millis() as u64)
            .unwrap_or(0)
    }
}

impl Supervisor {
    fn new(
        dsp: Arc<DspShared>,
        state: Arc<Mutex<(EngineStatus, DeviceList)>>,
        version: Arc<AtomicU64>,
        devices_dirty: Arc<AtomicBool>,
        notify: Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        Supervisor {
            dsp,
            state,
            version,
            devices_dirty,
            notify,
            route: None,
            running: None,
            health: Arc::new(StreamHealth::default()),
            retry_at: None,
            retry_delay: Duration::from_millis(500),
            last_poll: Instant::now(),
        }
    }

    fn publish_status(&self, status: EngineStatus) {
        let mut st = self.state.lock().unwrap();
        if st.0 != status {
            st.0 = status;
            drop(st);
            self.version.fetch_add(1, Ordering::Relaxed);
            (self.notify)();
        }
    }

    fn devices(&self) -> DeviceList {
        self.state.lock().unwrap().1.clone()
    }

    fn run(mut self, rx: Receiver<Command>) {
        loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(Command::SetRoute(r)) => {
                    if r != self.route {
                        self.route = r;
                        self.stop();
                        self.retry_delay = Duration::from_millis(500);
                        self.try_start();
                    }
                }
                Ok(Command::RefreshDevices) => self.refresh_devices(),
                Ok(Command::Shutdown(done)) => {
                    self.stop();
                    let _ = done.send(());
                    return;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    self.stop();
                    return;
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
            self.check_health();
            // Windows pushes endpoint notifications; elsewhere poll every 3 s.
            let poll = !cfg!(windows) && self.last_poll.elapsed() > Duration::from_secs(3);
            if self.devices_dirty.swap(false, Ordering::Relaxed) || poll {
                if !poll {
                    // Notifications come in bursts (e.g. Bluetooth registers several endpoints).
                    thread::sleep(Duration::from_millis(400));
                    self.devices_dirty.store(false, Ordering::Relaxed);
                }
                self.refresh_devices();
            }
            if self.running.is_none() && self.retry_at.is_some_and(|t| Instant::now() >= t) {
                self.try_start();
            }
        }
    }

    fn refresh_devices(&mut self) {
        self.last_poll = Instant::now();
        let list = enumerate();
        {
            let mut st = self.state.lock().unwrap();
            if st.1 == list {
                drop(st);
                // Same devices; still retry a route that is waiting for one.
                if self.running.is_none() && self.route.is_some() {
                    self.try_start();
                }
                return;
            }
            st.1 = list;
        }
        self.version.fetch_add(1, Ordering::Relaxed);
        (self.notify)();
        // Restart only if the endpoints in use changed (e.g. a new default device).
        let wanted = self.resolve();
        match (&self.running, wanted) {
            (Some(r), Some((i, o))) if r.input_id == i && r.output_id == o => {}
            _ => {
                self.stop();
                self.try_start();
            }
        }
    }

    fn check_health(&mut self) {
        let Some(_) = &self.running else { return };
        let fault = self.health.fault.lock().unwrap().take();
        let silent = self
            .health
            .now_ms()
            .saturating_sub(self.health.last_input_ms.load(Ordering::Relaxed))
            > 3000;
        if fault.is_some() || silent {
            let reason = fault.unwrap_or_else(|| "no audio from the input device".into());
            eprintln!("audio stream fault: {reason}; restarting");
            self.stop();
            self.publish_status(EngineStatus::Starting);
            self.schedule_retry();
        }
    }

    fn schedule_retry(&mut self) {
        self.retry_at = Some(Instant::now() + self.retry_delay);
        self.retry_delay = (self.retry_delay * 2).min(Duration::from_secs(5));
    }

    /// Endpoint ids the route currently maps to, if both devices exist.
    fn resolve(&self) -> Option<(String, String)> {
        let route = self.route.as_ref()?;
        let list = self.devices();
        let pick = |id: &str, default: &Option<String>, all: &[DeviceInfo]| -> Option<String> {
            if id == DEFAULT_DEVICE_ID {
                default.clone()
            } else {
                all.iter().any(|d| d.id == id).then(|| id.to_string())
            }
        };
        let i = pick(&route.input_id, &list.default_input, &list.inputs);
        let o = pick(&route.output_id, &list.default_output, &list.outputs);
        Some((i?, o?))
    }

    fn stop(&mut self) {
        if self.running.take().is_some() {
            self.dsp.meters.output_peak.take();
            self.dsp.meters.input_peak.take();
        }
        self.retry_at = None;
        if self.route.is_none() {
            self.publish_status(EngineStatus::Idle);
        }
    }

    fn try_start(&mut self) {
        self.retry_at = None;
        let Some(route) = self.route.clone() else {
            self.publish_status(EngineStatus::Idle);
            return;
        };
        let Some((input_id, output_id)) = self.resolve() else {
            let list = self.devices();
            let has = |id: &str, def: &Option<String>, all: &[DeviceInfo]| {
                if id == DEFAULT_DEVICE_ID {
                    def.is_some()
                } else {
                    all.iter().any(|d| d.id == id)
                }
            };
            self.publish_status(EngineStatus::MissingDevice {
                input: !has(&route.input_id, &list.default_input, &list.inputs),
                output: !has(&route.output_id, &list.default_output, &list.outputs),
            });
            return;
        };
        self.publish_status(EngineStatus::Starting);
        match self.start_streams(&input_id, &output_id) {
            Ok((running, status)) => {
                self.running = Some(running);
                self.retry_delay = Duration::from_millis(500);
                self.publish_status(status);
            }
            Err(e) => {
                eprintln!("failed to start audio: {e}");
                self.publish_status(EngineStatus::Error(e));
                self.schedule_retry();
            }
        }
    }

    fn start_streams(
        &mut self,
        input_id: &str,
        output_id: &str,
    ) -> Result<(Running, EngineStatus), String> {
        let host = cpal::default_host();
        let find = |id: &str| host.device_by_id(&DeviceId(host.id(), id.to_string()));
        let input = find(input_id).ok_or("input device not found")?;
        let output = find(output_id).ok_or("output device not found")?;
        let in_cfg = input.default_input_config().map_err(|e| e.to_string())?;
        let out_cfg = output.default_output_config().map_err(|e| e.to_string())?;
        let in_rate = in_cfg.sample_rate();
        let out_rate = out_cfg.sample_rate();

        // ~0.5 s of headroom; normal operation keeps it nearly empty.
        let (producer, consumer) = rtrb::RingBuffer::<f32>::new(out_rate as usize / 2);

        self.health = Arc::new(StreamHealth::default());
        *self.health.epoch.lock().unwrap() = Some(Instant::now());
        let stage = InputStage::new(
            in_cfg.channels() as usize,
            in_rate,
            out_rate,
            Processor::new(self.dsp.clone()),
            producer,
            self.health.clone(),
        );
        let out_stage = OutputStage::new(out_cfg.channels() as usize, out_rate, consumer);

        let in_stream = build_input(
            &input,
            in_cfg.sample_format(),
            &in_cfg.config(),
            stage,
            self.health.clone(),
        )?;
        let out_stream = build_output(
            &output,
            out_cfg.sample_format(),
            &out_cfg.config(),
            out_stage,
            self.health.clone(),
        )?;
        out_stream.play().map_err(|e| e.to_string())?;
        in_stream.play().map_err(|e| e.to_string())?;

        let latency_ms = (BLOCK as f32 / SAMPLE_RATE as f32 * 1000.0) + OutputStage::CUSHION_MS;
        Ok((
            Running {
                _input: in_stream,
                _output: out_stream,
                input_id: input_id.to_string(),
                output_id: output_id.to_string(),
            },
            EngineStatus::Running {
                input_rate: in_rate,
                output_rate: out_rate,
                latency_ms,
            },
        ))
    }
}

/// Lives in the capture callback: downmix, resample to 48 kHz, process in
/// blocks, resample to the output rate, hand to the render callback.
struct InputStage {
    channels: usize,
    to_dsp: Option<Resampler>,
    from_dsp: Option<Resampler>,
    mono: Vec<f32>,
    pending: Vec<f32>,
    block: Vec<f32>,
    out: Vec<f32>,
    processor: Processor,
    producer: rtrb::Producer<f32>,
    health: Arc<StreamHealth>,
}

impl InputStage {
    fn new(
        channels: usize,
        in_rate: u32,
        out_rate: u32,
        processor: Processor,
        producer: rtrb::Producer<f32>,
        health: Arc<StreamHealth>,
    ) -> Self {
        let resampler = |a: u32, b: u32| (a != b).then(|| Resampler::new(a, b));
        InputStage {
            channels: channels.max(1),
            to_dsp: resampler(in_rate, SAMPLE_RATE),
            from_dsp: resampler(SAMPLE_RATE, out_rate),
            mono: Vec::with_capacity(16_384),
            pending: Vec::with_capacity(32_768),
            block: vec![0.0; BLOCK],
            out: Vec::with_capacity(32_768),
            processor,
            producer,
            health,
        }
    }

    fn push<T: Sample>(&mut self, data: &[T])
    where
        f32: FromSample<T>,
    {
        self.health
            .last_input_ms
            .store(self.health.now_ms(), Ordering::Relaxed);
        let ch = self.channels;
        let scale = 1.0 / ch as f32;
        self.mono.clear();
        for frame in data.chunks_exact(ch) {
            let sum: f32 = frame.iter().map(|s| f32::from_sample(*s)).sum();
            self.mono.push(sum * scale);
        }
        match &mut self.to_dsp {
            Some(r) => r.process(&self.mono, &mut self.pending),
            None => self.pending.extend_from_slice(&self.mono),
        }

        let mut consumed = 0;
        self.out.clear();
        while self.pending.len() - consumed >= BLOCK {
            self.block
                .copy_from_slice(&self.pending[consumed..consumed + BLOCK]);
            consumed += BLOCK;
            self.processor.process(&mut self.block);
            match &mut self.from_dsp {
                Some(r) => r.process(&self.block, &mut self.out),
                None => self.out.extend_from_slice(&self.block),
            }
        }
        self.pending.drain(..consumed);

        if !self.out.is_empty() {
            let n = self.out.len().min(self.producer.slots());
            if let Ok(chunk) = self.producer.write_chunk_uninit(n) {
                chunk.fill_from_iter(self.out.iter().copied());
            }
        }
    }
}

struct OutputStage {
    channels: usize,
    consumer: rtrb::Consumer<f32>,
    cushion: usize,
    primed: bool,
}

impl OutputStage {
    /// Extra buffering kept to absorb jitter between the two device clocks.
    const CUSHION_MS: f32 = 10.0;

    fn new(channels: usize, rate: u32, consumer: rtrb::Consumer<f32>) -> Self {
        OutputStage {
            channels: channels.max(1),
            consumer,
            cushion: (rate as f32 * Self::CUSHION_MS / 1000.0) as usize,
            primed: false,
        }
    }

    fn pull<T: Sample + FromSample<f32>>(&mut self, data: &mut [T]) {
        let frames = data.len() / self.channels;
        let avail = self.consumer.slots();
        let target = frames + self.cushion;

        // Clock drift / a stalled output: drop the backlog instead of growing latency.
        if avail > target + frames.max(self.cushion) * 3 {
            if let Ok(c) = self.consumer.read_chunk(avail - target) {
                c.commit_all();
            }
        }
        if !self.primed && self.consumer.slots() >= target {
            self.primed = true;
        }

        let take = if self.primed {
            frames.min(self.consumer.slots())
        } else {
            0
        };
        if take < frames {
            // Underrun: play silence and wait for the cushion to refill.
            self.primed = false;
        }
        let mut written = 0;
        if take > 0 {
            if let Ok(chunk) = self.consumer.read_chunk(take) {
                let (a, b) = chunk.as_slices();
                for &s in a.iter().chain(b) {
                    let v = T::from_sample(s);
                    let base = written * self.channels;
                    for o in &mut data[base..base + self.channels] {
                        *o = v;
                    }
                    written += 1;
                }
                chunk.commit_all();
            }
        }
        for o in &mut data[written * self.channels..] {
            *o = T::EQUILIBRIUM;
        }
    }
}

fn error_handler(health: Arc<StreamHealth>) -> impl FnMut(cpal::StreamError) + Send + 'static {
    move |e| {
        *health.fault.lock().unwrap() = Some(e.to_string());
    }
}

macro_rules! for_formats {
    ($fmt:expr, $t:ident => $body:expr) => {
        match $fmt {
            SampleFormat::F32 => {
                type $t = f32;
                $body
            }
            SampleFormat::I16 => {
                type $t = i16;
                $body
            }
            SampleFormat::I32 => {
                type $t = i32;
                $body
            }
            SampleFormat::U16 => {
                type $t = u16;
                $body
            }
            SampleFormat::F64 => {
                type $t = f64;
                $body
            }
            SampleFormat::I8 => {
                type $t = i8;
                $body
            }
            SampleFormat::U8 => {
                type $t = u8;
                $body
            }
            SampleFormat::U32 => {
                type $t = u32;
                $body
            }
            other => Err(format!("unsupported sample format {other}")),
        }
    };
}

fn build_input(
    device: &Device,
    format: SampleFormat,
    config: &StreamConfig,
    mut stage: InputStage,
    health: Arc<StreamHealth>,
) -> Result<Stream, String> {
    fn build<T: SizedSample>(
        device: &Device,
        config: &StreamConfig,
        mut stage: InputStage,
        health: Arc<StreamHealth>,
    ) -> Result<Stream, String>
    where
        f32: FromSample<T>,
    {
        device
            .build_input_stream(
                config,
                move |data: &[T], _| stage.push(data),
                error_handler(health),
                None,
            )
            .map_err(|e| e.to_string())
    }
    let _ = &mut stage;
    for_formats!(format, T => build::<T>(device, config, stage, health))
}

fn build_output(
    device: &Device,
    format: SampleFormat,
    config: &StreamConfig,
    stage: OutputStage,
    health: Arc<StreamHealth>,
) -> Result<Stream, String> {
    fn build<T: SizedSample + FromSample<f32>>(
        device: &Device,
        config: &StreamConfig,
        mut stage: OutputStage,
        health: Arc<StreamHealth>,
    ) -> Result<Stream, String> {
        device
            .build_output_stream(
                config,
                move |data: &mut [T], _| stage.pull(data),
                error_handler(health),
                None,
            )
            .map_err(|e| e.to_string())
    }
    for_formats!(format, T => build::<T>(device, config, stage, health))
}
