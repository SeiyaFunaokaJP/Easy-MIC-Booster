---
title: 开发者指南
layout: default
parent: 首页
nav_order: 3
---

# 开发者指南
{: .no_toc }

## 目录
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 构建要求 {#build-requirements}

- **Rust**（stable）-- 使用 [rustup](https://rustup.rs/) 安装
- **Windows**：[Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022，并安装 *Desktop development with C++*（使用 C++ 的桌面开发）工作负载（MSVC + Windows SDK）。如需构建 ARM64 版本，请添加 *MSVC ARM64 build tools* 组件。
- **Linux**（Debian/Ubuntu 软件包名）：
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**：Xcode Command Line Tools（`xcode-select --install`）
- **编辑器** -- 任意；推荐 VS Code 搭配 rust-analyzer
- **Inno Setup 6**（可选，用于 Windows 安装程序）-- `winget install JRSoftware.InnoSetup`

## 支持的平台 {#supported-platforms}

| 平台 | Rust target | 状态 |
|:---------|:------------|:-------|
| Windows x64 | `x86_64-pc-windows-msvc` | 支持 |
| Windows x86 | `i686-pc-windows-msvc` | 支持 |
| Windows ARM64 | `aarch64-pc-windows-msvc` | 支持 |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | 实验性 |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | 实验性，未签名 |

在 Linux 上，音频经由 ALSA 传输：先创建一个 PipeWire/PulseAudio 空 sink（`pactl load-module module-null-sink sink_name=EasyMICBooster`），然后以 `PULSE_SINK=EasyMICBooster` 启动应用，将 **pulse**（或 **default**）选为输出，并在目标应用中将该 sink 的监视器（monitor）选为麦克风。在 macOS 上，请使用 [BlackHole](https://existential.audio/blackhole/) 代替 VB-CABLE。

## 项目结构 {#project-structure}

```
Easy-MIC-Booster/
├── Cargo.toml                 crate 清单（版本必须与 version.txt 一致）
├── build.rs                   将图标和版本信息嵌入 Windows exe
├── version.txt                发布版本（唯一的版本来源）
├── src/                       Rust 源代码
├── lang/                      语言文件（json，嵌入到二进制文件中）
├── assets/                    图标
├── installer/                 Inno Setup 脚本（Windows 安装程序）
├── tools/                     辅助脚本（check_lang.py）
├── docs/                      文档站点（本 Jekyll 站点）
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/ zh-CN/ zh-TW/      翻译，每种语言一个文件夹
│   ├── ko/ de/ fr/ es/
│   └── *.md                   英文页面
├── .github/workflows/         CI（ci.yml）和发布构建（release.yml）
└── build/                     构建脚本和产物
    ├── build_release.ps1      Windows 发布构建（x64 / x86 / arm64）
    ├── build_release.bat      build_release.ps1 的双击启动包装
    ├── build_linux.sh         Linux / macOS 发布构建（.tar.gz）
    ├── gen_notices.ps1        重新生成 THIRD-PARTY-NOTICES.txt（cargo-about）
    ├── about.toml, about.hbs  cargo-about 配置和模板
    ├── bin/                   暂存的软件包（按架构）
    └── zip/                   分发包
```

## 降噪 {#noise-suppression}

降噪有三种方式（`src/dsp/denoise.rs` 选择其中一种，并将其与原始干信号混合）：

- **轻量** — `src/dsp/spectral.rs`，使用 `realfft` 在 50 % 重叠的 STFT 上进行维纳滤波。不使用模型。
- **标准** — [nnnoiseless](https://github.com/jneem/nnnoiseless)，[RNNoise](https://github.com/xiph/rnnoise) 的纯 Rust 移植版（BSD-3-Clause），包含模型权重。
- **高精度** — 通过 [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet) 的 `libDF` crate（`deep_filter`，从 git 固定到某个发布标签），在纯 Rust 的 [tract](https://github.com/sonos/tract) 运行时上运行，模型已嵌入（可执行文件增加约 25 MB）。该模型不是 `Send`，因此在独立线程上运行（`src/dsp/deepfilter.rs`）。tract 在 ARM64 上会汇编 GNU 语法的内核，而 MSVC 无法处理，因此 ARM64 Windows 版本不包含此依赖；`build.rs` 会设置相应的 `deepfilter` cfg。

所有内容都编译进可执行文件，因此运行时不需要任何原生 DLL 或模型文件。`cargo test --release -- --ignored --nocapture bench_chain` 会输出每种方式在采集线程上的 CPU 开销。

## 构建 {#building}

{: .warning }
构建前请关闭从 `target/` 或 `build/bin` 启动的所有正在运行的实例。在 Windows 上，可执行文件被锁定会导致构建失败。

### 命令行 {#command-line}

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### 发布包（Windows） {#release-packages-windows}

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

该脚本会读取 `version.txt`（如果 `Cargo.toml` 中的版本不同则停止），使用 `cargo build --release --locked` 构建每个 target，将 `EasyMICBooster.exe` 与 `LICENSE-MIT`、`LICENSE-APACHE`、`THIRD-PARTY-NOTICES.txt` 和 `README.md` 一起暂存到 `build/bin/<arch>`，并将便携版 zip（以及在安装了 Inno Setup 时的安装程序）写入 `build/zip`。未安装的 target 会被跳过。

### 发布包（Linux / macOS） {#release-packages-linux--macos}

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

生成 `build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz`。

### 第三方声明 {#third-party-notices}

更改依赖（`Cargo.lock`）后，请重新生成声明：

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### 发布 {#releasing}

1. 更新 `version.txt` 和 `Cargo.toml` 中的 `version`（然后运行 `cargo check` 以刷新 `Cargo.lock`）。
2. 推送标签 `vX.Y.Z`。*Release* 工作流会构建所有平台，并创建包含软件包的 GitHub 发布草稿。
3. 检查草稿并发布。

## 文档站点 {#documentation-site}

本站点使用 [Jekyll](https://jekyllrb.com/) 和 [just-the-docs](https://just-the-docs.com/) 远程主题构建。GitHub Pages 直接从 `master` 分支的 `docs/` 目录渲染。

### 本地预览 {#local-preview}

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### 语言 {#languages}

英文页面位于 `docs/`；其他每种语言都有一个以其语言代码命名的文件夹（`ja`、`zh-CN`、`zh-TW`、`ko`、`de`、`fr`、`es`），其中的文件名与英文相同。`_config.yml` 中的 `languages` 列表决定了页眉中的语言菜单、侧边栏中的语言链接、`hreflang` 链接，以及只显示当前语言的侧边栏过滤器。

每个翻译文件夹都有一个首页（`index.md`，`has_children: true`），其子页面就是各个翻译页面。翻译后的标题保留英文锚点（`## 安装 {#installation}`），因此切换语言时读者仍会停留在同一节。

### 添加页面 {#adding-a-page}

1. 在 `docs/` 中创建英文页面，并添加 front matter：
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
2. 在每个语言文件夹中添加同名的翻译文件，并将 `parent:` 设为该语言首页的标题（例如 `parent: 首页`）。

## 参与贡献 {#contributing}

开发流程（fork、pull request 等）请参阅 [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/CONTRIBUTING.md)。

## 许可证 {#license}

采用 [MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-MIT) 或 [Apache License, Version 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-APACHE) 双重许可，可任选其一。
