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

- **OS**: Windows 10 / 11
- **Virtual Audio Driver**: VB-Audio Virtual Cable (or equivalent)

## Installation

### 1. Install VB-CABLE

1. Download from [vb-audio.com/Cable](https://vb-audio.com/Cable/).
2. Unzip and run `VBCABLE_Setup_x64.exe` **as Administrator**.
3. Restart your PC after installation.

### 2. Install Easy MIC Booster

Download the latest release from the [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases) page, either:

- **Installer** (`EasyMICBooster-x64-vX.Y.Z-setup.exe`): installs for the current user without administrator rights and adds a Start menu entry. Uninstall from Windows Settings > Apps; your settings in `%APPDATA%\EasyMICBooster` are kept.
- **Portable** (`EasyMICBooster-x64-vX.Y.Z-portable.zip`): unzip into any folder (for example `C:\Tools\EasyMICBooster`).

Use the `x86` files only on 32-bit Windows.

## First-Time Setup

### Launch the Application

Start Easy MIC Booster from the Start menu (installer) or run `EasyMICBooster.exe` from the unzipped folder (portable).
The UI language follows Windows on first run and can be changed with the selector at the top right.

### Configure Devices

| Field | Value |
|:------|:------|
| **Input Device** | Your physical microphone |
| **Output Device** | `CABLE Input (VB-Audio Virtual Cable)` |

### Verify Routing

1. Toggle the large switch in the center to **ON**.
2. Speak into the microphone — the level meter should respond.

### Configure Receiving Apps

In Discord, Zoom, OBS, etc., open audio settings and select `CABLE Output (VB-Audio Virtual Cable)` as the microphone (input device).

{: .warning }
Do **not** set `CABLE Output` as the **Default Playback Device** in Windows sound settings. Doing so creates a feedback loop (howling).

## Daily Operation

- **Mute** -- click the microphone icon or toggle the switch.
- **Auto-launch on Windows startup** -- enable the **Startup** checkbox at the bottom of the window.
- **Save settings** -- device choices, switch states, and EQ are written to `config.json` automatically when the app closes and restored on next launch.

See [Features](features) for detailed audio processing controls.
