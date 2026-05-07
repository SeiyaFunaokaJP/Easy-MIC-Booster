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
Input → Noise Gate → Equalizer → Limiter → Gain → Output
```

## Noise Gate

Cuts background hiss and keyboard typing when you are not speaking.

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
