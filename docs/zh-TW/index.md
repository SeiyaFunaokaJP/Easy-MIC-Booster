---
title: 首頁
layout: default
nav_order: 13
has_children: true
has_toc: false
---

# Easy MIC Booster

適用於 Windows 的麥克風增益放大與高品質音訊處理工具。
{: .fs-6 .fw-300 }

擷取您的麥克風音訊，加以清理並增強——降噪、等化器、增益、噪音閘與限幅器——再將結果路由到 VB-CABLE 等虛擬音訊裝置，讓所有應用程式都能聽到處理後的聲音。

[開始使用](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[下載](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Easy MIC Booster 主視窗](../images/app_top.png)

---

## 主要功能 {#key-features}

- **軟體放大器** -- 將音量偏小的麥克風增強到超過 Windows 增益上限（最高 +50 dB，解除限制後最高 +100 dB）
- **降噪** -- 即使在說話時也能去除風扇聲、鍵盤聲與環境噪音；可選擇輕量、標準（RNNoise）或高精度（DeepFilterNet3）
- **參數等化器** -- 在即時頻譜圖上調整您的音色
- **噪音閘** -- 在語句之間將麥克風靜音，讓空檔完全無聲
- **限幅器** -- 將峰值保持在上限以下，突如其來的大聲也不會爆音
- **預設組與裝置設定檔** -- 一鍵切換聲音設定以及麥克風／輸出的組合
- **系統匣控制** -- 常駐於通知區域；增強、預設組與裝置只要按一下右鍵即可操作
- **8 種語言** -- English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español

## 運作原理 {#how-it-works}

```
麥克風
    │
    ▼
擷取（Windows 上為 WASAPI）
    │
    ▼
降噪 → 等化器 → 增益 → 噪音閘 → 限幅器
    │
    ▼
虛擬音訊裝置（例如 CABLE Input）
    │
    ▼
Discord / Zoom / OBS / Teams / 遊戲 …
```

## 快速入門 {#quick-start}

1. 安裝 [VB-CABLE](https://vb-audio.com/Cable/)。
2. 下載最新版本（安裝程式 `...-setup.exe` 或可攜式 zip），然後啟動 Easy MIC Booster。
3. 在 **路由** 中，將您的麥克風選為 **輸入**，並將 `CABLE Input (VB-Audio Virtual Cable)` 選為 **輸出**。
4. 確認右上角的 **BOOST ON** 已開啟，並設定 **增益**。
5. 在 Discord、Zoom、OBS 等軟體中，將 `CABLE Output (VB-Audio Virtual Cable)` 選為麥克風。

完整說明請參閱 [使用者指南](user-guide)。
