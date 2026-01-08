# User Guide

This guide explains how to install and use Easy MIC Booster for daily operations.

## System Requirements

- **OS**: Windows 10/11
- **Virtual Audio Driver**: VB-Audio Virtual Cable (or equivalent)

## Installation Steps

1.  **Install VB-Cable**
    *   Download it from [vb-audio.com](https://vb-audio.com/Cable/).
    *   Unzip the file and run `VBCABLE_Setup_x64.exe` as **Administrator** to install.
    *   Restart your PC after installation.

2.  **Download Easy MIC Booster**
    *   Download the latest zip file from the GitHub Release page.
    *   Unzip it to any folder (e.g., `C:\Tools\EasyMICBooster`).

## First-Time Setup

1.  **Launch the Application**
    *   Run `EasyMICBooster.exe` located inside the unzipped folder.

2.  **Device Settings**
    *   **Input Device**: Select your physical microphone.
    *   **Output Device**: Select the virtual cable input (e.g., "CABLE Input (VB-Audio Virtual Cable)").

3.  **Check Routing**
    *   Turn the large switch in the center of the screen to **ON**.
    *   Speak into the microphone and check if the level meter moves.

4.  **Configure Each App**
    *   Open your chat app (Discord, Zoom, etc.) or streaming software (OBS).
    *   Select "**CABLE Output (VB-Audio Virtual Cable)**" for the microphone (Input Device) in the audio settings.

> [!IMPORTANT]
> Do **not** set "CABLE Output" as the "Default Playback Device" in Windows sound settings. This will cause howling (feedback loop).
