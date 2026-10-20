---
title: User Guide
layout: default
nav_order: 2
---

# User Guide
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## System Requirements

- **OS**: Windows 10 / 11 (x64, x86 or ARM64)
- **Virtual audio driver**: [VB-CABLE](https://vb-audio.com/Cable/) or an equivalent virtual cable

Linux and macOS builds are also published but are experimental; see the [Developer Guide](developer-guide#supported-platforms) for how to route audio there.

## Installation

### 1. Install VB-CABLE

1. Download it from [vb-audio.com/Cable](https://vb-audio.com/Cable/).
2. Unzip it and run `VBCABLE_Setup_x64.exe` **as administrator**.
3. Restart your PC.

### 2. Install Easy MIC Booster

Download the latest release from the [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest) page, either:

- **Installer** (`EasyMICBooster-x64-vX.Y.Z-setup.exe`): installs for the current user without administrator rights and adds a Start menu entry. Uninstall it from Windows Settings > Apps; your settings are kept.
- **Portable** (`EasyMICBooster-x64-vX.Y.Z-portable.zip`): unzip into any folder and run `EasyMICBooster.exe`.

Use the `arm64` files on Windows on ARM and the `x86` files only on 32-bit Windows. The ARM64 build does not include the *High quality* noise suppression method.

## First-Time Setup

### Choose the Devices

In the **Routing** card on the left:

| Field | Choose |
|:------|:-------|
| **Input (microphone)** | Your physical microphone, or *Default Device* to follow Windows |
| **Output (virtual cable)** | `CABLE Input (VB-Audio Virtual Cable)` |

The meter under each list shows the level before and after processing. The line between them summarizes what is active (for example `+12.0 dB · NS · Gate`).

{: .warning }
If the output is your speakers or headphones (the default playback device), you will hear yourself. Always pick the virtual cable as the output.

### Set the Gain

1. Make sure **BOOST ON** is enabled at the top right.
2. Speak normally and raise **Gain** in the **Sound** card until the output meter peaks in the green-to-yellow area. The −1 / +1 buttons step by 1 dB.
3. If the meter turns red and *Limiter* appears, the limiter is holding the peaks down; lower the gain a little.

The status chip next to the title shows **Live** with the sample rate and latency when audio is flowing.

### Configure the Receiving App

In Discord, Zoom, OBS, Teams, etc., open the audio settings and select `CABLE Output (VB-Audio Virtual Cable)` as the microphone (input device).

{: .warning }
Do **not** set `CABLE Output` as the **default playback device** in the Windows sound settings. That creates a feedback loop (howling).

## Daily Use

- **Bypass** -- switch **BOOST ON** off to pass the microphone through without gain or EQ (noise suppression and the gate stay as set).
- **Tray** -- closing the window keeps the app running in the notification area. Right-click the tray icon to toggle boost or noise suppression, change the gain, switch presets, device profiles and devices, or exit. Click it to open the window.
- **Presets** -- save your sound with **Save as new** and switch between presets in the **Presets** card or from the tray. See [Features](features#presets).
- **Device profiles** -- save a microphone/output pair and switch between them (for example a headset at the desk and a USB mic for streaming). See [Features](features#device-profiles).
- **Unplugged devices** -- if the microphone or output disappears, the status turns red; routing resumes automatically when it is connected again.

## Settings

Open **Settings** with the gear icon at the top right.

| Setting | Description |
|:--------|:------------|
| **Language** | UI language (follows Windows on first run) |
| **Start when I sign in** | Launch automatically when you sign in to Windows (starts in the tray) |
| **Keep running in the tray when closed** | Off: closing the window exits the app |
| **Allow gain above +50 dB** | Raises the gain limit to +100 dB |
| **Output ceiling** | Peak level the limiter holds the output under (default −1.0 dBFS) |
| **Check for updates** | Checks GitHub for a newer release at startup |

The buttons below open the settings folder, the GitHub page and the Sponsor page, or exit the app.

Settings, presets and device profiles are saved automatically to `%APPDATA%\EasyMICBooster` and restored on the next launch. Settings from v1.x are imported on first start.

See [Features](features) for every audio control in detail.
