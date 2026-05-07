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

- **.NET 8.0 SDK** -- [dotnet.microsoft.com](https://dotnet.microsoft.com/download/dotnet/8.0)
- **IDE** -- Visual Studio 2022（推奨）または VS Code

## プロジェクト構成

```
Easy-MIC-Booster/
├── src/                       ソースコード
│   └── Lang/                  言語ファイル (json)
├── docs/                      ドキュメントサイト（この Jekyll サイト）
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/                    日本語版
│   └── *.md                   英語版
└── build/                     ビルドスクリプトと成果物
    ├── build_debug.bat        デバッグビルド
    ├── build_release.bat      リリースビルド
    ├── Directory.Build.props  MSBuild 共通設定
    ├── bin/                   ビルド出力 (x64 / x86)
    └── zip/                   配布用パッケージ
```

## ビルド方法

{: .warning }
ビルド前に、`build/bin` から起動中の Easy MIC Booster を終了してください。ファイルがロックされているとビルドに失敗します。

### コマンドライン

```powershell
dotnet build EasyMICBooster.sln -c Release
```

### ビルドスクリプト

```cmd
.\build\build_release.bat
```

実行ファイルは `build/bin/x64` または `build/bin/x86` に出力されます。

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

[MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/LICENSE)
