//! Application state shared by the window and the tray menu. Every change goes
//! through here so both stay in sync and the audio engine always gets the
//! current parameters, even while the window is hidden.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use crate::audio::{DeviceList, Engine, EngineStatus, Route};
use crate::config::{
    BandType, ConfigStore, DeviceProfile, EqBand, EqMode, Preset, Settings, Sound,
};
use crate::dsp::{DspParams, DspShared};
use crate::i18n::tr;

pub type Shared = Arc<Mutex<Controller>>;

const BUILTIN_PREFIX: &str = "builtin:";

/// Built-in starting points (id, i18n key, preset).
pub fn builtin_presets() -> Vec<(&'static str, &'static str, Preset)> {
    let p = |gain: f32, bands: Vec<EqBand>, gate: f32| Preset {
        version: 2,
        gain_db: gain,
        flat_mode: bands.is_empty(),
        bands,
        noise_gate_threshold: gate,
        ..Default::default()
    };
    let b = |t: BandType, f: f32, g: f32, q: f32| EqBand {
        frequency: f,
        gain: g,
        q,
        band_type: t,
    };
    use BandType::*;
    let off = crate::dsp::dynamics::GATE_OFF_DB;
    vec![
        ("flat", "Preset_Flat", p(0.0, vec![], off)),
        ("boost12", "Preset_Boost12", p(12.0, vec![], off)),
        ("boost24", "Preset_Boost24", p(24.0, vec![], off)),
        (
            "clear",
            "Preset_ClearVoice",
            p(
                12.0,
                vec![
                    b(HighPass, 80.0, 0.0, 0.71),
                    b(Peaking, 250.0, -3.0, 1.0),
                    b(Peaking, 3500.0, 4.0, 0.9),
                    b(HighShelf, 10_000.0, 2.0, 1.0),
                ],
                off,
            ),
        ),
        (
            "warm",
            "Preset_WarmVoice",
            p(
                12.0,
                vec![
                    b(HighPass, 60.0, 0.0, 0.71),
                    b(LowShelf, 180.0, 3.0, 1.0),
                    b(Peaking, 3000.0, 1.5, 1.0),
                    b(HighShelf, 9000.0, -2.0, 1.0),
                ],
                off,
            ),
        ),
        (
            "stream",
            "Preset_Streaming",
            p(
                18.0,
                vec![
                    b(HighPass, 90.0, 0.0, 0.71),
                    b(Peaking, 300.0, -2.0, 1.0),
                    b(Peaking, 4000.0, 3.0, 1.0),
                ],
                -50.0,
            ),
        ),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PresetId {
    Builtin(&'static str),
    User(String),
}

impl PresetId {
    fn key(&self) -> String {
        match self {
            PresetId::Builtin(id) => format!("{BUILTIN_PREFIX}{id}"),
            PresetId::User(n) => n.clone(),
        }
    }
}

pub struct Controller {
    pub settings: Settings,
    pub presets: Vec<Preset>,
    pub profiles: Vec<DeviceProfile>,
    pub store: ConfigStore,
    pub engine: Engine,
    /// Bumped on every change; the tray rebuilds its menu when it moves.
    pub version: u64,
    dirty: bool,
    notify: Arc<dyn Fn() + Send + Sync>,
}

impl Controller {
    pub fn new(
        store: ConfigStore,
        notify: Arc<dyn Fn() + Send + Sync>,
        devices_dirty: Arc<AtomicBool>,
    ) -> Shared {
        let mut settings = store.load_settings();
        crate::i18n::set_language(&settings.language);
        settings.language = crate::i18n::current().to_string();
        let presets = store.load_presets();
        let profiles = store.load_profiles();
        let engine = Engine::spawn(
            Arc::new(DspShared::default()),
            devices_dirty,
            notify.clone(),
        );
        let c = Controller {
            settings,
            presets,
            profiles,
            store,
            engine,
            version: 1,
            dirty: true,
            notify,
        };
        c.push_params();
        c.push_route();
        let shared = Arc::new(Mutex::new(c));
        spawn_saver(Arc::downgrade(&shared));
        shared
    }

    pub fn dsp(&self) -> &Arc<DspShared> {
        &self.engine.dsp
    }

    pub fn status(&self) -> EngineStatus {
        self.engine.status()
    }

    pub fn devices(&self) -> DeviceList {
        self.engine.devices()
    }

    fn changed(&mut self) {
        self.version += 1;
        self.dirty = true;
        (self.notify)();
    }

    fn push_params(&self) {
        let s = &self.settings;
        self.engine.dsp.set_params(DspParams {
            boost: s.boost_enabled,
            gain_db: s.sound.gain_db,
            bands: s.sound.active_bands().to_vec(),
            noise_suppression: s.sound.noise_suppression,
            suppression_strength: s.sound.suppression_strength,
            gate_db: s.sound.noise_gate_db,
            limiter_ceiling_db: s.limiter_ceiling_db,
        });
    }

    fn push_route(&self) {
        let s = &self.settings;
        let route =
            (!s.input_device_id.is_empty() && !s.output_device_id.is_empty()).then(|| Route {
                input_id: s.input_device_id.clone(),
                output_id: s.output_device_id.clone(),
            });
        self.engine.set_route(route);
    }

    /// Applies a sound edit (from a slider, the EQ graph or the tray).
    pub fn edit_sound(&mut self, f: impl FnOnce(&mut Sound)) {
        let before = self.settings.sound.clone();
        f(&mut self.settings.sound);
        let max = self.settings.max_gain_db();
        let s = &mut self.settings.sound;
        s.gain_db = s.gain_db.clamp(-30.0, max);
        s.suppression_strength = s.suppression_strength.clamp(0.0, 1.0);
        s.noise_gate_db = s
            .noise_gate_db
            .clamp(crate::dsp::dynamics::GATE_OFF_DB, 0.0);
        s.bands.truncate(crate::config::MAX_EQ_POINTS);
        if self.settings.sound != before {
            self.push_params();
            self.changed();
        }
    }

    pub fn edit_settings(&mut self, f: impl FnOnce(&mut Settings)) {
        let before = self.settings.clone();
        f(&mut self.settings);
        if self.settings != before {
            if self.settings.language != before.language {
                crate::i18n::set_language(&self.settings.language);
            }
            if !self.settings.unlock_limit {
                let max = self.settings.max_gain_db();
                self.settings.sound.gain_db = self.settings.sound.gain_db.min(max);
            }
            self.push_params();
            self.changed();
        }
    }

    pub fn set_boost(&mut self, on: bool) {
        self.edit_settings(|s| s.boost_enabled = on);
    }

    pub fn set_input(&mut self, id: &str) {
        let name = self
            .devices()
            .input_name(id)
            .unwrap_or_default()
            .to_string();
        self.settings.input_device_id = id.to_string();
        if !name.is_empty() {
            self.settings.input_device_name = name;
        }
        self.push_route();
        self.changed();
    }

    pub fn set_output(&mut self, id: &str) {
        let name = self
            .devices()
            .output_name(id)
            .unwrap_or_default()
            .to_string();
        self.settings.output_device_id = id.to_string();
        if !name.is_empty() {
            self.settings.output_device_name = name;
        }
        self.push_route();
        self.changed();
    }

    // ---- presets -------------------------------------------------------------

    pub fn current_preset(&self) -> Option<PresetId> {
        let key = self.settings.current_preset.as_deref()?;
        if let Some(id) = key.strip_prefix(BUILTIN_PREFIX) {
            builtin_presets()
                .into_iter()
                .find(|(i, _, _)| *i == id)
                .map(|(i, _, _)| PresetId::Builtin(i))
        } else {
            self.presets
                .iter()
                .any(|p| p.name == key)
                .then(|| PresetId::User(key.to_string()))
        }
    }

    fn preset(&self, id: &PresetId) -> Option<Preset> {
        match id {
            PresetId::Builtin(b) => builtin_presets()
                .into_iter()
                .find(|(i, _, _)| i == b)
                .map(|(_, _, p)| p),
            PresetId::User(n) => self.presets.iter().find(|p| &p.name == n).cloned(),
        }
    }

    pub fn preset_label(&self, id: &PresetId) -> String {
        match id {
            PresetId::Builtin(b) => builtin_presets()
                .into_iter()
                .find(|(i, _, _)| i == b)
                .map(|(_, k, _)| tr(k))
                .unwrap_or_default(),
            PresetId::User(n) => n.clone(),
        }
    }

    /// Whether the current sound differs from the selected preset.
    pub fn preset_modified(&self) -> bool {
        match self.current_preset().and_then(|id| self.preset(&id)) {
            Some(p) => !p
                .to_sound(&self.settings.sound)
                .approx_eq(&self.settings.sound),
            None => false,
        }
    }

    pub fn apply_preset(&mut self, id: &PresetId) {
        let Some(p) = self.preset(id) else { return };
        let sound = p.to_sound(&self.settings.sound);
        self.settings.current_preset = Some(id.key());
        self.edit_sound(|s| *s = sound);
        self.changed();
    }

    /// Discards edits made since the preset was applied.
    pub fn revert_preset(&mut self) {
        if let Some(id) = self.current_preset() {
            self.apply_preset(&id);
        }
    }

    pub fn preset_exists(&self, name: &str) -> bool {
        self.presets
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(name.trim()))
    }

    /// Saves the current sound under `name` (overwriting a preset with that name).
    pub fn save_preset_as(&mut self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(tr("Msg_PresetNameRequired"));
        }
        // Keep the existing spelling when overwriting case-insensitively.
        let name = self
            .presets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| name.to_string());
        let mut preset = Preset::from_sound(&name, &self.settings.sound);
        if let Some(old) = self.presets.iter().find(|p| p.name == name) {
            // Keep the v1.x view fields of an overwritten file.
            preset.min_freq = old.min_freq;
            preset.max_freq = old.max_freq;
        }
        self.store.save_preset(&preset).map_err(|e| e.to_string())?;
        self.presets.retain(|p| p.name != name);
        self.presets.push(preset);
        self.presets.sort_by_key(|p| p.name.to_lowercase());
        self.settings.current_preset = Some(name);
        self.changed();
        Ok(())
    }

    pub fn rename_preset(&mut self, old: &str, new: &str) -> Result<(), String> {
        let new = new.trim();
        if new.is_empty() {
            return Err(tr("Msg_PresetNameRequired"));
        }
        if new != old && self.preset_exists(new) && !old.eq_ignore_ascii_case(new) {
            return Err(crate::i18n::trf("Msg_PresetExists", &[&new]));
        }
        let Some(mut p) = self.presets.iter().find(|p| p.name == old).cloned() else {
            return Ok(());
        };
        p.name = new.to_string();
        self.store.save_preset(&p).map_err(|e| e.to_string())?;
        if crate::config::file_name(old) != crate::config::file_name(new) {
            self.store.delete_preset(old);
        }
        self.presets.retain(|x| x.name != old);
        self.presets.push(p);
        self.presets.sort_by_key(|p| p.name.to_lowercase());
        if self.settings.current_preset.as_deref() == Some(old) {
            self.settings.current_preset = Some(new.to_string());
        }
        self.changed();
        Ok(())
    }

    pub fn delete_preset(&mut self, name: &str) {
        self.store.delete_preset(name);
        self.presets.retain(|p| p.name != name);
        if self.settings.current_preset.as_deref() == Some(name) {
            self.settings.current_preset = None;
        }
        self.changed();
    }

    // ---- device profiles -----------------------------------------------------

    /// The profile matching the devices in use, if any.
    pub fn active_profile(&self) -> Option<&DeviceProfile> {
        let s = &self.settings;
        self.profiles.iter().find(|p| {
            p.input_device_id == s.input_device_id && p.output_device_id == s.output_device_id
        })
    }

    pub fn apply_profile(&mut self, name: &str) {
        let Some(p) = self.profiles.iter().find(|p| p.name == name).cloned() else {
            return;
        };
        let s = &mut self.settings;
        if !p.input_device_id.is_empty() {
            s.input_device_id = p.input_device_id.clone();
            s.input_device_name = p.input_device_name.clone();
        }
        if !p.output_device_id.is_empty() {
            s.output_device_id = p.output_device_id.clone();
            s.output_device_name = p.output_device_name.clone();
        }
        s.current_device_profile = Some(p.name);
        self.push_route();
        self.changed();
    }

    pub fn profile_exists(&self, name: &str) -> bool {
        self.profiles
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(name.trim()))
    }

    pub fn save_profile_as(&mut self, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(tr("Msg_DeviceProfileNameRequired"));
        }
        let name = self
            .profiles
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| name.to_string());
        let devices = self.devices();
        let s = &self.settings;
        let p = DeviceProfile {
            name: name.clone(),
            input_device_id: s.input_device_id.clone(),
            output_device_id: s.output_device_id.clone(),
            input_device_name: devices
                .input_name(&s.input_device_id)
                .unwrap_or(&s.input_device_name)
                .to_string(),
            output_device_name: devices
                .output_name(&s.output_device_id)
                .unwrap_or(&s.output_device_name)
                .to_string(),
        };
        self.store.save_profile(&p).map_err(|e| e.to_string())?;
        self.profiles.retain(|x| x.name != name);
        self.profiles.push(p);
        self.profiles.sort_by_key(|p| p.name.to_lowercase());
        self.settings.current_device_profile = Some(name);
        self.changed();
        Ok(())
    }

    pub fn delete_profile(&mut self, name: &str) {
        self.store.delete_profile(name);
        self.profiles.retain(|p| p.name != name);
        if self.settings.current_device_profile.as_deref() == Some(name) {
            self.settings.current_device_profile = None;
        }
        self.changed();
    }

    // ---- EQ helpers ----------------------------------------------------------

    pub fn set_mode(&mut self, mode: EqMode) {
        self.edit_sound(|s| s.mode = mode);
    }

    pub fn save_now(&mut self) {
        if self.dirty {
            self.dirty = false;
            if let Err(e) = self.store.save_settings(&self.settings) {
                eprintln!("failed to save settings: {e}");
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.save_now();
        self.engine.shutdown();
    }
}

pub fn lock(c: &Shared) -> MutexGuard<'_, Controller> {
    c.lock().unwrap_or_else(|e| e.into_inner())
}

/// Writes settings at most twice a second, off the UI thread.
fn spawn_saver(c: std::sync::Weak<Mutex<Controller>>) {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::Relaxed) {
        return;
    }
    thread::Builder::new()
        .name("settings-saver".into())
        .spawn(move || loop {
            thread::sleep(Duration::from_millis(500));
            let Some(c) = c.upgrade() else { return };
            lock(&c).save_now();
        })
        .expect("spawn saver");
}
