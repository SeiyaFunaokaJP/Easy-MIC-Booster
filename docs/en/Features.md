# Features & Usage

Easy MIC Booster provides professional audio processing tools to improve microphone sound quality.

## Main Interface

- **Gain**: Adjust the overall output volume using the large central graph.
  - **Flat Mode**: When enabled, the entire equalizer line moves up and down, functioning as a pure gain control.
- **Mute**: Click the microphone icon or toggle the switch to mute/unmute.
- **Startup**: Check "Startup" at the bottom of the screen to automatically launch the app when Windows starts.

## Audio Processing Effects

Effects are applied in the following order:

1.  **Noise Gate**
2.  **Equalizer**
3.  **Limiter**

### Noise Gate

Cuts out white noise (hiss) when not speaking or keyboard typing sounds.

- **Threshold**: Adjust with the slider. Audio below this level will be muted.

### Equalizer

Adjust the tone of your voice with a visually operable parametric equalizer.

#### Controls
- **Graph Operation**:
    - **Add Point**: Click the "Add Point" button or click on an empty space on the graph (except in Flat Mode).
    - **Move Point**: Left-click and drag a point to adjust Frequency (Hz) and Gain (dB).
    - **Remove Point**: **Right-click** a point to remove it.
- **Flat Mode**: Locks all points. Moving one moves the entire line up or down, which is useful when you want to increase volume without changing the sound quality.
- **Unlock Limit**: Normally limited to ±50dB, but checking this extends it to ±100dB.
- **Frequency Range**: You can change the display range (zoom) by directly editing the numbers below the graph.

#### Presets
You can save your favorite equalizer settings.
- **Save**: Enter a name in the dropdown and press "Save Preset".
- **Load**: Select an existing preset from the list.
- **Delete**: Press the trash can icon to delete the selected preset.
- **Folder**: Press the 📂 button to open the folder where preset files (`.json`) are saved.

### Limiter

Prevents sound distortion (clipping) caused by sudden loud noises.

> [!TIP]
> The limiter is enabled by default to protect the listener's ears from loud noises.

## Saving Settings

Device selection, switch positions, equalizer settings, etc., are automatically saved to `config.json` when the app closes and are restored upon the next launch.
