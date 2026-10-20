---
title: 開発者ガイド
layout: default
parent: ホーム
nav_order: 3
---

# 開発者ガイド
{: .no_toc }

## 目次
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## ビルド要件 {#build-requirements}

- **Rust**（stable） -- [rustup](https://rustup.rs/) でインストールします
- **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 2022 と *C++ によるデスクトップ開発* ワークロード（MSVC + Windows SDK）。ARM64 向けにビルドするには *MSVC ARM64 ビルドツール* コンポーネントを追加します。
- **Linux**（Debian/Ubuntu のパッケージ名）:
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**: Xcode Command Line Tools（`xcode-select --install`）
- **エディター** -- 何でも構いません。VS Code と rust-analyzer の組み合わせがおすすめです
- **Inno Setup 6**（任意、Windows インストーラー用） -- `winget install JRSoftware.InnoSetup`

## 対応プラットフォーム {#supported-platforms}

| プラットフォーム | Rust ターゲット | 状態 |
|:---------|:------------|:-------|
| Windows x64 | `x86_64-pc-windows-msvc` | 対応 |
| Windows x86 | `i686-pc-windows-msvc` | 対応 |
| Windows ARM64 | `aarch64-pc-windows-msvc` | 対応 |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | 試験的 |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | 試験的、署名なし |

Linux では音声は ALSA を経由します。PipeWire/PulseAudio の null sink を作成し
（`pactl load-module module-null-sink sink_name=EasyMICBooster`）、
`PULSE_SINK=EasyMICBooster` を指定してアプリを起動します。出力に **pulse**（または **default**）を選び、
受信側のアプリでは sink のモニターをマイクとして選択してください。macOS では
VB-CABLE の代わりに [BlackHole](https://existential.audio/blackhole/) を使用します。

## プロジェクト構成 {#project-structure}

```
Easy-MIC-Booster/
├── Cargo.toml                 クレートのマニフェスト（version は version.txt と一致させる）
├── build.rs                   Windows の exe にアイコンとバージョン情報を埋め込む
├── version.txt                リリースバージョン（正となる値）
├── src/                       Rust ソースコード
├── lang/                      言語ファイル（json、バイナリに埋め込まれる）
├── assets/                    アイコン
├── installer/                 Inno Setup スクリプト（Windows インストーラー）
├── tools/                     補助スクリプト（check_lang.py）
├── docs/                      ドキュメントサイト（この Jekyll サイト）
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/ zh-CN/ zh-TW/      翻訳（言語ごとに 1 フォルダー）
│   ├── ko/ de/ fr/ es/
│   └── *.md                   英語のページ
├── .github/workflows/         CI（ci.yml）とリリースビルド（release.yml）
└── build/                     ビルドスクリプトと成果物
    ├── build_release.ps1      Windows リリースビルド（x64 / x86 / arm64）
    ├── build_release.bat      build_release.ps1 をダブルクリックで実行するラッパー
    ├── build_linux.sh         Linux / macOS リリースビルド（.tar.gz）
    ├── gen_notices.ps1        THIRD-PARTY-NOTICES.txt を再生成（cargo-about）
    ├── about.toml, about.hbs  cargo-about の設定とテンプレート
    ├── bin/                   ステージングされたパッケージ（アーキテクチャごと）
    └── zip/                   配布パッケージ
```

## ノイズ抑制 {#noise-suppression}

ノイズ抑制には 3 つの方式があります（`src/dsp/denoise.rs` がいずれかを選び、
元の信号とミックスします）。

- **軽量** — `src/dsp/spectral.rs`。`realfft` を使った 50 % オーバーラップ STFT 上の
  Wiener フィルターです。モデルは使いません。
- **標準** — [nnnoiseless](https://github.com/jneem/nnnoiseless)。
  [RNNoise](https://github.com/xiph/rnnoise) の純 Rust 移植版（BSD-3-Clause）で、
  モデルの重みを含んでいます。
- **高精度** — [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet)
  を、その `libDF` クレート（`deep_filter`、git のリリースタグに固定）経由で、
  純 Rust 製の [tract](https://github.com/sonos/tract) ランタイム上で動かします。
  モデルは埋め込まれています（実行ファイルが約 25 MB 増加）。モデルは `Send` ではないため、
  専用のスレッドで動作します（`src/dsp/deepfilter.rs`）。tract は ARM64 で GNU 構文の
  カーネルをアセンブルしますが、MSVC はこれを扱えないため、ARM64 版 Windows ビルドでは
  この依存関係を除外しています。`build.rs` が対応する `deepfilter` cfg を設定します。

すべてが実行ファイルにコンパイルされるため、実行時にネイティブ DLL やモデルファイルは
不要です。`cargo test --release -- --ignored --nocapture bench_chain` を実行すると、
キャプチャスレッド上での各方式の CPU コストが表示されます。

## ビルド {#building}

{: .warning }
ビルドの前に、`target/` や `build/bin` から起動したインスタンスをすべて終了してください。Windows では実行ファイルがロックされているとビルドに失敗します。

### コマンドライン {#command-line}

```bash
cargo run                    # debug build (dependencies are optimized, so audio stays smooth)
cargo build --release        # optimized build in target/release/
cargo test
python tools/check_lang.py   # checks lang/*.json against en.json
```

### リリースパッケージ（Windows） {#release-packages-windows}

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # or: .\build\build_release.ps1 -Arch x64
```

このスクリプトは `version.txt` を読み込み（`Cargo.toml` のバージョンが異なる場合は停止します）、
各ターゲットを `cargo build --release --locked` でビルドし、
`EasyMICBooster.exe` を `LICENSE-MIT`、`LICENSE-APACHE`、`THIRD-PARTY-NOTICES.txt`、`README.md` とともに
`build/bin/<arch>` に配置して、ポータブル版の zip（Inno Setup がある場合はインストーラーも）を
`build/zip` に出力します。インストールされていないターゲットはスキップされます。

### リリースパッケージ（Linux / macOS） {#release-packages-linux--macos}

```bash
bash build/build_linux.sh                              # host target
bash build/build_linux.sh aarch64-unknown-linux-gnu    # explicit target
```

`build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz` を出力します。

### サードパーティ通知 {#third-party-notices}

依存関係（`Cargo.lock`）を変更したら、通知ファイルを再生成します。

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### リリース手順 {#releasing}

1. `version.txt` と `Cargo.toml` の `version` を更新します（その後 `cargo check` を実行して `Cargo.lock` を更新します）。
2. タグ `vX.Y.Z` をプッシュします。*Release* ワークフローが全プラットフォームをビルドし、パッケージ付きの GitHub リリースを下書きとして作成します。
3. 下書きを確認して公開します。

## ドキュメントサイト {#documentation-site}

このサイトは [Jekyll](https://jekyllrb.com/) と [just-the-docs](https://just-the-docs.com/) リモートテーマで構築されています。GitHub Pages が `master` ブランチの `docs/` ディレクトリから直接レンダリングします。

### ローカルプレビュー {#local-preview}

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### 言語 {#languages}

英語のページは `docs/` にあり、その他の言語はそれぞれの言語コード
（`ja`、`zh-CN`、`zh-TW`、`ko`、`de`、`fr`、`es`）のフォルダーに同じファイル名で置かれています。
`_config.yml` の `languages` リストが、ヘッダーの言語メニュー、サイドバーの言語リンク、
`hreflang` リンク、そして現在の言語のページだけを表示するサイドバーのフィルターを制御します。

各翻訳フォルダーにはホームページ（`index.md`、`has_children: true`）があり、
翻訳されたページはその子ページになります。翻訳された見出しには英語のアンカーを残す
（`## インストール {#installation}`）ため、言語を切り替えても同じセクションに留まれます。

### ページの追加 {#adding-a-page}

1. `docs/` に英語のページを作成し、次のフロントマターを付けます。
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
2. 各言語フォルダーに同じファイル名で翻訳を追加し、`parent:` にその言語の
   ホームページのタイトルを設定します（例: `parent: ホーム`）。

## コントリビュート {#contributing}

開発の流れ（フォーク、プルリクエストなど）については [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/CONTRIBUTING.md) を参照してください。

## ライセンス {#license}

[MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-MIT) または
[Apache License, Version 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/master/LICENSE-APACHE) のいずれかを選択できるデュアルライセンスです。
