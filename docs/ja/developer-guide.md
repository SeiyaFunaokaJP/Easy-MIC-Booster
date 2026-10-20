---
title: 開発者ガイド
layout: default
parent: 日本語
nav_order: 3
---

# 開発者ガイド
{: .no_toc }

## 目次
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## ビルド要件

- **Rust**（stable）-- [rustup](https://rustup.rs/) でインストール
- **Windows**: [Visual Studio Build Tools](https://visualstudio.microsoft.com/ja/visual-cpp-build-tools/) 2022 の「C++ によるデスクトップ開発」ワークロード（MSVC + Windows SDK）。ARM64 向けにビルドする場合は「MSVC ARM64 ビルドツール」コンポーネントも追加
- **Linux**（Debian/Ubuntu のパッケージ名）:
  ```bash
  sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev libxdo-dev \
    libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev libx11-dev \
    libxcursor-dev libxrandr-dev libxi-dev
  ```
- **macOS**: Xcode コマンドラインツール（`xcode-select --install`）
- **エディタ** -- 任意。VS Code + rust-analyzer がおすすめ
- **Inno Setup 6**（任意、Windows インストーラー用）-- `winget install JRSoftware.InnoSetup`

## 対応プラットフォーム

| プラットフォーム | Rust ターゲット | 状態 |
|:-----------------|:----------------|:-----|
| Windows x64 | `x86_64-pc-windows-msvc` | 対応 |
| Windows x86 | `i686-pc-windows-msvc` | 対応 |
| Windows ARM64 | `aarch64-pc-windows-msvc` | 対応 |
| Linux x64 / ARM64 | `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` | 試験的 |
| macOS Apple Silicon / Intel | `aarch64-apple-darwin` / `x86_64-apple-darwin` | 試験的・未署名 |

Linux では音声は ALSA 経由で出力されます。PipeWire/PulseAudio の null シンク
（`pactl load-module module-null-sink sink_name=EasyMICBooster`）を作成し、
`PULSE_SINK=EasyMICBooster` を付けてアプリを起動して出力に **pulse**（または **default**）を選び、
相手側アプリのマイクにそのシンクのモニターを選択します。macOS では VB-CABLE の代わりに
[BlackHole](https://existential.audio/blackhole/) を使用します。

## プロジェクト構成

```
Easy-MIC-Booster/
├── Cargo.toml                 クレート定義（version は version.txt と一致させる）
├── build.rs                   Windows exe へのアイコン・バージョン情報の埋め込み
├── version.txt                リリースバージョン（基準）
├── src/                       Rust ソースコード
├── lang/                      言語ファイル (json、バイナリに埋め込み)
├── assets/                    アイコン
├── installer/                 Inno Setup スクリプト（Windows インストーラー）
├── tools/                     補助スクリプト (check_lang.py)
├── docs/                      ドキュメントサイト（この Jekyll サイト）
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/                    日本語版
│   └── *.md                   英語版
├── .github/workflows/         CI (ci.yml) とリリースビルド (release.yml)
└── build/                     ビルドスクリプトと成果物
    ├── build_release.ps1      Windows リリースビルド (x64 / x86 / arm64)
    ├── build_release.bat      build_release.ps1 をダブルクリックで実行するラッパー
    ├── build_linux.sh         Linux / macOS リリースビルド (.tar.gz)
    ├── gen_notices.ps1        THIRD-PARTY-NOTICES.txt の再生成 (cargo-about)
    ├── about.toml, about.hbs  cargo-about の設定とテンプレート
    ├── bin/                   アーキテクチャ別のパッケージ内容
    └── zip/                   配布用パッケージ
```

## ノイズ抑制

ノイズ抑制には 3 つの方式があります（`src/dsp/denoise.rs` が方式を選び、元の音と混合します）。

- **軽量** — `src/dsp/spectral.rs`。`realfft` による 50 % オーバーラップ STFT 上のウィーナーフィルター。モデルなし。
- **標準** — [RNNoise](https://github.com/xiph/rnnoise)（BSD-3-Clause）の pure Rust 移植
  [nnnoiseless](https://github.com/jneem/nnnoiseless)（学習済みモデルを同梱）。
- **高精度** — [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet) の `libDF`
  クレート（`deep_filter`。git のリリースタグに固定）を pure Rust の推論エンジン
  [tract](https://github.com/sonos/tract) 上で実行し、モデルを埋め込んでいます（実行ファイルが約 25 MB 増加）。
  モデルが `Send` でないため専用スレッドで動かしています（`src/dsp/deepfilter.rs`）。
  tract は ARM64 で GNU 形式のアセンブリをビルドし MSVC では扱えないため、ARM64 版 Windows
  ではこの依存を外しています（対応する `deepfilter` cfg は `build.rs` が設定します）。

すべて実行ファイルに組み込まれるため、実行時に DLL やモデルファイルは不要です。
キャプチャスレッド上での各方式の CPU 負荷は `cargo test --release -- --ignored --nocapture bench_chain` で確認できます。

## ビルド方法

{: .warning }
ビルド前に、`target/` や `build/bin` から起動中の Easy MIC Booster を終了してください。Windows では実行ファイルがロックされているとビルドに失敗します。

### コマンドライン

```bash
cargo run                    # デバッグビルド（依存クレートは最適化されるため音声は途切れません）
cargo build --release        # 最適化ビルド（target/release/ に出力）
cargo test
python tools/check_lang.py   # lang/*.json を en.json と照合
```

### リリースパッケージ (Windows)

```powershell
rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-pc-windows-msvc
.\build\build_release.bat    # または: .\build\build_release.ps1 -Arch x64
```

スクリプトは `version.txt` を読み込み（`Cargo.toml` のバージョンと異なる場合は停止）、
各ターゲットを `cargo build --release --locked` でビルドして、`EasyMICBooster.exe` と
`LICENSE-MIT`・`LICENSE-APACHE`・`THIRD-PARTY-NOTICES.txt`・`README.md` を `build/bin/<arch>` にまとめ、
ポータブル zip（Inno Setup があればインストーラーも）を `build/zip` に出力します。
未インストールのターゲットはスキップされます。

### リリースパッケージ (Linux / macOS)

```bash
bash build/build_linux.sh                              # ホストのターゲット
bash build/build_linux.sh aarch64-unknown-linux-gnu    # ターゲットを指定
```

`build/zip/EasyMICBooster-<linux|macos>-<x64|arm64>-vX.Y.Z.tar.gz` を出力します。

### サードパーティーライセンス表記

依存関係（`Cargo.lock`）を変更したら再生成してください。

```powershell
cargo install --locked cargo-about
.\build\gen_notices.ps1
```

### リリース手順

1. `version.txt` と `Cargo.toml` の `version` を更新（`cargo check` を実行して `Cargo.lock` も更新）。
2. タグ `vX.Y.Z` を push。*Release* ワークフローが全プラットフォームをビルドし、パッケージ付きのドラフトリリースを作成します。
3. ドラフトを確認して公開します。

## ドキュメントサイト

このサイトは [Jekyll](https://jekyllrb.com/) と [just-the-docs](https://just-the-docs.com/) リモートテーマで構築されています。GitHub Pages が `main` ブランチの `docs/` ディレクトリから直接配信します。

### ローカルプレビュー

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### ページの追加

1. `docs/`（英語）と `docs/ja/`（日本語）に Markdown ファイルを作成。
2. 以下のフロントマターを追加:
   ```yaml
   ---
   title: ページタイトル
   layout: default
   nav_order: <番号>
   ---
   ```
3. 日本語版には `parent: 日本語` も追加。

ページ下部の言語スイッチャーが英語/日本語の対応 URL を自動的に切り替えます。

## コントリビューション

開発フロー（フォーク、プルリクエスト等）の詳細は [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/CONTRIBUTING.md) を参照してください。

## ライセンス

[MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/LICENSE-MIT) と [Apache License 2.0](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/LICENSE-APACHE) のデュアルライセンスです（いずれかを選択できます）。
