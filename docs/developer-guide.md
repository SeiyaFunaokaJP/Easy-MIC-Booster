---
title: Developer Guide
layout: default
nav_order: 4
---

# Developer Guide
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Build Requirements

- **.NET 8.0 SDK** -- [dotnet.microsoft.com](https://dotnet.microsoft.com/download/dotnet/8.0)
- **IDE** -- Visual Studio 2022 (recommended) or VS Code

## Project Structure

```
Easy-MIC-Booster/
├── src/                       Source code
│   └── Lang/                  Language files (json)
├── docs/                      Documentation site (this Jekyll site)
│   ├── _config.yml
│   ├── _includes/
│   ├── images/
│   ├── ja/                    Japanese translation
│   └── *.md                   English pages
├── native/                    Bundled native binaries
│   └── runtimes/
│       ├── win-x64/native/    rnnoise.dll for x64
│       └── win-x86/native/    rnnoise.dll for x86
└── build/                     Build scripts and artifacts
    ├── build_debug.bat        Debug build
    ├── build_release.bat      Release build
    ├── build_rnnoise.ps1      Builds rnnoise.dll from xiph/rnnoise source
    ├── Directory.Build.props  Common MSBuild settings
    ├── bin/                   Build output (x64 / x86)
    └── zip/                   Distribution packages
```

## Native Dependencies

The AI Noise Suppression feature uses [RNNoise](https://github.com/xiph/rnnoise)
(BSD-3-Clause) via P/Invoke. Pre-built `rnnoise.dll` binaries are committed
under `native/runtimes/<rid>/native/` so a clean clone produces a complete
release without requiring an MSVC setup. To rebuild them from source:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File build\build_rnnoise.ps1
```

The script clones `xiph/rnnoise`, downloads the model weights blob from
`media.xiph.org` (hash pinned via the upstream `model_version` file), and
compiles a single-DLL build with `cl.exe /LD /MT /O2`. Visual Studio 2019/2022
with the *Desktop development with C++* workload is required when running it.
`build_release.bat` invokes the script automatically if the DLLs are missing.

## Building

{: .warning }
Close any running instance launched from `build/bin` before building. Locked files will fail the build.

### Command Line

```powershell
dotnet build EasyMICBooster.sln -c Release
```

### Build Script

```cmd
.\build\build_release.bat
```

Executables are written to `build/bin/x64` or `build/bin/x86`.

## Documentation Site

This site is built with [Jekyll](https://jekyllrb.com/) using the [just-the-docs](https://just-the-docs.com/) remote theme. GitHub Pages renders it directly from the `docs/` directory on the `main` branch.

### Local Preview

```bash
cd docs
bundle init
bundle add jekyll
bundle add jekyll-remote-theme
bundle exec jekyll serve
```

### Adding a Page

1. Create a Markdown file in `docs/` (English) and `docs/ja/` (Japanese).
2. Add front matter:
   ```yaml
   ---
   title: Page Title
   layout: default
   nav_order: <number>
   ---
   ```
3. For Japanese pages, also add `parent: 日本語`.

The language switcher at the bottom of every page links between matching English / Japanese URLs automatically.

## Contributing

See [CONTRIBUTING.md](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/CONTRIBUTING.md) for the development workflow (forks, pull requests, etc.).

## License

[MIT License](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/blob/main/LICENSE).
