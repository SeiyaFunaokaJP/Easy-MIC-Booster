---
title: Entwicklerhandbuch
layout: default
parent: Startseite
nav_order: 3
---

# Entwicklerhandbuch
{: .no_toc }

## Inhaltsverzeichnis
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Build-Voraussetzungen {#build-requirements}

- **Rust** (stable) -- Installation mit [rustup](https://rustup.rs/)
- **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022 mit der Workload *Desktop development with C++* (MSVC + Windows SDK). Fügen Sie die Komponente *MSVC ARM64 build tools* hinzu, um für ARM64 zu bauen.
- **Linux** (Paketnamen für Debian/Ubuntu):
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Editor** -- beliebig; VS Code mit rust-analyzer funktioniert gut
- **Inno Setup 6** (optional, Windows-Installationsprogramme) -- `winget install JRSoftware.InnoSetup`

## Unterstützte Plattformen {#supported-platforms}

| Plattform | Rust-Target | Status |
|:----------|:------------|:-------|
| Windows x64 | `x86_64-pc-windows-msvc` | Unterstützt |
| Windows x86 | `i686-pc-windows-msvc` | Unterstützt |
| Windows ARM64 | `aarch64-pc-windows-msvc` | Unterstützt |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | Experimentell |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | Experimentell, unsigniert |

Unter Linux läuft Audio über ALSA: Legen Sie ein PipeWire/PulseAudio-Null-Sink an
(`pactl load-module module-null-sink sink_name=EasyMICBooster`), starten Sie die App
mit `PULSE_SINK=EasyMICBooster`, wählen Sie **pulse** (oder **default**) als Ausgang
und in der Ziel-App den Monitor des Sinks als Mikrofon. Unter macOS verwenden Sie
[BlackHole](https://existential.audio/blackhole/) anstelle von VB-CABLE.

## Projektstruktur {#project-structure}

```
Easy-MIC-Booster/
├── Cargo.toml                 Crate-Manifest (Version muss mit version.txt übereinstimmen)
├── build.rs                   Bettet Symbol und Versionsinfo in die Windows-exe ein
├── version.txt                Release-Version (maßgebliche Quelle)
├── src/                       Rust-Quellcode
├── lang/                      Sprachdateien (json, in die Binärdatei eingebettet)
├── assets/                    Symbole
├── installer/                 Inno-Setup-Skript (Windows-Installationsprogramm)
├── tools/                     Hilfsskripte (check_lang.py)
├── docs/                      Dokumentationsseite (diese Jekyll-Seite)
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/ zh-CN/ zh-TW/      Übersetzungen, ein Ordner pro Sprache
│   ├── ko/ de/ fr/ es/
│   └── *.md                   Englische Seiten
├── .github/workflows/         CI (ci.yml) und Release-Builds (release.yml)
└── build/                     Build-Skripte und Artefakte
    ├── build_release.ps1      Windows-Release-Build (x64 / x86 / arm64)
    ├── build_release.bat      Wrapper zum Doppelklicken für build_release.ps1
    ├── build_linux.sh         Linux- / macOS-Release-Build (.tar.gz)
    ├── gen_notices.ps1        Erzeugt THIRD-PARTY-NOTICES.txt neu (cargo-about)
    ├── about.toml, about.hbs  Konfiguration und Vorlage für cargo-about
    ├── bin/                   Bereitgestellte Pakete (pro Architektur)
    └── zip/                   Distributionspakete
```

## Rauschunterdrückung {#noise-suppression}

Die Rauschunterdrückung hat drei Methoden (`src/dsp/denoise.rs` wählt eine aus und mischt
sie mit dem unbearbeiteten Signal):

- **Leicht** — `src/dsp/spectral.rs`, ein Wiener-Filter auf einer STFT mit 50 % Überlappung
  unter Verwendung von `realfft`. Kein Modell.
- **Standard** — [nnnoiseless](https://github.com/jneem/nnnoiseless), eine
  reine Rust-Portierung von [RNNoise](https://github.com/xiph/rnnoise) (BSD-3-Clause),
  die die Modellgewichte enthält.
- **Hohe Qualität** — [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet)
  über dessen Crate `libDF` (`deep_filter`, auf ein Release-Tag aus git festgelegt)
  auf der reinen Rust-Laufzeit [tract](https://github.com/sonos/tract), mit
  eingebettetem Modell (ca. 25 MB zusätzlich in der ausführbaren Datei). Das Modell ist nicht `Send`,
  daher läuft es in einem eigenen Thread (`src/dsp/deepfilter.rs`). tract assembliert
  Kernel in GNU-Syntax auf ARM64, was MSVC nicht kann, daher wird die Abhängigkeit
  in ARM64-Windows-Builds weggelassen; `build.rs` setzt das passende `deepfilter`-cfg.

Alles wird in die ausführbare Datei kompiliert, sodass zur Laufzeit keine native DLL und keine
Modelldatei benötigt wird. `cargo test --release -- --ignored --nocapture bench_chain`
gibt die CPU-Kosten jeder Methode im Aufnahme-Thread aus.

## Bauen {#building}

{: .warning }
Schließen Sie vor dem Bauen alle laufenden Instanzen, die aus `target/` oder `build/bin` gestartet wurden. Eine gesperrte ausführbare Datei lässt den Build unter Windows fehlschlagen.

### Kommandozeile {#command-line}

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### Release-Pakete (Windows) {#release-packages-windows}

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

Das Skript liest `version.txt` (und bricht ab, wenn `Cargo.toml` eine andere
Version hat), baut jedes Target mit `cargo build --release --locked`, stellt
`EasyMICBooster.exe` zusammen mit `LICENSE-MIT`, `LICENSE-APACHE`, `THIRD-PARTY-NOTICES.txt` und `README.md`
in `build/bin/<arch>` bereit und schreibt die portablen Zips (und mit Inno Setup
die Installationsprogramme) nach `build/zip`. Nicht installierte Targets werden übersprungen.

### Release-Pakete (Linux / macOS) {#release-packages-linux--macos}

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

Schreibt `build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz`.

### Hinweise zu Drittanbietern {#third-party-notices}

Erzeugen Sie die Hinweise nach einer Änderung der Abhängigkeiten (`Cargo.lock`) neu:

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### Veröffentlichung {#releasing}

1. Aktualisieren Sie `version.txt` und `version` in `Cargo.toml` (führen Sie danach `cargo check` aus, um `Cargo.lock` zu aktualisieren).
2. Pushen Sie ein Tag `vX.Y.Z`. Der Workflow *Release* baut alle Plattformen und erstellt ein Entwurfs-Release auf GitHub mit den Paketen.
3. Prüfen Sie den Entwurf und veröffentlichen Sie ihn.

## Dokumentationsseite {#documentation-site}

Diese Seite wird mit [Jekyll](https://jekyllrb.com/) und dem Remote-Theme [just-the-docs](https://just-the-docs.com/) erstellt. GitHub Pages rendert sie direkt aus dem Verzeichnis `docs/` im Branch `master`.

### Lokale Vorschau {#local-preview}

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### Sprachen {#languages}

Englische Seiten liegen in `docs/`; jede andere Sprache hat einen nach ihrem
Code benannten Ordner (`ja`, `zh-CN`, `zh-TW`, `ko`, `de`, `fr`, `es`) mit denselben
Dateinamen. Die Liste `languages` in `_config.yml` steuert das Sprachmenü in der
Kopfzeile, die Sprachlinks in der Seitenleiste, die `hreflang`-Links und den
Seitenleistenfilter, der nur die aktuelle Sprache anzeigt.

Jeder übersetzte Ordner hat eine Startseite (`index.md`, `has_children: true`),
deren Unterseiten die übersetzten Seiten sind. Übersetzte Überschriften behalten den
englischen Anker (`## Installation {#installation}`), sodass man beim Sprachwechsel
im selben Abschnitt bleibt.

### Eine Seite hinzufügen {#adding-a-page}

1. Legen Sie die englische Seite in `docs/` mit Front Matter an:
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
2. Fügen Sie die Übersetzung mit demselben Dateinamen in jeden Sprachordner ein,
   wobei `parent:` auf den Titel der Startseite der jeweiligen Sprache gesetzt ist
   (zum Beispiel `parent: Startseite`).

## Mitwirken {#contributing}

Den Entwicklungsablauf (Forks, Pull Requests usw.) beschreibt [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/CONTRIBUTING.md).

## Lizenz {#license}

Doppelt lizenziert unter der [MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-MIT) oder der
[Apache License, Version 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-APACHE), nach Ihrer Wahl.
