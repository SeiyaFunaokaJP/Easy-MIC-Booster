---
title: ホーム
layout: default
nav_order: 11
has_children: true
has_toc: false
---

# Easy MIC Booster

Windows 向けのマイクゲイン増幅と高品質な音声処理ツール。
{: .fs-6 .fw-300 }

マイクの音声を取り込み、ノイズ抑制・イコライザー・ゲイン・ノイズゲート・リミッターできれいにして増幅し、VB-CABLE などの仮想オーディオデバイスへ送ります。これにより、どのアプリでも処理済みの声を使えます。

[はじめる](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[ダウンロード](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Easy MIC Booster のメインウィンドウ](../images/app_top.png)

---

## 主な機能 {#key-features}

- **ソフトウェアアンプ** -- 小さいマイクの音量を Windows のゲイン上限を超えて増幅します（最大 +50 dB、上限解除時は +100 dB）
- **ノイズ抑制** -- 話している最中でもファンやキーボード、部屋の雑音を除去します。軽量・標準（RNNoise）・高精度（DeepFilterNet3）から選べます
- **パラメトリックイコライザー** -- リアルタイムのスペクトルグラフ上で音質を調整できます
- **ノイズゲート** -- 話の合間にマイクをミュートし、無音部分を完全に静かにします
- **リミッター** -- ピークを上限以下に抑え、突然の大きな音でも音割れしません
- **プリセットとデバイスプロファイル** -- サウンドやマイク/出力の組み合わせをワンクリックで切り替えられます
- **トレイから操作** -- 通知領域で常駐し、ブースト・プリセット・デバイスを右クリックですぐに操作できます
- **8 言語対応** -- English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español

## 仕組み {#how-it-works}

```
マイク
    │
    ▼
キャプチャ (Windows では WASAPI)
    │
    ▼
ノイズ抑制 → イコライザー → ゲイン → ノイズゲート → リミッター
    │
    ▼
仮想オーディオデバイス (例: CABLE Input)
    │
    ▼
Discord / Zoom / OBS / Teams / ゲーム …
```

## クイックスタート {#quick-start}

1. [VB-CABLE](https://vb-audio.com/Cable/) をインストールします。
2. 最新のリリース（インストーラー `...-setup.exe` またはポータブル版 zip）をダウンロードし、Easy MIC Booster を起動します。
3. **ルーティング** で、**入力** にお使いのマイク、**出力** に `CABLE Input (VB-Audio Virtual Cable)` を選択します。
4. 右上の **BOOST ON** が有効になっていることを確認し、**ゲイン** を設定します。
5. Discord、Zoom、OBS などで、マイクとして `CABLE Output (VB-Audio Virtual Cable)` を選択します。

詳しい手順は [ユーザーガイド](user-guide) を参照してください。
