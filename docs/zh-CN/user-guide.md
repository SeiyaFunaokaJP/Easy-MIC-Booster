---
title: 用户指南
layout: default
parent: 首页
nav_order: 1
---

# 用户指南
{: .no_toc }

## 目录
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## 系统要求 {#system-requirements}

- **操作系统**：Windows 10 / 11（x64、x86 或 ARM64）
- **虚拟音频驱动**：[VB-CABLE](https://vb-audio.com/Cable/) 或同类虚拟音频线

我们也发布了 Linux 和 macOS 版本，但目前仍处于实验阶段；在这些平台上如何路由音频，请参阅[开发者指南](developer-guide#supported-platforms)。

## 安装 {#installation}

### 1. 安装 VB-CABLE {#1-install-vb-cable}

1. 从 [vb-audio.com/Cable](https://vb-audio.com/Cable/) 下载。
2. 解压后，**以管理员身份**运行 `VBCABLE_Setup_x64.exe`。
3. 重启电脑。

### 2. 安装 Easy MIC Booster {#2-install-easy-mic-booster}

从 [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest) 页面下载最新版本，可选择以下任一种：

- **安装程序**（`EasyMICBooster-x64-vX.Y.Z-setup.exe`）：无需管理员权限，为当前用户安装，并添加开始菜单项。可在 Windows 设置 > 应用 中卸载；卸载后设置会保留。
- **便携版**（`EasyMICBooster-x64-vX.Y.Z-portable.zip`）：解压到任意文件夹，运行 `EasyMICBooster.exe` 即可。

在 ARM 版 Windows 上请使用 `arm64` 文件，`x86` 文件仅用于 32 位 Windows。ARM64 版本不包含 *高精度* 降噪方式。

## 首次设置 {#first-time-setup}

### 选择设备 {#choose-the-devices}

在左侧的 **路由** 卡片中：

| 项目 | 选择 |
|:------|:-------|
| **输入（麦克风）** | 你的实体麦克风，或选择 *默认设备* 以跟随 Windows 设置 |
| **输出（虚拟音频线）** | `CABLE Input (VB-Audio Virtual Cable)` |

每个列表下方的电平表分别显示处理前和处理后的电平。两者之间的一行文字概括了当前启用的处理（例如 `+12.0 dB · NS · Gate`）。

{: .warning }
如果输出设备是扬声器或耳机（默认播放设备），你会听到自己的声音。请务必将虚拟音频线选为输出。

### 设置增益 {#set-the-gain}

1. 确认右上角的 **BOOST ON** 已开启。
2. 以正常音量说话，同时在 **声音** 卡片中调高 **增益**，直到输出电平表的峰值落在绿色到黄色区域。−1 / +1 按钮每次调整 1 dB。
3. 如果电平表变红并显示 *限幅器*，说明限幅器正在压制峰值；请稍微降低增益。

音频正常传输时，标题旁的状态标签会显示 **运行中**，以及采样率和延迟。

### 设置接收端应用 {#configure-the-receiving-app}

在 Discord、Zoom、OBS、Teams 等软件中打开音频设置，将 `CABLE Output (VB-Audio Virtual Cable)` 选为麦克风（输入设备）。

{: .warning }
**不要**在 Windows 声音设置中将 `CABLE Output` 设为 **默认播放设备**。这会形成反馈回路（啸叫）。

## 日常使用 {#daily-use}

- **旁路** -- 关闭 **BOOST ON** 后，麦克风声音将不经增益和 EQ 直接通过（降噪和噪声门保持当前设置）。
- **托盘** -- 关闭窗口后，应用会继续在通知区域中运行。右键单击托盘图标，可以开关增强或降噪、调整增益、切换预设、设备配置和设备，或退出应用。单击图标即可打开窗口。
- **预设** -- 使用 **另存为新预设** 保存你的声音，并在 **预设** 卡片或托盘中切换预设。请参阅[功能](features#presets)。
- **设备配置** -- 保存麦克风/输出设备组合并在它们之间切换（例如在桌前使用耳机，直播时使用 USB 麦克风）。请参阅[功能](features#device-profiles)。
- **设备断开** -- 如果麦克风或输出设备消失，状态会变为红色；设备重新连接后将自动恢复路由。

## 设置 {#settings}

点击右上角的齿轮图标打开 **设置**。

| 设置项 | 说明 |
|:--------|:------------|
| **语言** | 界面语言（首次运行时跟随 Windows） |
| **登录时启动** | 登录 Windows 时自动启动（在托盘中启动） |
| **关闭窗口后继续在托盘中运行** | 关闭此项时：关闭窗口即退出应用 |
| **允许增益超过 +50 dB** | 将增益上限提高到 +100 dB |
| **输出上限** | 限幅器将输出峰值保持在此电平以下（默认 −1.0 dBFS） |
| **检查更新** | 启动时在 GitHub 上检查是否有新版本 |

下方的按钮可以打开设置文件夹、GitHub 页面和赞助页面，或退出应用。

设置、预设和设备配置会自动保存到 `%APPDATA%\EasyMICBooster`，并在下次启动时恢复。首次启动时会导入 v1.x 的设置。

各项音频控制的详细说明请参阅[功能](features)。
