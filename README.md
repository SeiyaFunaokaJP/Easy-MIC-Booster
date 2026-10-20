# Easy MIC Booster

System-level microphone gain and high-quality audio processing for Windows (Linux and macOS: experimental).
Captures microphone input (WASAPI on Windows, ALSA/PipeWire on Linux, CoreAudio on macOS), applies a DSP chain — Noise Suppression, Equalizer, Noise Gate, Limiter — and routes the result to a virtual audio device such as VB-CABLE.

**[Download](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest)** | **[Documentation](https://seiyafunaokajp.github.io/Easy-MIC-Booster/)**

## Features

| Module | Description |
|--------|-------------|
| Software Amplifier | Boosts microphone level beyond the OS gain limit |
| Noise Suppression | Removes fans, keystrokes, and other background noise even while you speak; choose Light (spectral), Standard (RNNoise) or High quality (DeepFilterNet3), all built in |
| Noise Gate | Suppresses ambient noise, keyboard sounds, and breath |
| Equalizer | Multi-band EQ for tailoring vocal tone |
| Limiter | Prevents clipping at high gain settings |
| Virtual device routing | Sends processed audio to VB-CABLE or any virtual input |
| Presets & device profiles | Built-in and user presets (unsaved changes are shown, never silently lost); input/output pairs saved as profiles |
| Tray quick controls | Toggle boost / noise suppression, change gain, presets, profiles and devices from the tray menu |
| Languages | English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español |

## What's new in 2.0

- Rewritten in Rust: a single ~14 MB executable, no .NET runtime.
- Much lower load: hidden in the tray it uses about 0.3% of one CPU core with noise suppression on (v1.x: ~8–11%). The whole DSP chain costs 0.16% of a core, 0.72% with noise suppression.
- New UI: real EQ response curve with live spectrum, point inspector, gate meter with a draggable threshold, clearer preset management.
- Tray menu for quick changes; `--minimized` start; Windows ARM64 build; experimental Linux and macOS builds.
- Settings, presets and device profiles from v1.x are imported automatically.

## Prerequisites

- **Windows 10/11** (x64, x86 or ARM64)
- **VB-CABLE** (<https://vb-audio.com/Cable/>) or another virtual audio device

### Linux and macOS (experimental)

Linux (x64 / ARM64) and macOS (Apple Silicon / Intel) builds are published as `.tar.gz` archives containing a single `EasyMICBooster` binary. They are less tested than the Windows version, and the macOS binary is not signed.

- **Linux**: needs GTK 3 and libayatana-appindicator3 (for the tray icon). Audio goes through ALSA, so with PipeWire/PulseAudio create a virtual sink, start the app with `PULSE_SINK` pointing at it, pick **pulse** (or **default**) as **Output**, and choose *Monitor of EasyMICBooster* as the microphone in the target app:
  ```bash
  pactl load-module module-null-sink sink_name=EasyMICBooster
  PULSE_SINK=EasyMICBooster ./EasyMICBooster
  ```
- **macOS**: install [BlackHole](https://existential.audio/blackhole/), select it as **Output**, and choose BlackHole as the microphone in the target app. Because the binary is unsigned, allow it once with `xattr -d com.apple.quarantine EasyMICBooster` (or from System Settings > Privacy & Security).

## Quick Start

1. Install VB-CABLE.
2. Download the latest release: the installer (`EasyMICBooster-x64-vX.Y.Z-setup.exe`) or the portable zip (`...-portable.zip`, extract anywhere). Use the `arm64` files on Windows on ARM and `x86` only on 32-bit Windows.
3. Run Easy MIC Booster from the Start menu (or `EasyMICBooster.exe` in the extracted folder).
4. Select **Input** (your microphone) and **Output** (VB-CABLE Input).
5. Toggle the switch to **ON**.
6. In the application you want to use the processed mic in (Discord, OBS, etc.), select **VB-CABLE Output** as the input device.

## Building

Requires stable Rust via [rustup](https://rustup.rs/). On Windows, also install the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the *Desktop development with C++* workload.

```bash
git clone https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster.git
cd Easy-MIC-Booster
cargo run --release
```

Windows release packages:

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
build\build_release.bat
```

Outputs are written to `build/zip/`: `EasyMICBooster-{x64,x86,arm64}-vX.Y.Z-portable.zip` and, when [Inno Setup 6](https://jrsoftware.org/isinfo.php) is installed (`winget install JRSoftware.InnoSetup`), `EasyMICBooster-{x64,x86,arm64}-vX.Y.Z-setup.exe` built by `installer/EasyMICBooster.iss`. Architectures whose Rust target is not installed are skipped with a warning.

Linux packages (Debian/Ubuntu dependencies shown):

```bash
sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
  libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
  libxcursor-dev libxrandr-dev libxi-dev
bash build/build_linux.sh
```

> **Note**: The version comes from `version.txt` and must match `version` in `Cargo.toml`. Pushing a `vX.Y.Z` tag builds every platform on GitHub Actions and creates a draft release.

## Documentation

Full documentation: **<https://seiyafunaokajp.github.io/Easy-MIC-Booster/>**

- [User Guide](https://seiyafunaokajp.github.io/Easy-MIC-Booster/user-guide) — installation and usage
- [Features](https://seiyafunaokajp.github.io/Easy-MIC-Booster/features) — audio processing reference
- [Developer Guide](https://seiyafunaokajp.github.io/Easy-MIC-Booster/developer-guide) — build and contribute

Japanese version: **<https://seiyafunaokajp.github.io/Easy-MIC-Booster/ja/>**

## License

This project is licensed under either of

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

at your option. Unless you explicitly state otherwise, any contribution you
submit for inclusion in this project shall be dual-licensed as above, without
any additional terms or conditions.

Released binaries are self-contained and statically link third-party Rust
crates. Their copyright notices and license terms are reproduced in
[THIRD-PARTY-NOTICES.txt](THIRD-PARTY-NOTICES.txt), which is also shipped
inside the release packages.

## References

- [egui / eframe](https://github.com/emilk/egui) — immediate-mode GUI
- [cpal](https://github.com/RustAudio/cpal) — cross-platform audio capture and playback
- [nnnoiseless](https://github.com/jneem/nnnoiseless) — Rust port of [RNNoise](https://github.com/xiph/rnnoise), used for Standard noise suppression
- [DeepFilterNet](https://github.com/Rikorose/DeepFilterNet) on [tract](https://github.com/sonos/tract) — used for High quality noise suppression
- [tray-icon](https://github.com/tauri-apps/tray-icon) — system tray icon
- [VB-CABLE](https://vb-audio.com/Cable/) — Virtual audio device for inter-application audio routing
