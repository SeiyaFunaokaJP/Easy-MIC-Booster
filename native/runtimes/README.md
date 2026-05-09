# Native binaries

This folder holds the native `rnnoise.dll` used by the AI Noise Suppression
feature. The DLLs are committed to the repo so cloning and running the
standard release build produces a complete bundle without requiring MSVC.
`build/build_rnnoise.ps1` regenerates them from xiph/rnnoise source when you
want to refresh the model or pick up upstream changes.

## Layout

```
native/
  runtimes/
    win-x64/native/rnnoise.dll   <- 64-bit (tracked in git)
    win-x86/native/rnnoise.dll   <- 32-bit (tracked in git)
```

`EasyMICBooster.csproj` copies the matching DLL next to `EasyMICBooster.exe`
based on the publish RID (`win-x64` / `win-x86`). When neither file exists the
project still compiles and runs — the **NS: ON/OFF** toggle just gets disabled.

## Regenerating the DLLs

You only need this when the upstream model is updated or you want to rebuild
from source for verification.

```powershell
# from the project root
powershell -NoProfile -ExecutionPolicy Bypass -File build\build_rnnoise.ps1
```

`build/build_release.bat` also invokes this automatically if the DLLs are
missing.

What the script does:

1. Clones `https://github.com/xiph/rnnoise.git` into `build/rnnoise-src/`
   (cached on subsequent runs; pass `-Clean` to force a fresh clone).
2. Downloads the model-weights tarball from `media.xiph.org` (the hash in
   `model_version` keeps this reproducible).
3. Compiles a single-DLL build with MSVC (`cl.exe /LD /MT /O2`) for both x64
   and x86, linking the C runtime statically so the DLL has no extra
   dependency.
4. Drops the resulting DLLs into the directories above.

Requires Visual Studio 2022 with the *Desktop development with C++* workload
(supplies `cl.exe` and the Windows SDKs). Visual Studio 2019 also works — the
script searches both. `git` must be on `PATH`.

## Source and license

- Upstream: <https://github.com/xiph/rnnoise>
- License: BSD 3-Clause (Xiph.Org, Mozilla, Jean-Marc Valin). Full text is
  reproduced in `THIRD-PARTY-NOTICES.txt` at the repo root and shipped inside
  the release ZIPs.
