---
title: Features
layout: default
nav_order: 3
---

# Features
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Main Window

| Area | What it does |
|:-----|:-------------|
| **Header** | Status chip (Live / Bypassed / device problems, with sample rate and latency), the **BOOST ON** switch and the Settings gear |
| **Routing** | Input and output devices with level meters, and device profiles |
| **Presets** | Your presets and the built-in ones |
| **Sound** | Gain, Simple / Equalizer mode and the EQ graph with a live spectrum |
| **Noise Suppression** | On/off, method and strength |
| **Noise Gate** | On/off and the threshold |

**BOOST ON** turns the gain and the EQ on or off. While it is off the microphone passes through at its original level; noise suppression and the noise gate keep working as set.

## Signal Chain

Audio passes through the effects in this order:

```
Input → Noise Suppression → Equalizer → Gain → Noise Gate → Limiter → Output
```

The chain runs internally at 48 kHz mono — the format the noise suppressors expect — so input
of any sample rate or channel count is converted up-front and the output is
re-mixed by Windows for the destination device.

## Gain

Set the gain with the slider, the −1 / +1 buttons, or by dragging the gain line on the graph up or down. The range is −30 dB to +50 dB; **Allow gain above +50 dB** in Settings extends it to +100 dB.

## Equalizer

The **Sound** card has two modes:

- **Simple** -- only the gain is applied. Drag anywhere on the graph to set it.
- **Equalizer** -- adds up to 16 EQ points on top of the gain.

The graph shows the response curve over a live spectrum of the output.

### Graph Operation

| Action | Operation |
|:-------|:----------|
| **Add a point** | Double-click on the graph, or click **Add point** |
| **Move a point** | Drag it — left/right changes the frequency, up/down the gain |
| **Change the width (Q)** | Scroll the mouse wheel over the point |
| **Change type / delete** | Right-click the point |
| **Edit exact values** | Click a point and use the fields below the graph (type, frequency, gain, Q) |
| **Clear all points** | **Reset EQ** |

Point types: **Peak**, **Low shelf**, **High shelf**, **Low cut (high-pass)** and **High cut (low-pass)**.

## Noise Suppression

Removes background noise while preserving voice. Unlike a noise gate, this
works **even while you are speaking**. Three methods are available:

| Method | How it works | Good at | CPU (one core) | Latency |
|:-------|:-------------|:--------|:---------------|:--------|
| **Light** | Spectral noise reduction (Wiener filter, noise floor learned per frequency) | Steady noise: fans, hiss, air conditioner | ~0.05 % | +10 ms |
| **Standard** | [RNNoise](https://github.com/xiph/rnnoise) neural network | Steady noise plus keystrokes and mouse clicks | ~0.5 % | +10 ms |
| **High quality** | [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet) neural network, full band | The most natural voice, even with loud or changing noise | ~2 % | +40 ms |

| Control | Description |
|:--------|:------------|
| **Toggle** | Turns noise suppression on or off |
| **Method** | Light / Standard / High quality (hover for details) |
| **Strength** | Mix between the processed and the original signal |
| **Voice** | Meter of how confident the suppressor is that you are speaking |

- The method and strength are saved in presets.
- High quality loads its model in the background the first time it is
  selected (well under a second); until then the signal passes through
  unprocessed. It is not available in the ARM64 Windows build.
- Light needs a second or two to learn the noise after it is switched on.
- All methods are built into the executable: Standard uses
  [nnnoiseless](https://github.com/jneem/nnnoiseless), a Rust port of RNNoise
  (BSD-3-Clause); High quality uses DeepFilterNet's `libDF` on the pure-Rust
  [tract](https://github.com/sonos/tract) runtime (MIT / Apache-2.0).

{: .note }
The noise gate is still useful in addition to noise suppression — the
suppressors attenuate noise but do not always reach absolute silence between phrases.
The gate cleans up the residual floor.

## Noise Gate

Mutes the microphone while its level stays below a threshold, so the gaps between phrases are completely silent.

- Turn it on with the toggle, then **drag the marker** on the meter to just above your background noise (or click where it should go).
- The meter shows the level after gain; the badge reads **Passing** while you are heard and **Muted** while the gate is closed.

## Limiter

Always on at the end of the chain. It holds the output peaks under the **Output ceiling** (Settings, default −1.0 dBFS), so a sudden loud sound never clips. While it is reducing the level, the output meter turns red and *Limiter: −x dB* appears under it.

## Presets

A preset stores the gain, the mode and EQ points, noise suppression (on/off, method, strength) and the noise gate.

| Action | How |
|:-------|:----|
| **Apply** | Click a preset in **My presets** or **Built-in** |
| **Save as new** | **Save as new**, then enter a name |
| **Save changes** | **Save** (appears when the current user preset is modified) |
| **Undo changes** | **Revert** |
| **Rename / overwrite / delete** | The **⋯** button on a preset, or right-click it |
| **Open folder** | The folder icon opens the folder with the preset `.json` files |

When you switch away from a modified preset, Easy MIC Booster asks whether to save it first. Built-in presets: *Original (0 dB)*, *Boost +12 dB*, *Boost +24 dB*, *Clear voice*, *Warm voice* and *Streaming*.

## Device Profiles

A device profile remembers an input/output pair. Use the **Device Profile** list in the **Routing** card:

- **Save current devices as profile** -- stores the current input and output under a name.
- Pick a profile to switch both devices at once; delete it from the same list.

Profiles can also be switched from the tray menu.

## Tray Menu

Right-click the tray icon for:

- **Boost** and **Noise suppression** on/off
- **Gain** -- ±3 dB steps, common values and *Use equalizer*
- **Presets**, **Device profiles**, **Input** and **Output**
- **Open Easy MIC Booster** and **Exit**

## Persistent Settings

Everything is saved automatically to `%APPDATA%\EasyMICBooster` (`settings.json`, plus the `Presets` and `DeviceProfiles` folders) and restored on the next launch. Settings and presets from v1.x are imported automatically.
