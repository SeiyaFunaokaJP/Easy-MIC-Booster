---
title: Developer Guide
layout: default
nav_order: 4
---

# Developer Guide
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Build Requirements

- **Rust** (stable) -- install with [rustup](https://rustup.rs/)
- **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022 with the *Desktop development with C++* workload (MSVC + Windows SDK). Add the *MSVC ARM64 build tools* component to build for ARM64.
- **Linux** (Debian/Ubuntu package names):
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Editor** -- any; VS Code with rust-analyzer works well
- **Inno Setup 6** (optional, Windows installers) -- `winget install JRSoftware.InnoSetup`

## Supported Platforms

| Platform | Rust target | Status |
|:---------|:------------|:-------|
| Windows x64 | `x86_64-pc-windows-msvc` | Supported |
| Windows x86 | `i686-pc-windows-msvc` | Supported |
| Windows ARM64 | `aarch64-pc-windows-msvc` | Supported |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | Experimental |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | Experimental, unsigned |

On Linux, route the output to a PipeWire/PulseAudio null sink
(`pactl load-module module-null-sink sink_name=EasyMICBooster`) and select its
monitor as the microphone in the target app. On macOS, use
[BlackHole](https://existential.audio/blackhole/) in place of VB-CABLE.

## Project Structure

```
Easy-MIC-Booster/
├── Cargo.toml                 Crate manifest (version must match version.txt)
├── build.rs                   Embeds the icon and version info into the Windows exe
├── version.txt                Release version (source of truth)
├── src/                       Rust source code
├── lang/                      Language files (json, embedded into the binary)
├── assets/                    Icons
├── installer/                 Inno Setup script (Windows installer)
├── tools/                     Helper scripts (check_lang.py)
├── docs/                      Documentation site (this Jekyll site)
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/                    Japanese translation
│   └── *.md                   English pages
├── .github/workflows/         CI (ci.yml) and release builds (release.yml)
└── build/                     Build scripts and artifacts
    ├── build_release.ps1      Windows release build (x64 / x86 / arm64)
    ├── build_release.bat      Double-click wrapper for build_release.ps1
    ├── build_linux.sh         Linux / macOS release build (.tar.gz)
    ├── gen_notices.ps1        Regenerates THIRD-PARTY-NOTICES.txt (cargo-about)
    ├── about.toml, about.hbs  cargo-about configuration and template
    ├── bin/                   Staged packages (per architecture)
    └── zip/                   Distribution packages
```

## Noise Suppression

AI Noise Suppression uses [nnnoiseless](https://github.com/jneem/nnnoiseless),
a pure-Rust port of [RNNoise](https://github.com/xiph/rnnoise) (BSD-3-Clause)
that includes the model weights. It is compiled into the executable, so no
native DLL or C toolchain setup is needed.

## Building

{: .warning }
Close any running instance launched from `target/` or `build/bin` before building. A locked executable will fail the build on Windows.

### Command Line

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### Release Packages (Windows)

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

The script reads `version.txt` (and stops if `Cargo.toml` has a different
version), builds each target with `cargo build --release --locked`, stages
`EasyMICBooster.exe` with `LICENSE`, `THIRD-PARTY-NOTICES.txt` and `README.md`
into `build/bin/<arch>`, and writes the portable zips (and, with Inno Setup,
the installers) to `build/zip`. Targets that are not installed are skipped.

### Release Packages (Linux / macOS)

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

Writes `build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz`.

### Third-Party Notices

After changing dependencies (`Cargo.lock`), regenerate the notices:

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### Releasing

1. Update `version.txt` and `version` in `Cargo.toml` (then run `cargo check` to refresh `Cargo.lock`).
2. Push a tag `vX.Y.Z`. The *Release* workflow builds all platforms and creates a draft GitHub release with the packages.
3. Review the draft and publish it.

## Documentation Site

This site is built with [Jekyll](https://jekyllrb.com/) using the [just-the-docs](https://just-the-docs.com/) remote theme. GitHub Pages renders it directly from the `docs/` directory on the `main` branch.

### Local Preview

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### Adding a Page

1. Create a Markdown file in `docs/` (English) and `docs/ja/` (Japanese).
2. Add front matter:
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
3. For Japanese pages, also add `parent: 日本語`.

The language switcher at the bottom of every page links between matching English / Japanese URLs automatically.

## Contributing

See [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/CONTRIBUTING.md) for the development workflow (forks, pull requests, etc.).

## License

[MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/LICENSE).
