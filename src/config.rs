//! Settings, presets and device profiles.
//!
//! Files live where v1.x kept them (`%APPDATA%\EasyMICBooster` on Windows) so an
//! upgrade keeps everything: `config.ini` is imported once into `settings.json`
//! and left untouched, and the `Presets` / `DeviceProfiles` JSON files are read in
//! the v1.x schema and written in a superset of it.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::dsp::dynamics::GATE_OFF_DB;

pub const DEFAULT_DEVICE_ID: &str = "@DEFAULT_DEVICE";
pub const MAX_EQ_POINTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BandType {
    #[default]
    Peaking,
    LowShelf,
    HighShelf,
    HighPass,
    LowPass,
}

impl BandType {
    pub const ALL: [BandType; 5] = [
        BandType::Peaking,
        BandType::LowShelf,
        BandType::HighShelf,
        BandType::HighPass,
        BandType::LowPass,
    ];

    /// Whether the band has a gain (pass filters only have a cutoff and Q).
    pub fn has_gain(self) -> bool {
        !matches!(self, BandType::HighPass | BandType::LowPass)
    }

    pub fn has_q(self) -> bool {
        !matches!(self, BandType::LowShelf | BandType::HighShelf)
    }
}

// System.Text.Json wrote the enum as its integer value; keep that format.
impl Serialize for BandType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for BandType {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            N(u8),
            S(String),
        }
        Ok(match Raw::deserialize(d)? {
            Raw::N(1) => BandType::LowShelf,
            Raw::N(2) => BandType::HighShelf,
            Raw::N(3) => BandType::HighPass,
            Raw::N(4) => BandType::LowPass,
            Raw::N(_) => BandType::Peaking,
            Raw::S(s) => match s.as_str() {
                "LowShelf" => BandType::LowShelf,
                "HighShelf" => BandType::HighShelf,
                "HighPass" => BandType::HighPass,
                "LowPass" => BandType::LowPass,
                _ => BandType::Peaking,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EqBand {
    pub frequency: f32,
    pub gain: f32,
    #[serde(default = "default_q")]
    pub q: f32,
    #[serde(rename = "Type", default)]
    pub band_type: BandType,
}

fn default_q() -> f32 {
    1.0
}

impl EqBand {
    pub fn peaking(frequency: f32, gain: f32, q: f32) -> Self {
        EqBand {
            frequency,
            gain,
            q,
            band_type: BandType::Peaking,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EqMode {
    /// Only the overall gain (v1.x "Flat Mode").
    #[default]
    Simple,
    /// Gain plus the parametric EQ points.
    Equalizer,
}

/// Everything a preset stores. Also the "sound" part of [`Settings`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Sound {
    pub gain_db: f32,
    pub mode: EqMode,
    pub bands: Vec<EqBand>,
    pub noise_gate_db: f32,
    pub noise_suppression: bool,
    pub suppression_strength: f32,
}

impl Default for Sound {
    fn default() -> Self {
        Sound {
            gain_db: 0.0,
            mode: EqMode::Simple,
            bands: Vec::new(),
            noise_gate_db: GATE_OFF_DB,
            noise_suppression: false,
            suppression_strength: 1.0,
        }
    }
}

impl Sound {
    /// The bands that actually run (none in simple mode).
    pub fn active_bands(&self) -> &[EqBand] {
        match self.mode {
            EqMode::Simple => &[],
            EqMode::Equalizer => &self.bands,
        }
    }

    /// Equality that ignores float noise from round-tripping through JSON.
    pub fn approx_eq(&self, other: &Sound) -> bool {
        let close = |a: f32, b: f32| (a - b).abs() < 0.05;
        close(self.gain_db, other.gain_db)
            && self.mode == other.mode
            && close(self.noise_gate_db, other.noise_gate_db)
            && self.noise_suppression == other.noise_suppression
            && close(self.suppression_strength, other.suppression_strength)
            && (self.mode == EqMode::Simple
                || (self.bands.len() == other.bands.len()
                    && self.bands.iter().zip(&other.bands).all(|(a, b)| {
                        a.band_type == b.band_type
                            && (a.frequency - b.frequency).abs() < 0.5
                            && close(a.gain, b.gain)
                            && (a.q - b.q).abs() < 0.005
                    })))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub boost_enabled: bool,
    pub input_device_id: String,
    pub output_device_id: String,
    /// Last known names, shown while a device is unplugged.
    pub input_device_name: String,
    pub output_device_name: String,
    pub sound: Sound,
    pub unlock_limit: bool,
    pub limiter_ceiling_db: f32,
    pub current_preset: Option<String>,
    pub current_device_profile: Option<String>,
    pub language: String,
    pub check_updates: bool,
    pub close_to_tray: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            boost_enabled: true,
            input_device_id: DEFAULT_DEVICE_ID.into(),
            output_device_id: String::new(),
            input_device_name: String::new(),
            output_device_name: String::new(),
            sound: Sound::default(),
            unlock_limit: false,
            limiter_ceiling_db: -1.0,
            current_preset: None,
            current_device_profile: None,
            language: crate::i18n::detect_system_language(),
            check_updates: true,
            close_to_tray: true,
        }
    }
}

impl Settings {
    pub fn max_gain_db(&self) -> f32 {
        if self.unlock_limit {
            100.0
        } else {
            50.0
        }
    }
}

/// A user preset. Field names are the v1.x ones (PascalCase) plus new ones.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Preset {
    pub name: String,
    /// 2 for presets written by the Rust version; absent (0) in v1.x files.
    pub version: u32,
    pub gain_db: f32,
    pub bands: Vec<EqBand>,
    pub flat_mode: bool,
    pub noise_gate_threshold: f32,
    pub noise_suppression: Option<bool>,
    pub suppression_strength: Option<f32>,
    // v1.x view settings, kept so the files stay readable by v1.x.
    pub min_freq: f64,
    pub max_freq: f64,
    pub unlock_limit: bool,
}

impl Default for Preset {
    fn default() -> Self {
        Preset {
            name: String::new(),
            version: 0,
            gain_db: 0.0,
            bands: Vec::new(),
            flat_mode: true,
            noise_gate_threshold: GATE_OFF_DB,
            noise_suppression: None,
            suppression_strength: None,
            min_freq: 20.0,
            max_freq: 20_000.0,
            unlock_limit: false,
        }
    }
}

impl Preset {
    pub fn from_sound(name: &str, sound: &Sound) -> Self {
        Preset {
            name: name.to_string(),
            version: 2,
            gain_db: sound.gain_db,
            bands: sound.bands.clone(),
            flat_mode: sound.mode == EqMode::Simple,
            noise_gate_threshold: sound.noise_gate_db,
            noise_suppression: Some(sound.noise_suppression),
            suppression_strength: Some(sound.suppression_strength),
            ..Default::default()
        }
    }

    /// Converts to the current model. v1.x stored the flat-mode gain on every
    /// point and the point gains as absolute values with no master gain.
    pub fn to_sound(&self, current: &Sound) -> Sound {
        let (gain_db, bands, mode) = if self.version >= 2 {
            let mode = if self.flat_mode {
                EqMode::Simple
            } else {
                EqMode::Equalizer
            };
            (self.gain_db, self.bands.clone(), mode)
        } else if self.flat_mode {
            (
                self.bands.first().map(|b| b.gain).unwrap_or(0.0),
                Vec::new(),
                EqMode::Simple,
            )
        } else {
            (0.0, self.bands.clone(), EqMode::Equalizer)
        };
        Sound {
            gain_db,
            mode,
            bands,
            // v1.x clamped the gate to -80..0 dB internally.
            noise_gate_db: self.noise_gate_threshold.clamp(GATE_OFF_DB, 0.0),
            noise_suppression: self.noise_suppression.unwrap_or(current.noise_suppression),
            suppression_strength: self
                .suppression_strength
                .unwrap_or(current.suppression_strength),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase", default)]
pub struct DeviceProfile {
    pub name: String,
    pub input_device_id: String,
    pub output_device_id: String,
    pub input_device_name: String,
    pub output_device_name: String,
}

pub struct ConfigStore {
    dir: PathBuf,
}

impl ConfigStore {
    pub fn open() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::at(base.join("EasyMICBooster"))
    }

    pub fn at(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(dir.join("Presets"));
        let _ = fs::create_dir_all(dir.join("DeviceProfiles"));
        ConfigStore { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn presets_dir(&self) -> PathBuf {
        self.dir.join("Presets")
    }

    fn profiles_dir(&self) -> PathBuf {
        self.dir.join("DeviceProfiles")
    }

    fn settings_path(&self) -> PathBuf {
        self.dir.join("settings.json")
    }

    pub fn load_settings(&self) -> Settings {
        if let Ok(text) = fs::read_to_string(self.settings_path()) {
            if let Ok(s) = serde_json::from_str::<Settings>(&text) {
                return s;
            }
        }
        match fs::read_to_string(self.dir.join("config.ini")) {
            Ok(ini) => import_legacy_ini(&ini, &self.load_presets()),
            Err(_) => Settings::default(),
        }
    }

    pub fn save_settings(&self, s: &Settings) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(s).map_err(std::io::Error::other)?;
        write_atomic(&self.settings_path(), json.as_bytes())
    }

    pub fn load_presets(&self) -> Vec<Preset> {
        let mut list: Vec<Preset> = read_json_dir(&self.presets_dir());
        list.retain(|p| !p.name.trim().is_empty());
        list.sort_by_key(|p| p.name.to_lowercase());
        list
    }

    pub fn save_preset(&self, p: &Preset) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(p).map_err(std::io::Error::other)?;
        write_atomic(
            &self.presets_dir().join(file_name(&p.name)),
            json.as_bytes(),
        )
    }

    pub fn delete_preset(&self, name: &str) {
        let _ = fs::remove_file(self.presets_dir().join(file_name(name)));
    }

    pub fn load_profiles(&self) -> Vec<DeviceProfile> {
        let mut list: Vec<DeviceProfile> = read_json_dir(&self.profiles_dir());
        list.retain(|p| !p.name.trim().is_empty());
        list.sort_by_key(|p| p.name.to_lowercase());
        list
    }

    pub fn save_profile(&self, p: &DeviceProfile) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(p).map_err(std::io::Error::other)?;
        write_atomic(
            &self.profiles_dir().join(file_name(&p.name)),
            json.as_bytes(),
        )
    }

    pub fn delete_profile(&self, name: &str) {
        let _ = fs::remove_file(self.profiles_dir().join(file_name(name)));
    }
}

fn read_json_dir<T: for<'de> Deserialize<'de>>(dir: &Path) -> Vec<T> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("json"))
        })
        .filter_map(|e| fs::read_to_string(e.path()).ok())
        // System.Text.Json may have written a UTF-8 BOM.
        .filter_map(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok())
        .collect()
}

