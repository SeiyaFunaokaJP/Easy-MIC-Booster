---
title: 개발자 가이드
layout: default
parent: 홈
nav_order: 3
---

# 개발자 가이드
{: .no_toc }

## 목차
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 빌드 요구 사항 {#build-requirements}

- **Rust** (stable) -- [rustup](https://rustup.rs/)으로 설치
- **Windows**: *Desktop development with C++* 워크로드(MSVC + Windows SDK)가 포함된 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022. ARM64용으로 빌드하려면 *MSVC ARM64 build tools* 구성 요소를 추가하세요.
- **Linux** (Debian/Ubuntu 패키지 이름):
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **에디터** -- 무엇이든 가능하며, rust-analyzer를 사용하는 VS Code가 잘 맞습니다
- **Inno Setup 6** (선택 사항, Windows 설치 프로그램용) -- `winget install JRSoftware.InnoSetup`

## 지원 플랫폼 {#supported-platforms}

| 플랫폼 | Rust 타깃 | 상태 |
|:---------|:------------|:-------|
| Windows x64 | `x86_64-pc-windows-msvc` | 지원 |
| Windows x86 | `i686-pc-windows-msvc` | 지원 |
| Windows ARM64 | `aarch64-pc-windows-msvc` | 지원 |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | 실험적 |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | 실험적, 서명 없음 |

Linux에서는 오디오가 ALSA를 거칩니다. PipeWire/PulseAudio null sink를 만들고
(`pactl load-module module-null-sink sink_name=EasyMICBooster`), `PULSE_SINK=EasyMICBooster`로
앱을 시작한 뒤, 출력으로 **pulse**(또는 **default**)를 선택하고, 대상 앱에서 해당 sink의
monitor를 마이크로 선택하세요. macOS에서는 VB-CABLE 대신
[BlackHole](https://existential.audio/blackhole/)을 사용하세요.

## 프로젝트 구조 {#project-structure}

```
Easy-MIC-Booster/
├── Cargo.toml                 크레이트 매니페스트 (버전은 version.txt와 일치해야 함)
├── build.rs                   Windows exe에 아이콘과 버전 정보를 포함
├── version.txt                릴리스 버전 (기준 정보)
├── src/                       Rust 소스 코드
├── lang/                      언어 파일 (json, 바이너리에 포함됨)
├── assets/                    아이콘
├── installer/                 Inno Setup 스크립트 (Windows 설치 프로그램)
├── tools/                     보조 스크립트 (check_lang.py)
├── docs/                      문서 사이트 (이 Jekyll 사이트)
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/ zh-CN/ zh-TW/      번역, 언어별 폴더
│   ├── ko/ de/ fr/ es/
│   └── *.md                   영어 페이지
├── .github/workflows/         CI (ci.yml) 및 릴리스 빌드 (release.yml)
└── build/                     빌드 스크립트 및 산출물
    ├── build_release.ps1      Windows 릴리스 빌드 (x64 / x86 / arm64)
    ├── build_release.bat      build_release.ps1을 더블클릭으로 실행하는 래퍼
    ├── build_linux.sh         Linux / macOS 릴리스 빌드 (.tar.gz)
    ├── gen_notices.ps1        THIRD-PARTY-NOTICES.txt 재생성 (cargo-about)
    ├── about.toml, about.hbs  cargo-about 설정 및 템플릿
    ├── bin/                   스테이징된 패키지 (아키텍처별)
    └── zip/                   배포 패키지
```

## 노이즈 억제 {#noise-suppression}

노이즈 억제에는 세 가지 방식이 있습니다(`src/dsp/denoise.rs`가 하나를 선택해
원래(dry) 신호와 혼합합니다).

- **경량** — `src/dsp/spectral.rs`. `realfft`를 사용한 50 % 오버랩 STFT 위의
  Wiener 필터입니다. 모델은 없습니다.
- **표준** — [nnnoiseless](https://github.com/jneem/nnnoiseless).
  [RNNoise](https://github.com/xiph/rnnoise)(BSD-3-Clause)의 순수 Rust 포트로,
  모델 가중치가 포함되어 있습니다.
- **고품질** — [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet).
  `libDF` 크레이트(`deep_filter`, git의 릴리스 태그로 고정)를 통해 순수 Rust
  [tract](https://github.com/sonos/tract) 런타임 위에서 실행되며, 모델이 내장되어
  있습니다(실행 파일 크기 약 25 MB 증가). 모델이 `Send`가 아니므로 전용 스레드에서
  실행됩니다(`src/dsp/deepfilter.rs`). tract는 ARM64에서 GNU 문법 커널을 어셈블하는데
  MSVC는 이를 처리할 수 없으므로, ARM64 Windows 빌드에서는 이 의존성이 제외됩니다.
  `build.rs`가 이에 맞는 `deepfilter` cfg를 설정합니다.

모든 것이 실행 파일에 컴파일되어 있으므로 실행 시 네이티브 DLL이나 모델 파일이
필요하지 않습니다. `cargo test --release -- --ignored --nocapture bench_chain`은
캡처 스레드에서 각 방식의 CPU 비용을 출력합니다.

## 빌드 {#building}

{: .warning }
빌드하기 전에 `target/` 또는 `build/bin`에서 실행한 인스턴스를 모두 종료하세요. Windows에서는 실행 파일이 잠겨 있으면 빌드가 실패합니다.

### 명령줄 {#command-line}

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### 릴리스 패키지 (Windows) {#release-packages-windows}

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

스크립트는 `version.txt`를 읽고(`Cargo.toml`의 버전이 다르면 중단합니다),
`cargo build --release --locked`로 각 타깃을 빌드한 뒤, `EasyMICBooster.exe`를
`LICENSE-MIT`, `LICENSE-APACHE`, `THIRD-PARTY-NOTICES.txt`, `README.md`와 함께
`build/bin/<arch>`에 스테이징하고, 포터블 zip(Inno Setup이 있으면 설치 프로그램도)을
`build/zip`에 생성합니다. 설치되지 않은 타깃은 건너뜁니다.

### 릴리스 패키지 (Linux / macOS) {#release-packages-linux--macos}

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

`build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz`를 생성합니다.

### 서드파티 고지 {#third-party-notices}

의존성(`Cargo.lock`)을 변경한 후에는 고지 파일을 다시 생성하세요.

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### 릴리스 {#releasing}

1. `version.txt`와 `Cargo.toml`의 `version`을 업데이트합니다(그다음 `cargo check`를 실행해 `Cargo.lock`을 갱신합니다).
2. `vX.Y.Z` 태그를 푸시합니다. *Release* 워크플로가 모든 플랫폼을 빌드하고 패키지가 포함된 GitHub 릴리스 초안을 만듭니다.
3. 초안을 검토하고 게시합니다.

## 문서 사이트 {#documentation-site}

이 사이트는 [Jekyll](https://jekyllrb.com/)과 [just-the-docs](https://just-the-docs.com/) 원격 테마로 만들어졌습니다. GitHub Pages가 `master` 브랜치의 `docs/` 디렉터리에서 직접 렌더링합니다.

### 로컬 미리보기 {#local-preview}

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### 언어 {#languages}

영어 페이지는 `docs/`에 있고, 다른 언어는 각각 언어 코드(`ja`, `zh-CN`, `zh-TW`,
`ko`, `de`, `fr`, `es`)로 이름 붙인 폴더에 같은 파일 이름으로 들어 있습니다.
`_config.yml`의 `languages` 목록이 헤더의 언어 메뉴, 사이드바의 언어 링크,
`hreflang` 링크, 그리고 현재 언어만 보여 주는 사이드바 필터를 제어합니다.

각 번역 폴더에는 홈 페이지(`index.md`, `has_children: true`)가 있고, 번역된
페이지들이 그 하위 페이지가 됩니다. 번역된 제목은 영어 앵커를 유지하므로
(`## 설치 {#installation}`), 언어를 바꿔도 독자는 같은 섹션에 머무릅니다.

### 페이지 추가 {#adding-a-page}

1. `docs/`에 다음 front matter로 영어 페이지를 만듭니다.
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
2. 각 언어 폴더에 같은 파일 이름으로 번역을 추가하고, `parent:`를 해당 언어의
   홈 페이지 제목으로 설정합니다(예: `parent: 홈`).

## 기여 {#contributing}

개발 워크플로(포크, 풀 리퀘스트 등)는 [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/CONTRIBUTING.md)를 참고하세요.

## 라이선스 {#license}

[MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-MIT) 또는
[Apache License, Version 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-APACHE) 중 원하는 쪽을 선택할 수 있는 이중 라이선스입니다.
