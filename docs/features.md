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
Input → Noise Suppression → Equalizer → Gain → Noise Gate → Limiter → Output
```

The chain runs internally at 48 kHz mono — the format the noise suppressors expect — so input
of any sample rate or channel count is converted up-front and the output is
re-mixed by Windows for the destination device.

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
