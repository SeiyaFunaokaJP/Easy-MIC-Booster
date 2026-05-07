---
title: ユーザーガイド
layout: default
parent: 日本語
nav_order: 1
---

# ユーザーガイド
{: .no_toc }

## 目次
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 動作環境

- **OS**: Windows 10 / 11
- **仮想オーディオドライバ**: VB-Audio Virtual Cable（または同等品）

## インストール

### 1. VB-CABLE をインストール

1. [vb-audio.com/Cable](https://vb-audio.com/Cable/) からダウンロード。
2. zip を解凍し、`VBCABLE_Setup_x64.exe` を**管理者として実行**。
3. インストール後、PC を再起動。

### 2. Easy MIC Booster をインストール

1. [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases) ページから最新の zip をダウンロード。
2. 任意のフォルダ（例: `C:\Tools\EasyMICBooster`）に解凍。

## 初回セットアップ

### アプリの起動

解凍したフォルダ内の `EasyMICBooster.exe` を実行します。

### デバイス設定

| 項目 | 設定値 |
|:-----|:-------|
| **入力デバイス** | 物理マイク |
| **出力デバイス** | `CABLE Input (VB-Audio Virtual Cable)` |

### ルーティングの確認

1. 画面中央の大きなスイッチを **ON** にします。
2. マイクに向かって話し、レベルメーターが反応することを確認します。

### 受信側アプリの設定

Discord、Zoom、OBS などの音声設定を開き、マイク（入力デバイス）に `CABLE Output (VB-Audio Virtual Cable)` を選択します。

{: .warning }
Windows のサウンド設定で `CABLE Output` を**既定の再生デバイス**に設定**しないでください**。ハウリング（フィードバックループ）が発生します。

## 日常的な操作

- **ミュート** -- マイクアイコンをクリック、またはスイッチで切替。
- **Windows 起動時に自動実行** -- 画面下部の「スタートアップ」をオン。
- **設定の保存** -- デバイス選択、スイッチ状態、EQ などはアプリ終了時に `config.json` へ自動保存され、次回起動時に復元されます。

詳細な音声処理の操作については[機能リファレンス](features)を参照してください。
