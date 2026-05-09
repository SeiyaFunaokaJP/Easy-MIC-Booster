# Easy MIC Booster

System-level microphone gain and high-quality audio processing for Windows.
Captures microphone input via NAudio (WASAPI), applies a DSP chain — Noise Gate, Equalizer, Limiter — and routes the result to a virtual audio device such as VB-CABLE.

**[Download](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest)** | **[Documentation](https://seiyafunaokajp.github.io/Easy-MIC-Booster/)**

## Features

| Module | Description |
|--------|-------------|
| Software Amplifier | Boosts microphone level beyond the OS gain limit |
| AI Noise Suppression | RNNoise-based removal of fans, keystrokes, and other background noise even while you speak (optional, requires `rnnoise.dll` — see [native/runtimes/README.md](native/runtimes/README.md)) |
| Noise Gate | Suppresses ambient noise, keyboard sounds, and breath |
| Equalizer | Multi-band EQ for tailoring vocal tone |
| Limiter | Prevents clipping at high gain settings |
| Virtual device routing | Sends processed audio to VB-CABLE or any virtual input |

## Prerequisites

- **Windows 10/11** (x64 or x86)
- **VB-CABLE** (<https://vb-audio.com/Cable/>) or another virtual audio device

## Quick Start

1. Install VB-CABLE.
2. Download the latest release and extract the zip.
3. Run `EasyMICBooster.exe`.
4. Select **Input** (your microphone) and **Output** (VB-CABLE Input).
5. Toggle the switch to **ON**.
6. In the application you want to use the processed mic in (Discord, OBS, etc.), select **VB-CABLE Output** as the input device.

## Building

```bash
git clone https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster.git
cd Easy-MIC-Booster
build\build_release.bat
```

Outputs are written to `build/zip/EasyMICBooster-x64-vX.Y.Z.zip` and `build/zip/EasyMICBooster-x86-vX.Y.Z.zip`.

> **Note**: Requires the .NET 8 SDK. The build script reads the version from `version.txt` and produces self-contained x64 and x86 builds.

## Documentation

Full documentation: **<https://seiyafunaokajp.github.io/Easy-MIC-Booster/>**

- [User Guide](https://seiyafunaokajp.github.io/Easy-MIC-Booster/user-guide) — installation and usage
- [Features](https://seiyafunaokajp.github.io/Easy-MIC-Booster/features) — audio processing reference
- [Developer Guide](https://seiyafunaokajp.github.io/Easy-MIC-Booster/developer-guide) — build and contribute

Japanese version: **<https://seiyafunaokajp.github.io/Easy-MIC-Booster/ja/>**

## License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for the full text.

Released binaries are self-contained and bundle the .NET 8 Desktop Runtime
together with third-party libraries. Their copyright notices and license
terms are reproduced in [THIRD-PARTY-NOTICES.txt](THIRD-PARTY-NOTICES.txt),
which is also shipped inside the release ZIPs.

## References

- [NAudio](https://github.com/naudio/NAudio) — .NET audio library used for WASAPI capture and rendering
- [VB-CABLE](https://vb-audio.com/Cable/) — Virtual audio device for inter-application audio routing
- [Hardcodet.NotifyIcon.Wpf](https://github.com/hardcodet/wpf-notifyicon) — System tray icon for WPF