/// Same sanitizing as v1.x (`Path.GetInvalidFileNameChars` → '_'), so
/// overwriting a preset replaces its v1.x file instead of duplicating it.
pub fn file_name(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_control() || r#"<>:"/\|?*"#.contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let safe = if safe.trim().is_empty() {
        "Default".to_string()
    } else {
        safe
    };
    format!("{safe}.json")
}

fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)
}

/// Imports v1.x `config.ini` (written by `ConfigManager.WriteConfig`).
fn import_legacy_ini(ini: &str, presets: &[Preset]) -> Settings {
    let mut s = Settings::default();
    let mut bands = Vec::new();
    let mut flat = false;
    let mut gate = GATE_OFF_DB;
    let mut ns = false;
    for line in ini.lines() {
        let Some((key, value)) = line.trim().split_once('=') else {
            continue;
        };
        let on = value == "1";
        match key {
            "Enabled" => s.boost_enabled = on,
            "InputDevice" if !value.is_empty() => s.input_device_id = value.to_string(),
            "OutputDevice" => s.output_device_id = value.to_string(),
            "UnlockLimit" => s.unlock_limit = on,
            "EqBands" => {
                bands = value
                    .split(',')
                    .filter_map(|item| {
                        let mut it = item.split(':').map(|p| p.trim().parse::<f32>());
                        match (it.next(), it.next(), it.next()) {
                            (Some(Ok(f)), Some(Ok(g)), Some(Ok(q))) => {
                                Some(EqBand::peaking(f, g, q))
                            }
                            _ => None,
                        }
                    })
                    .collect();
            }
            "NoiseGate" => {
                if let Ok(v) = value.parse::<f32>() {
                    gate = if v >= 0.0 {
                        GATE_OFF_DB
                    } else {
                        v.max(GATE_OFF_DB)
                    };
                }
            }
            "FlatMode" => flat = on,
            "LastPreset" if !value.is_empty() => s.current_preset = Some(value.to_string()),
            "LastDeviceProfile" if !value.is_empty() => {
                s.current_device_profile = Some(value.to_string())
            }
            "Language" if !value.is_empty() => s.language = value.to_string(),
            "UpdateCheck" => s.check_updates = on,
            "NoiseSuppression" => ns = on,
            _ => {}
        }
    }
    let legacy = Preset {
        bands,
        flat_mode: flat,
        noise_gate_threshold: gate,
        ..Default::default()
    };
    s.sound = legacy.to_sound(&Sound {
        noise_suppression: ns,
        ..Sound::default()
    });
    s.sound.noise_suppression = ns;
    // v1.x reloaded points without their type; restore the preset's types when they match.
    if let Some(p) = s
        .current_preset
        .as_ref()
        .and_then(|n| presets.iter().find(|p| &p.name == n))
    {
        if p.bands.len() == s.sound.bands.len() {
            for (b, pb) in s.sound.bands.iter_mut().zip(&p.bands) {
                if (b.frequency - pb.frequency).abs() < 0.5 {
                    b.band_type = pb.band_type;
                }
            }
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const V1_INI: &str = "[Settings]\nValue=1.0\nEnabled=1\nInputDevice={0.0.1.00000000}.{6f0c}\n\
        OutputDevice={0.0.0.00000000}.{d885}\nUnlockLimit=0\nEqBands=20:32.28451:0.1,20000:32.28451:0.1\n\
        NoiseGate=-29.95\nLimiter=0.0\nLimiterEnabled=1\nFlatMode=1\nLastPreset=test\n\
        LastDeviceProfile=test\nLanguage=ja\nUpdateCheck=1\nNoiseSuppression=1\n";

    #[test]
    fn imports_v1_flat_config() {
        let s = import_legacy_ini(V1_INI, &[]);
        assert!(s.boost_enabled);
        assert_eq!(s.input_device_id, "{0.0.1.00000000}.{6f0c}");
        assert_eq!(s.output_device_id, "{0.0.0.00000000}.{d885}");
        assert_eq!(s.sound.mode, EqMode::Simple);
        assert!((s.sound.gain_db - 32.28451).abs() < 1e-4);
        assert!(s.sound.bands.is_empty());
        assert!((s.sound.noise_gate_db + 29.95).abs() < 1e-4);
        assert!(s.sound.noise_suppression);
        assert_eq!(s.language, "ja");
        assert_eq!(s.current_preset.as_deref(), Some("test"));
    }

    #[test]
    fn imports_v1_point_mode() {
        let ini = V1_INI.replace("FlatMode=1", "FlatMode=0").replace(
            "EqBands=20:32.28451:0.1,20000:32.28451:0.1",
            "EqBands=20:0:0.1,1000:6:0.1,20000:0:0.1",
        );
        let s = import_legacy_ini(&ini, &[]);
        assert_eq!(s.sound.mode, EqMode::Equalizer);
        assert_eq!(s.sound.gain_db, 0.0);
        assert_eq!(s.sound.bands.len(), 3);
        assert_eq!(s.sound.bands[1], EqBand::peaking(1000.0, 6.0, 0.1));
    }

    #[test]
    fn reads_v1_preset_json() {
        let json = r#"{"Name":"test","Bands":[{"Frequency":20,"Gain":12.5,"Q":0.1,"Type":1},
            {"Frequency":20000,"Gain":12.5,"Q":0.1,"Type":2}],"FlatMode":true,"MinFreq":20,
            "MaxFreq":20000,"UnlockLimit":false,"NoiseGateThreshold":-45}"#;
        let p: Preset = serde_json::from_str(json).unwrap();
        assert_eq!(p.bands[0].band_type, BandType::LowShelf);
        let s = p.to_sound(&Sound::default());
        assert_eq!(s.mode, EqMode::Simple);
        assert_eq!(s.gain_db, 12.5);
        assert_eq!(s.noise_gate_db, -45.0);
    }

    #[test]
    fn preset_round_trip() {
        let sound = Sound {
            gain_db: 18.0,
            mode: EqMode::Equalizer,
            bands: vec![EqBand {
                frequency: 90.0,
                gain: 0.0,
                q: 0.71,
                band_type: BandType::HighPass,
            }],
            noise_gate_db: -50.0,
            noise_suppression: true,
            suppression_strength: 0.8,
        };
        let p = Preset::from_sound("x", &sound);
        let back: Preset = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert!(back.to_sound(&Sound::default()).approx_eq(&sound));
    }

    #[test]
    fn sanitizes_file_names_like_v1() {
        assert_eq!(file_name("a/b:c"), "a_b_c.json");
        assert_eq!(file_name("  "), "Default.json");
    }

    #[test]
    fn store_round_trip() {
        let dir = std::env::temp_dir().join(format!("emb-test-{}", std::process::id()));
        let store = ConfigStore::at(dir.clone());
        let mut s = Settings::default();
        s.sound.gain_db = 7.0;
        store.save_settings(&s).unwrap();
        assert_eq!(store.load_settings(), s);
        store
            .save_preset(&Preset::from_sound("P", &s.sound))
            .unwrap();
        assert_eq!(store.load_presets().len(), 1);
        store.delete_preset("P");
        assert!(store.load_presets().is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}
