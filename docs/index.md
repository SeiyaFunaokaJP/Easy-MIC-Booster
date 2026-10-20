---
title: Home
layout: default
nav_order: 1
---

# Easy MIC Booster

Microphone gain amplification & high-quality audio processing for Windows.
{: .fs-6 .fw-300 }

Captures your microphone, cleans and boosts it — noise suppression, equalizer, gain, noise gate and limiter — and routes the result to a virtual audio device such as VB-CABLE, so every app hears the processed voice.

[Get Started](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[Download](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Easy MIC Booster main window](images/app_top.png)

---

## Key Features

- **Software Amplifier** -- boosts a quiet microphone beyond the Windows gain limit (up to +50 dB, or +100 dB when unlocked)
- **Noise Suppression** -- removes fans, keystrokes and room noise even while you speak; choose Light, Standard (RNNoise) or High quality (DeepFilterNet3)
- **Parametric Equalizer** -- shape your tone on a live spectrum graph
- **Noise Gate** -- mutes the mic between phrases for completely silent gaps
- **Limiter** -- keeps peaks below a ceiling so sudden loud sounds never clip
- **Presets & Device Profiles** -- switch sounds and microphone/output pairs in one click
- **Tray Control** -- keeps running in the notification area; boost, presets and devices are one right-click away
- **8 Languages** -- English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español

## How It Works

```
Microphone
    │
    ▼
Capture (WASAPI on Windows)
    │
    ▼
Noise Suppression → Equalizer → Gain → Noise Gate → Limiter
    │
    ▼
Virtual audio device (e.g. CABLE Input)
    │
    ▼
Discord / Zoom / OBS / Teams / games …
```

## Quick Start

1. Install [VB-CABLE](https://vb-audio.com/Cable/).
2. Download the latest release (installer `...-setup.exe`, or the portable zip) and start Easy MIC Booster.
3. Under **Routing**, pick your microphone as **Input** and `CABLE Input (VB-Audio Virtual Cable)` as **Output**.
4. Make sure **BOOST ON** is enabled at the top right and set the **Gain**.
5. In Discord, Zoom, OBS, etc., select `CABLE Output (VB-Audio Virtual Cable)` as the microphone.

See the [User Guide](user-guide) for the full walkthrough.
