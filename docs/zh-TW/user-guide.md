---
title: 使用者指南
layout: default
parent: 首頁
nav_order: 1
---

# 使用者指南
{: .no_toc }

## 目錄
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 系統需求 {#system-requirements}

- **作業系統**：Windows 10 / 11（x64、x86 或 ARM64）
- **虛擬音訊驅動程式**：[VB-CABLE](https://vb-audio.com/Cable/) 或同等的虛擬音訊線

另有發布 Linux 與 macOS 版本，但目前仍屬實驗性質；在這些平台上如何路由音訊，請參閱 [開發者指南](developer-guide#supported-platforms)。

## 安裝 {#installation}

### 1. 安裝 VB-CABLE {#1-install-vb-cable}

1. 從 [vb-audio.com/Cable](https://vb-audio.com/Cable/) 下載。
2. 解壓縮後，**以系統管理員身分** 執行 `VBCABLE_Setup_x64.exe`。
3. 重新啟動電腦。

### 2. 安裝 Easy MIC Booster {#2-install-easy-mic-booster}

從 [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest) 頁面下載最新版本，可選擇以下任一種：

- **安裝程式**（`EasyMICBooster-x64-vX.Y.Z-setup.exe`）：不需系統管理員權限，為目前使用者安裝，並新增「開始」功能表項目。可從 Windows 設定 > 應用程式解除安裝；您的設定會保留。
- **可攜式**（`EasyMICBooster-x64-vX.Y.Z-portable.zip`）：解壓縮到任意資料夾，然後執行 `EasyMICBooster.exe`。

在 Windows on ARM 上請使用 `arm64` 檔案，`x86` 檔案僅用於 32 位元 Windows。ARM64 版本不包含 *高精度* 降噪方式。

## 首次設定 {#first-time-setup}

### 選擇裝置 {#choose-the-devices}

在左側的 **路由** 卡片中：

| 欄位 | 選擇 |
|:------|:-------|
| **輸入（麥克風）** | 您的實體麥克風，或選擇 *預設裝置* 以跟隨 Windows 設定 |
| **輸出（虛擬音訊線）** | `CABLE Input (VB-Audio Virtual Cable)` |

每個清單下方的音量表會顯示處理前與處理後的音量。兩者之間的那一行會摘要顯示目前啟用的功能（例如 `+12.0 dB · NS · Gate`）。

{: .warning }
如果輸出是您的喇叭或耳機（預設播放裝置），您會聽到自己的聲音。請務必將虛擬音訊線選為輸出。

### 設定增益 {#set-the-gain}

1. 確認右上角的 **BOOST ON** 已開啟。
2. 以平常的音量說話，並在 **聲音** 卡片中調高 **增益**，直到輸出音量表的峰值落在綠色到黃色的範圍。−1 / +1 按鈕每次調整 1 dB。
3. 如果音量表變紅並出現 *限幅器*，表示限幅器正在壓制峰值；請稍微降低增益。

音訊正常傳輸時，標題旁的狀態標籤會顯示 **運作中**，以及取樣率與延遲。

### 設定接收端應用程式 {#configure-the-receiving-app}

在 Discord、Zoom、OBS、Teams 等軟體中，開啟音訊設定，並將 `CABLE Output (VB-Audio Virtual Cable)` 選為麥克風（輸入裝置）。

{: .warning }
**請勿** 在 Windows 音效設定中將 `CABLE Output` 設為 **預設播放裝置**。這會造成回授迴路（嘯叫）。

## 日常使用 {#daily-use}

- **旁路** -- 關閉 **BOOST ON**，麥克風音訊便會直接通過，不套用增益與 EQ（降噪與噪音閘維持目前設定）。
- **系統匣** -- 關閉視窗後，應用程式會繼續在通知區域中執行。在系統匣圖示上按右鍵，即可切換增強或降噪、變更增益、切換預設組、裝置設定檔與裝置，或結束程式。按一下圖示即可開啟視窗。
- **預設組** -- 使用 **另存為新預設組** 儲存您的聲音，並在 **預設組** 卡片或系統匣中切換預設組。請參閱 [功能說明](features#presets)。
- **裝置設定檔** -- 儲存麥克風／輸出的組合並在其間切換（例如桌前使用耳機麥克風、直播時使用 USB 麥克風）。請參閱 [功能說明](features#device-profiles)。
- **裝置拔除** -- 如果麥克風或輸出裝置消失，狀態會變成紅色；重新連線後會自動恢復路由。

## 設定 {#settings}

按一下右上角的齒輪圖示即可開啟 **設定**。

| 設定 | 說明 |
|:--------|:------------|
| **語言** | 介面語言（首次執行時跟隨 Windows） |
| **登入時啟動** | 登入 Windows 時自動啟動（在系統匣中啟動） |
| **關閉視窗後繼續在系統匣中執行** | 關閉時：關閉視窗即結束應用程式 |
| **允許增益超過 +50 dB** | 將增益上限提高到 +100 dB |
| **輸出上限** | 限幅器將輸出保持在此峰值音量以下（預設 −1.0 dBFS） |
| **檢查更新** | 啟動時在 GitHub 上檢查是否有新版本 |

下方的按鈕可開啟設定資料夾、GitHub 頁面與贊助頁面，或結束應用程式。

設定、預設組與裝置設定檔會自動儲存到 `%APPDATA%\EasyMICBooster`，並在下次啟動時還原。首次啟動時會匯入 v1.x 的設定。

每個音訊控制項的詳細說明，請參閱 [功能說明](features)。
