---
title: Home
layout: default
nav_order: 1
---

# Easy MIC Booster

Microphone gain amplification & high-quality audio processing for Windows.
{: .fs-6 .fw-300 }

Captures microphone input via WASAPI, processes it through Noise Suppression (spectral, RNNoise or DeepFilterNet3), Equalizer, Noise Gate, and Limiter, and routes it to a virtual audio device such as VB-CABLE.

[Get Started](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Easy MIC Booster Top Screen](images/app_top.jpg)

---

## Key Features

- **Software Amplifier** -- eliminates microphone gain shortage at the system level
- **Noise Suppression** -- removal of fans, keystrokes, and other background noise even while you speak; Light, Standard (RNNoise) or High quality (DeepFilterNet3)
- **Noise Gate** -- mutes the channel between phrases for completely silent gaps
- **Parametric Equalizer** -- visual graph editing with savable presets
- **Limiter** -- prevents clipping from sudden loud noises
- **Bilingual UI** -- English / Japanese
- **Auto Save** -- device, switches, and EQ are restored on next launch

## How It Works

```
Microphone Input
       │
       ▼
 WASAPI Capture (PCM)
       │
       ▼
 DSP Chain (Noise Suppression → Equalizer → Gain → Noise Gate → Limiter)
       │
       ▼
 Virtual Audio Device (VB-CABLE Input)
       │
       ▼
 Discord / Zoom / OBS / etc.
```

## Quick Start

1. Install [VB-CABLE](https://vb-audio.com/Cable/).
2. Download the latest release (installer `...-setup.exe`, or the portable zip).
3. Launch Easy MIC Booster.
4. Select your microphone as **Input** and `CABLE Input (VB-Audio Virtual Cable)` as **Output**.
5. Toggle the central switch **ON**.
6. In Discord/Zoom/OBS, select `CABLE Output (VB-Audio Virtual Cable)` as the microphone.

See [User Guide](user-guide) for the full walkthrough.
