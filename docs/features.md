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

## Main Interface

| Control | Description |
|:--------|:------------|
| **Gain** | Adjust overall output volume from the central graph |
| **Flat Mode** | Move the entire EQ line up/down as a pure gain control |
| **Mute** | Click the microphone icon or toggle the switch |
| **Startup** | Auto-launch on Windows startup |

## Signal Chain

Audio passes through DSP effects in the following order:

```
Input → AI Noise Suppression → Equalizer → Gain → Noise Gate → Limiter → Output
```

The chain runs internally at 48 kHz mono — RNNoise's required format — so input
of any sample rate or channel count is converted up-front and the output is
re-mixed by Windows for the destination device.

## AI Noise Suppression (RNNoise)

Removes constant background noise (fans, hiss, air conditioner) **and**
non-stationary noise (keystrokes, mouse clicks, paper rustling) while
preserving voice. Unlike a noise gate, this works **even while you are
speaking** — the trained network separates speech from noise frame by frame.

| Control | Description |
|:--------|:------------|
| **NS: ON / OFF** | Toggle button next to the Noise Gate section |

- Adds about 10 ms of latency when enabled (one 480-sample frame at 48 kHz).
- Implemented via the [RNNoise](https://github.com/xiph/rnnoise) library
  (BSD-3-Clause). The `rnnoise.dll` ships inside the release ZIPs; if it ever
  goes missing the toggle is disabled automatically and the rest of the app
  keeps working.

{: .note }
The noise gate is still useful in addition to AI Noise Suppression — RNNoise
attenuates noise but does not always reach absolute silence between phrases.
The gate cleans up the residual floor.

## Noise Gate

Mutes the channel when the input level falls below a threshold. Useful for
clean silent gaps between phrases.

- **Threshold** -- audio below this level is muted. Adjust with the slider.

## Equalizer

A parametric equalizer with a visual graph for tone shaping.

### Graph Operation

| Action | Operation |
|:-------|:----------|
| **Add point** | Click **Add Point**, or click empty graph area (disabled in Flat Mode) |
| **Move point** | Left-click and drag — adjusts Frequency (Hz) and Gain (dB) |
| **Remove point** | **Right-click** the point |

### Modes

- **Flat Mode** -- locks all points; moving one shifts the entire line. Useful for changing volume without altering tone.
- **Unlock Limit** -- extends the gain range from ±50 dB to ±100 dB.
- **Frequency Range** -- edit the numeric fields below the graph to zoom the display range.

### Presets

Save and recall favorite EQ configurations.

| Action | How |
|:-------|:----|
| **Save** | Type a name in the dropdown and click **Save Preset** |
| **Load** | Pick an existing preset from the list |
| **Delete** | Click the trash icon |
| **Open folder** | Click 📂 to open the folder containing preset `.json` files |

## Limiter

Prevents clipping from sudden loud sounds.

{: .note }
The limiter is enabled by default to protect listeners from sudden loud noises.

## Persistent Settings

Device selection, switch states, EQ points, presets, and other UI state are written to `config.json` on exit and restored on next launch.
