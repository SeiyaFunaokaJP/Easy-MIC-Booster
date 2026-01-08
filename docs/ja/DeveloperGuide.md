# 開発者ガイド

このガイドは、Easy MIC Boosterをソースコードからビルドしたい方や、プロジェクトに貢献したい開発者向けの情報です。

## ビルド要件

- **.NET 8.0 SDK**: [dotnet.microsoft.com](https://dotnet.microsoft.com/download/dotnet/8.0) からダウンロード可能です。
- **開発環境**: Visual Studio 2022 (推奨) または VS Code。

## プロジェクト構成

```
EasyMICBooster/
├── src/                # ソースコード
│   └── Lang/           # 言語ファイル (json)
├── docs/               # ドキュメント (ja/en)
└── build/              # ビルドスクリプトと成果物
    ├── build_debug.bat       # デバッグビルド用
    ├── build_release.bat     # リリースビルド用
    ├── Directory.Build.props # MSBuild共通設定
    ├── bin/                  # ビルド出力先 (x64/x86)
    └── zip/                  # 配布用パッケージ
```

## ビルド方法

> [!IMPORTANT]
> ビルドを実行する前に、`build/bin` フォルダ内で実行中の Easy MIC Booster があれば終了してください。ファイルがロックされているとビルドに失敗します。

### コマンドラインを使用する場合

1. プロジェクトのルートディレクトリでターミナルを開きます。
2. 以下のコマンドを実行します：

```powershell
dotnet build EasyMICBooster.sln -c Release
```

### ビルドスクリプトを使用する場合

同梱のバッチファイルを実行するだけでビルド可能です：

```cmd
.\build\build_release.bat
```

ビルドされた実行ファイルは `build/bin/x64` または `build/bin/x86` に出力されます。

## コントリビューション

開発フロー（フォーク、プルリクエストなど）の詳細は [CONTRIBUTING.md](../../CONTRIBUTING.md) を参照してください。

## ライセンス

MIT License。
