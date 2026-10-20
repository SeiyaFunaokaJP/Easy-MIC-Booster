---
title: Guía para desarrolladores
layout: default
parent: Inicio
nav_order: 3
---

# Guía para desarrolladores
{: .no_toc }

## Índice
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Requisitos de compilación {#build-requirements}

- **Rust** (stable) -- instálelo con [rustup](https://rustup.rs/)
- **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022 con la carga de trabajo *Desktop development with C++* (MSVC + Windows SDK). Añada el componente *MSVC ARM64 build tools* para compilar para ARM64.
- **Linux** (nombres de paquetes de Debian/Ubuntu):
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Editor** -- cualquiera; VS Code con rust-analyzer funciona bien
- **Inno Setup 6** (opcional, instaladores de Windows) -- `winget install JRSoftware.InnoSetup`

## Plataformas compatibles {#supported-platforms}

| Plataforma | Target de Rust | Estado |
|:---------|:------------|:-------|
| Windows x64 | `x86_64-pc-windows-msvc` | Compatible |
| Windows x86 | `i686-pc-windows-msvc` | Compatible |
| Windows ARM64 | `aarch64-pc-windows-msvc` | Compatible |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | Experimental |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | Experimental, sin firmar |

En Linux, el audio pasa por ALSA: cree un null sink de PipeWire/PulseAudio
(`pactl load-module module-null-sink sink_name=EasyMICBooster`), inicie la aplicación
con `PULSE_SINK=EasyMICBooster`, elija **pulse** (o **default**) como salida
y seleccione el monitor del sink como micrófono en la aplicación de destino. En macOS, use
[BlackHole](https://existential.audio/blackhole/) en lugar de VB-CABLE.

## Estructura del proyecto {#project-structure}

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
│   ├── ja/ zh-CN/ zh-TW/      Translations, one folder per language
│   ├── ko/ de/ fr/ es/
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

## Supresión de ruido {#noise-suppression}

La supresión de ruido tiene tres métodos (`src/dsp/denoise.rs` elige uno y lo mezcla
con la señal original):

- **Ligero** — `src/dsp/spectral.rs`, un filtro de Wiener sobre una STFT con solapamiento del 50 %
  usando `realfft`. Sin modelo.
- **Estándar** — [nnnoiseless](https://github.com/jneem/nnnoiseless), un
  port en Rust puro de [RNNoise](https://github.com/xiph/rnnoise) (BSD-3-Clause)
  que incluye los pesos del modelo.
- **Alta calidad** — [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet)
  a través de su crate `libDF` (`deep_filter`, fijado a una etiqueta de versión desde git)
  sobre el runtime [tract](https://github.com/sonos/tract), escrito en Rust puro, con el
  modelo integrado (~25 MB añadidos al ejecutable). El modelo no es `Send`, así que
  se ejecuta en su propio hilo (`src/dsp/deepfilter.rs`). tract ensambla
  kernels con sintaxis GNU en ARM64, algo que MSVC no puede hacer, por lo que la dependencia se excluye
  de las compilaciones de Windows para ARM64; `build.rs` establece el cfg `deepfilter` correspondiente.

Todo se compila dentro del ejecutable, por lo que no se necesita ninguna DLL nativa ni archivo de modelo
en tiempo de ejecución. `cargo test --release -- --ignored --nocapture bench_chain`
muestra el coste de CPU de cada método en el hilo de captura.

## Compilación {#building}

{: .warning }
Cierre cualquier instancia en ejecución lanzada desde `target/` o `build/bin` antes de compilar. En Windows, un ejecutable bloqueado hará fallar la compilación.

### Línea de comandos {#command-line}

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### Paquetes de versión (Windows) {#release-packages-windows}

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

El script lee `version.txt` (y se detiene si `Cargo.toml` tiene una versión
distinta), compila cada target con `cargo build --release --locked`, prepara
`EasyMICBooster.exe` junto con `LICENSE-MIT`, `LICENSE-APACHE`, `THIRD-PARTY-NOTICES.txt` y `README.md`
en `build/bin/<arch>`, y escribe los zips portables (y, con Inno Setup,
los instaladores) en `build/zip`. Los targets que no están instalados se omiten.

### Paquetes de versión (Linux / macOS) {#release-packages-linux--macos}

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

Genera `build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz`.

### Avisos de terceros {#third-party-notices}

Después de cambiar las dependencias (`Cargo.lock`), regenere los avisos:

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### Publicación de versiones {#releasing}

1. Actualice `version.txt` y `version` en `Cargo.toml` (luego ejecute `cargo check` para actualizar `Cargo.lock`).
2. Suba una etiqueta `vX.Y.Z`. El flujo de trabajo *Release* compila todas las plataformas y crea un borrador de release en GitHub con los paquetes.
3. Revise el borrador y publíquelo.

## Sitio de documentación {#documentation-site}

Este sitio se genera con [Jekyll](https://jekyllrb.com/) usando el tema remoto [just-the-docs](https://just-the-docs.com/). GitHub Pages lo renderiza directamente desde el directorio `docs/` de la rama `master`.

### Vista previa local {#local-preview}

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### Idiomas {#languages}

Las páginas en inglés están en `docs/`; cada uno de los demás idiomas tiene una carpeta con el nombre
de su código (`ja`, `zh-CN`, `zh-TW`, `ko`, `de`, `fr`, `es`) con los mismos nombres
de archivo. La lista `languages` de `_config.yml` controla el menú de idiomas del
encabezado, los enlaces de idioma de la barra lateral, los enlaces `hreflang` y el
filtro de la barra lateral que muestra solo el idioma actual.

Cada carpeta traducida tiene una página de inicio (`index.md`, `has_children: true`)
cuyas páginas hijas son las páginas traducidas. Los encabezados traducidos conservan el
ancla en inglés (`## Instalación {#installation}`), de modo que al cambiar de idioma el
lector permanece en la misma sección.

### Añadir una página {#adding-a-page}

1. Cree la página en inglés en `docs/` con el front matter:
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
2. Añada la traducción con el mismo nombre de archivo en cada carpeta de idioma, con
   `parent:` establecido en el título de la página de inicio de ese idioma (por ejemplo,
   `parent: Inicio`).

## Contribuir {#contributing}

Consulte [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/CONTRIBUTING.md) para conocer el flujo de trabajo de desarrollo (forks, pull requests, etc.).

## Licencia {#license}

Con licencia dual bajo la [Licencia MIT](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-MIT) o la
[Licencia Apache, versión 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-APACHE), a su elección.
