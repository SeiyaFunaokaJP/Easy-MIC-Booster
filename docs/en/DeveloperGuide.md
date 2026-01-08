# Developer Guide

This guide is for developers who want to build Easy MIC Booster from source or contribute to the project.

## Build Requirements

- **.NET 8.0 SDK**: Download from [dotnet.microsoft.com](https://dotnet.microsoft.com/download/dotnet/8.0).
- **Development Environment**: Visual Studio 2022 (Recommended) or VS Code.

## Project Structure

```
EasyMICBooster/
├── src/                # Source Code
│   └── Lang/           # Language Files (json)
├── docs/               # Documentation (ja/en)
└── build/              # Build Scripts and Artifacts
    ├── build_debug.bat       # Debug Build Script
    ├── build_release.bat     # Release Build Script
    ├── Directory.Build.props # Common MSBuild Settings
    ├── bin/                  # Build Output (x64/x86)
    └── zip/                  # Distribution Packages
```

## How to Build

> [!IMPORTANT]
> Before running the build, please ensure that any instances of Easy MIC Booster running from the `build/bin` folder are closed. The build will fail if files are locked.

### Using Command Line

1. Open a terminal in the project's root directory.
2. Run the following command:

```powershell
dotnet build EasyMICBooster.sln -c Release
```

### Using Build Script

You can build simply by running the included batch file:

```cmd
.\build\build_release.bat
```

The built executables will be output to `build/bin/x64` or `build/bin/x86`.

## Contributing

Please refer to [CONTRIBUTING.md](../../CONTRIBUTING.md) for details on the development flow (forking, pull requests, etc.).

## License

MIT License.
