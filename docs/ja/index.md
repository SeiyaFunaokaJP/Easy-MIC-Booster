---
title: 日本語
layout: default
nav_order: 5
has_children: true
---

# Easy MIC Booster

Windows 向けマイクゲイン増幅 & 高品質音声処理ツール。
{: .fs-6 .fw-300 }

NAudio でマイク入力をキャプチャし、AI ノイズ抑制（RNNoise）・イコライザー・ノイズゲート・リミッターを通して、VB-CABLE などの仮想オーディオデバイスへ出力します。

[使い方を見る](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Easy MIC Booster Top Screen](../images/app_top.jpg)

---

## 主な機能

- **ソフトウェアアンプ** -- マイクのゲイン不足をシステムレベルで解消
- **AI ノイズ抑制** -- RNNoise によるファン音・打鍵音などの背景雑音除去（発話中でも有効）
- **ノイズゲート** -- 発話していない無音区間を完全に消音
- **パラメトリックイコライザー** -- 視覚的なグラフ操作とプリセット保存
- **リミッター** -- 突発的な大音量による音割れを防止
- **多言語対応** -- 英語 / 日本語
- **設定の自動保存** -- デバイス・スイッチ・EQ などを次回起動時に復元

## 動作の仕組み

```
マイク入力
   │
   ▼
 NAudio キャプチャ (PCM)
   │
   ▼
 DSP チェーン (RNNoise → イコライザー → ゲイン → ノイズゲート → リミッター)
   │
   ▼
 仮想オーディオデバイス (VB-CABLE Input)
   │
   ▼
 Discord / Zoom / OBS など
```

## クイックスタート

1. [VB-CABLE](https://vb-audio.com/Cable/) をインストール。
2. リリース版の zip をダウンロードして解凍。
3. `EasyMICBooster.exe` を起動。
4. **入力** に物理マイク、**出力** に `CABLE Input (VB-Audio Virtual Cable)` を選択。
5. 中央のスイッチを **ON**。
6. Discord / Zoom / OBS のマイクに `CABLE Output (VB-Audio Virtual Cable)` を選択。

詳細は[ユーザーガイド](user-guide)を参照してください。
