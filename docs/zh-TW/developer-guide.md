---
title: 開發者指南
layout: default
parent: 首頁
nav_order: 3
---

# 開發者指南
{: .no_toc }

## 目錄
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 建置需求 {#build-requirements}

- **Rust**（stable）-- 使用 [rustup](https://rustup.rs/) 安裝
- **Windows**：[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022，並安裝 *Desktop development with C++* 工作負載（MSVC + Windows SDK）。若要建置 ARM64 版本，請加入 *MSVC ARM64 build tools* 元件。
- **Linux**（Debian/Ubuntu 套件名稱）：
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**：Xcode Command Line Tools（`xcode-select --install`）
- **編輯器** -- 任何編輯器皆可；VS Code 搭配 rust-analyzer 相當好用
- **Inno Setup 6**（選用，用於 Windows 安裝程式）-- `winget install JRSoftware.InnoSetup`

## 支援平台 {#supported-platforms}

| 平台 | Rust 目標 | 狀態 |
|:---------|:------------|:-------|
| Windows x64 | `x86_64-pc-windows-msvc` | 支援 |
| Windows x86 | `i686-pc-windows-msvc` | 支援 |
| Windows ARM64 | `aarch64-pc-windows-msvc` | 支援 |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | 實驗性 |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | 實驗性，未簽署 |

在 Linux 上，音訊會經由 ALSA 傳輸：建立 PipeWire/PulseAudio null sink（`pactl load-module module-null-sink sink_name=EasyMICBooster`），以 `PULSE_SINK=EasyMICBooster` 啟動應用程式，將 **pulse**（或 **default**）選為輸出，並在目標應用程式中將該 sink 的 monitor 選為麥克風。在 macOS 上，請使用 [BlackHole](https://existential.audio/blackhole/) 取代 VB-CABLE。

## 專案結構 {#project-structure}

```
Easy-MIC-Booster/
├── Cargo.toml                 Crate 清單（版本必須與 version.txt 一致）
├── build.rs                   將圖示與版本資訊嵌入 Windows exe
├── version.txt                發布版本（唯一的依據來源）
├── src/                       Rust 原始碼
├── lang/                      語言檔（json，嵌入至執行檔中）
├── assets/                    圖示
├── installer/                 Inno Setup 指令碼（Windows 安裝程式）
├── tools/                     輔助指令碼（check_lang.py）
├── docs/                      文件網站（本 Jekyll 網站）
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/ zh-CN/ zh-TW/      翻譯，每種語言一個資料夾
│   ├── ko/ de/ fr/ es/
│   └── *.md                   英文頁面
├── .github/workflows/         CI（ci.yml）與發布建置（release.yml）
└── build/                     建置指令碼與產出物
    ├── build_release.ps1      Windows 發布建置（x64 / x86 / arm64）
    ├── build_release.bat      build_release.ps1 的按兩下執行包裝
    ├── build_linux.sh         Linux / macOS 發布建置（.tar.gz）
    ├── gen_notices.ps1        重新產生 THIRD-PARTY-NOTICES.txt（cargo-about）
    ├── about.toml, about.hbs  cargo-about 設定與範本
    ├── bin/                   暫存的套件（依架構分類）
    └── zip/                   發行套件
```

## 降噪 {#noise-suppression}

降噪有三種方式（由 `src/dsp/denoise.rs` 選擇其中一種，並與原始訊號混合）：

- **輕量** — `src/dsp/spectral.rs`，使用 `realfft` 在 50 % 重疊的 STFT 上套用維納濾波器。不使用模型。
- **標準** — [nnnoiseless](https://github.com/jneem/nnnoiseless)，[RNNoise](https://github.com/xiph/rnnoise) 的純 Rust 移植版（BSD-3-Clause），內含模型權重。
- **高精度** — [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet)，透過其 `libDF` crate（`deep_filter`，從 git 固定在某個發布標籤）在純 Rust 的 [tract](https://github.com/sonos/tract) 執行環境上運作，並嵌入模型（執行檔約增加 25 MB）。由於模型不是 `Send`，因此會在獨立的執行緒上執行（`src/dsp/deepfilter.rs`）。tract 在 ARM64 上會組譯 GNU 語法的核心程式，而 MSVC 無法處理，因此 ARM64 Windows 版本不包含此相依套件；`build.rs` 會設定對應的 `deepfilter` cfg。

所有內容都編譯進執行檔中，因此執行時不需要任何原生 DLL 或模型檔。`cargo test --release -- --ignored --nocapture bench_chain` 會輸出各方式在擷取執行緒上的 CPU 成本。

## 建置 {#building}

{: .warning }
建置前，請先關閉從 `target/` 或 `build/bin` 啟動的所有執行中執行個體。在 Windows 上，執行檔被鎖定會導致建置失敗。

### 命令列 {#command-line}

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### 發行套件（Windows） {#release-packages-windows}

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

此指令碼會讀取 `version.txt`（若 `Cargo.toml` 的版本不同則會停止），以 `cargo build --release --locked` 建置各目標，將 `EasyMICBooster.exe` 連同 `LICENSE-MIT`、`LICENSE-APACHE`、`THIRD-PARTY-NOTICES.txt` 與 `README.md` 暫存到 `build/bin/<arch>`，並將可攜式 zip（若有 Inno Setup，也包含安裝程式）寫入 `build/zip`。未安裝的目標會被略過。

### 發行套件（Linux / macOS） {#release-packages-linux--macos}

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

會寫入 `build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz`。

### 第三方聲明 {#third-party-notices}

變更相依套件（`Cargo.lock`）後，請重新產生聲明：

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### 發布 {#releasing}

1. 更新 `version.txt` 與 `Cargo.toml` 中的 `version`（然後執行 `cargo check` 以更新 `Cargo.lock`）。
2. 推送標籤 `vX.Y.Z`。*Release* 工作流程會建置所有平台，並建立包含各套件的 GitHub 發布草稿。
3. 檢查草稿後將其發布。

## 文件網站 {#documentation-site}

本網站使用 [Jekyll](https://jekyllrb.com/) 搭配 [just-the-docs](https://just-the-docs.com/) 遠端主題建置。GitHub Pages 會直接從 `master` 分支的 `docs/` 目錄進行轉譯。

### 本機預覽 {#local-preview}

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### 語言 {#languages}

英文頁面位於 `docs/`；其他每種語言都有一個以語言代碼命名的資料夾（`ja`、`zh-CN`、`zh-TW`、`ko`、`de`、`fr`、`es`），檔案名稱與英文相同。`_config.yml` 中的 `languages` 清單決定了標題列的語言選單、側邊欄的語言連結、`hreflang` 連結，以及只顯示目前語言的側邊欄篩選。

每個翻譯資料夾都有一個首頁（`index.md`，`has_children: true`），其子頁面即為翻譯後的頁面。翻譯後的標題會保留英文錨點（`## インストール {#installation}`），因此切換語言時讀者會停留在同一個章節。

### 新增頁面 {#adding-a-page}

1. 在 `docs/` 中建立英文頁面，並加上 front matter：
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
2. 在每個語言資料夾中加入相同檔名的翻譯，並將 `parent:` 設為該語言首頁的標題（例如 `parent: ホーム`）。

## 參與貢獻 {#contributing}

開發流程（fork、pull request 等）請參閱 [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/CONTRIBUTING.md)。

## 授權 {#license}

採用 [MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-MIT) 或 [Apache License, Version 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-APACHE) 雙重授權，您可任選其一。
