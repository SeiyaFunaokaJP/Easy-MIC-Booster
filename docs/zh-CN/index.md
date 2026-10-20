---
title: 首页
layout: default
nav_order: 12
has_children: true
has_toc: false
---

# Easy MIC Booster

适用于 Windows 的麦克风增益放大与高品质音频处理工具。
{: .fs-6 .fw-300 }

采集你的麦克风声音，进行净化和放大——降噪、均衡器、增益、噪声门和限幅器——并将处理结果路由到 VB-CABLE 等虚拟音频设备，让所有应用都能听到处理后的声音。

[开始使用](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[下载](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Easy MIC Booster 主窗口](../images/app_top.png)

---

## 主要功能 {#key-features}

- **软件放大器** -- 将音量过小的麦克风放大到超出 Windows 增益上限的水平（最高 +50 dB，解除限制后最高 +100 dB）
- **降噪** -- 即使在说话时也能去除风扇声、键盘声和环境噪音；可选择轻量、标准（RNNoise）或高精度（DeepFilterNet3）
- **参数均衡器** -- 在实时频谱图上调整音色
- **噪声门** -- 在语句之间将麦克风静音，让停顿处完全无声
- **限幅器** -- 将峰值保持在上限以下，突然的大声也不会削波
- **预设与设备配置** -- 一键切换音色以及麦克风/输出设备组合
- **托盘控制** -- 在通知区域中持续运行；右键单击即可操作增强、预设和设备
- **8 种语言** -- English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español

## 工作原理 {#how-it-works}

```
麦克风
    │
    ▼
采集（Windows 上使用 WASAPI）
    │
    ▼
降噪 → 均衡器 → 增益 → 噪声门 → 限幅器
    │
    ▼
虚拟音频设备（例如 CABLE Input）
    │
    ▼
Discord / Zoom / OBS / Teams / 游戏 …
```

## 快速开始 {#quick-start}

1. 安装 [VB-CABLE](https://vb-audio.com/Cable/)。
2. 下载最新版本（安装程序 `...-setup.exe` 或便携版 zip），然后启动 Easy MIC Booster。
3. 在 **路由** 中，将你的麦克风选为 **输入（麦克风）**，将 `CABLE Input (VB-Audio Virtual Cable)` 选为 **输出（虚拟音频线）**。
4. 确认右上角的 **BOOST ON** 已开启，并设置 **增益**。
5. 在 Discord、Zoom、OBS 等软件中，将 `CABLE Output (VB-Audio Virtual Cable)` 选为麦克风。

完整步骤请参阅[用户指南](user-guide)。
