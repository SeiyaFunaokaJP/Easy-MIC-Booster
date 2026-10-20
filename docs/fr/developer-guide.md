---
title: Guide du développeur
layout: default
parent: Accueil
nav_order: 3
---

# Guide du développeur
{: .no_toc }

## Table des matières
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Prérequis de compilation {#build-requirements}

- **Rust** (stable) -- à installer avec [rustup](https://rustup.rs/)
- **Windows** : [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022 avec la charge de travail *Desktop development with C++* (MSVC + Windows SDK). Ajoutez le composant *MSVC ARM64 build tools* pour compiler pour ARM64.
- **Linux** (noms de paquets Debian/Ubuntu) :
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS** : Xcode Command Line Tools (`xcode-select --install`)
- **Éditeur** -- au choix ; VS Code avec rust-analyzer fonctionne bien
- **Inno Setup 6** (facultatif, programmes d'installation Windows) -- `winget install JRSoftware.InnoSetup`

## Plateformes prises en charge {#supported-platforms}

| Plateforme | Cible Rust | État |
|:-----------|:-----------|:-----|
| Windows x64 | `x86_64-pc-windows-msvc` | Pris en charge |
| Windows x86 | `i686-pc-windows-msvc` | Pris en charge |
| Windows ARM64 | `aarch64-pc-windows-msvc` | Pris en charge |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | Expérimental |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | Expérimental, non signé |

Sous Linux, l'audio passe par ALSA : créez un null sink PipeWire/PulseAudio
(`pactl load-module module-null-sink sink_name=EasyMICBooster`), lancez l'application
avec `PULSE_SINK=EasyMICBooster`, choisissez **pulse** (ou **default**) comme sortie,
puis sélectionnez le moniteur du sink comme microphone dans l'application cible. Sous macOS, utilisez
[BlackHole](https://existential.audio/blackhole/) à la place de VB-CABLE.

## Structure du projet {#project-structure}

```
Easy-MIC-Booster/
├── Cargo.toml                 Manifeste du crate (la version doit correspondre à version.txt)
├── build.rs                   Intègre l'icône et les infos de version dans l'exe Windows
├── version.txt                Version de publication (source de référence)
├── src/                       Code source Rust
├── lang/                      Fichiers de langue (json, intégrés au binaire)
├── assets/                    Icônes
├── installer/                 Script Inno Setup (programme d'installation Windows)
├── tools/                     Scripts utilitaires (check_lang.py)
├── docs/                      Site de documentation (ce site Jekyll)
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/ zh-CN/ zh-TW/      Traductions, un dossier par langue
│   ├── ko/ de/ fr/ es/
│   └── *.md                   Pages en anglais
├── .github/workflows/         CI (ci.yml) et builds de publication (release.yml)
└── build/                     Scripts de build et artefacts
    ├── build_release.ps1      Build de publication Windows (x64 / x86 / arm64)
    ├── build_release.bat      Lanceur double-clic pour build_release.ps1
    ├── build_linux.sh         Build de publication Linux / macOS (.tar.gz)
    ├── gen_notices.ps1        Régénère THIRD-PARTY-NOTICES.txt (cargo-about)
    ├── about.toml, about.hbs  Configuration et modèle de cargo-about
    ├── bin/                   Paquets préparés (par architecture)
    └── zip/                   Paquets de distribution
```

## Suppression de bruit {#noise-suppression}

La suppression de bruit propose trois méthodes (`src/dsp/denoise.rs` en choisit une et la mélange
avec le signal d'origine) :

- **Léger** — `src/dsp/spectral.rs`, un filtre de Wiener sur une STFT à recouvrement de 50 %
  utilisant `realfft`. Aucun modèle.
- **Standard** — [nnnoiseless](https://github.com/jneem/nnnoiseless), un
  portage 100 % Rust de [RNNoise](https://github.com/xiph/rnnoise) (BSD-3-Clause)
  qui inclut les poids du modèle.
- **Haute qualité** — [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet)
  via son crate `libDF` (`deep_filter`, épinglé sur un tag de version depuis git)
  sur le moteur d'exécution 100 % Rust [tract](https://github.com/sonos/tract), avec le
  modèle intégré (~25 Mo ajoutés à l'exécutable). Le modèle n'est pas `Send` ; il
  s'exécute donc sur son propre thread (`src/dsp/deepfilter.rs`). tract assemble
  des noyaux en syntaxe GNU sur ARM64, ce que MSVC ne sait pas faire ; la dépendance est donc exclue
  des builds Windows ARM64, et `build.rs` définit le cfg `deepfilter` correspondant.

Tout est compilé dans l'exécutable : aucune DLL native ni aucun fichier de modèle n'est
nécessaire à l'exécution. `cargo test --release -- --ignored --nocapture bench_chain`
affiche le coût CPU de chaque méthode sur le thread de capture.

## Compilation {#building}

{: .warning }
Fermez toute instance lancée depuis `target/` ou `build/bin` avant de compiler. Sous Windows, un exécutable verrouillé fait échouer la compilation.

### Ligne de commande {#command-line}

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### Paquets de publication (Windows) {#release-packages-windows}

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

Le script lit `version.txt` (et s'arrête si `Cargo.toml` indique une autre
version), compile chaque cible avec `cargo build --release --locked`, place
`EasyMICBooster.exe` avec `LICENSE-MIT`, `LICENSE-APACHE`, `THIRD-PARTY-NOTICES.txt` et `README.md`
dans `build/bin/<arch>`, puis écrit les zips portables (et, avec Inno Setup,
les programmes d'installation) dans `build/zip`. Les cibles non installées sont ignorées.

### Paquets de publication (Linux / macOS) {#release-packages-linux--macos}

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

Produit `build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz`.

### Mentions tierces {#third-party-notices}

Après avoir modifié les dépendances (`Cargo.lock`), régénérez les mentions :

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### Publication d'une version {#releasing}

1. Mettez à jour `version.txt` et `version` dans `Cargo.toml` (puis lancez `cargo check` pour actualiser `Cargo.lock`).
2. Poussez un tag `vX.Y.Z`. Le workflow *Release* compile toutes les plateformes et crée un brouillon de release GitHub avec les paquets.
3. Relisez le brouillon et publiez-le.

## Site de documentation {#documentation-site}

Ce site est généré avec [Jekyll](https://jekyllrb.com/) et le thème distant [just-the-docs](https://just-the-docs.com/). GitHub Pages le produit directement à partir du répertoire `docs/` de la branche `master`.

### Aperçu local {#local-preview}

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### Langues {#languages}

Les pages en anglais se trouvent dans `docs/` ; chaque autre langue dispose d'un dossier nommé d'après
son code (`ja`, `zh-CN`, `zh-TW`, `ko`, `de`, `fr`, `es`) contenant les mêmes noms
de fichiers. La liste `languages` de `_config.yml` pilote le menu des langues dans
l'en-tête, les liens de langue dans la barre latérale, les liens `hreflang` et le
filtre de la barre latérale qui n'affiche que la langue courante.

Chaque dossier traduit possède une page d'accueil (`index.md`, `has_children: true`)
dont les pages enfants sont les pages traduites. Les titres traduits conservent l'ancre
anglaise (`## インストール {#installation}`), si bien que changer de langue maintient le
lecteur sur la même section.

### Ajouter une page {#adding-a-page}

1. Créez la page anglaise dans `docs/` avec l'en-tête (front matter) suivant :
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
2. Ajoutez la traduction, avec le même nom de fichier, dans chaque dossier de langue, en
   définissant `parent:` sur le titre de la page d'accueil de cette langue (par exemple
   `parent: ホーム`).

## Contribuer {#contributing}

Consultez [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/CONTRIBUTING.md) pour le processus de développement (forks, pull requests, etc.).

## Licence {#license}

Double licence : [MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-MIT) ou
[Apache License, Version 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-APACHE), au choix.
