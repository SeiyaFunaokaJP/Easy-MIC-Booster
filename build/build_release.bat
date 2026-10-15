@echo off
cd /d "%~dp0"

echo ==========================================
echo Building Release Configuration (Self-Contained)
echo ==========================================

REM Read version from version.txt in root (parent dir)
set /p VERSION=<..\version.txt
echo Version: %VERSION%

REM ---------------------------------------------------------
REM Build native rnnoise.dll for both architectures if missing.
REM (DLLs are tracked in git; this only regenerates them if deleted.)
REM ---------------------------------------------------------
if not exist "..\native\runtimes\win-x64\native\rnnoise.dll" goto BUILD_RNNOISE
if not exist "..\native\runtimes\win-x86\native\rnnoise.dll" goto BUILD_RNNOISE
goto SKIP_RNNOISE
:BUILD_RNNOISE
echo.
echo Building rnnoise.dll (x64 and x86)...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build_rnnoise.ps1"
if %ERRORLEVEL% NEQ 0 (
    echo rnnoise build failed!
    pause
    exit /b %ERRORLEVEL%
)
:SKIP_RNNOISE

REM Define build directories
set BUILD_DIR=%~dp0
set BIN_DIR=%BUILD_DIR%bin
set ZIP_DIR=%BUILD_DIR%zip

REM Clean old build artifacts
if exist "%BIN_DIR%" rmdir /s /q "%BIN_DIR%"
if exist "%ZIP_DIR%" rmdir /s /q "%ZIP_DIR%"

mkdir "%BIN_DIR%"
mkdir "%ZIP_DIR%"

REM ---------------------------------------------------------
REM Inno Setup 6 compiler (optional: without it only the zips are built).
REM Install with: winget install JRSoftware.InnoSetup
REM ---------------------------------------------------------
set ISCC=
for %%P in ("%LOCALAPPDATA%\Programs\Inno Setup 6\ISCC.exe" "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" "%ProgramFiles%\Inno Setup 6\ISCC.exe") do if not defined ISCC if exist %%P set ISCC=%%~P
if not defined ISCC for /f "delims=" %%P in ('where ISCC.exe 2^>nul') do if not defined ISCC set ISCC=%%P
if defined ISCC (echo Inno Setup: %ISCC%) else (echo WARNING: Inno Setup 6 ^(ISCC.exe^) not found; skipping the installers.)

REM ---------------------------------------------------------
REM Build for x64
REM ---------------------------------------------------------
echo.
echo Building for Windows x64...
dotnet publish ..\src\EasyMICBooster.csproj -c Release -r win-x64 --self-contained true -o "%BIN_DIR%\x64" /p:AssemblyVersion=%VERSION%.0 /p:FileVersion=%VERSION%.0 /p:Version=%VERSION%

if %ERRORLEVEL% NEQ 0 (
    echo x64 Build Failed!
    pause
    exit /b %ERRORLEVEL%
)

echo Zipping x64...
powershell -Command "Compress-Archive -Path '%BIN_DIR%\x64\*' -DestinationPath '%ZIP_DIR%\EasyMICBooster-x64-v%VERSION%-portable.zip' -Force"

if defined ISCC (
    echo Building x64 installer...
    "%ISCC%" /Q /DAppVersion=%VERSION% /DArch=x64 "/DPkgDir=%BIN_DIR%\x64" "/DOutputDir=%ZIP_DIR%" "%~dp0..\installer\EasyMICBooster.iss"
    if errorlevel 1 (
        echo x64 Installer Failed!
        pause
        exit /b 1
    )
)

REM ---------------------------------------------------------
REM Build for x86
REM ---------------------------------------------------------
echo.
echo Building for Windows x86...
dotnet publish ..\src\EasyMICBooster.csproj -c Release -r win-x86 --self-contained true -o "%BIN_DIR%\x86" /p:AssemblyVersion=%VERSION%.0 /p:FileVersion=%VERSION%.0 /p:Version=%VERSION%

if %ERRORLEVEL% NEQ 0 (
    echo x86 Build Failed!
    pause
    exit /b %ERRORLEVEL%
)

echo Zipping x86...
powershell -Command "Compress-Archive -Path '%BIN_DIR%\x86\*' -DestinationPath '%ZIP_DIR%\EasyMICBooster-x86-v%VERSION%-portable.zip' -Force"

if defined ISCC (
    echo Building x86 installer...
    "%ISCC%" /Q /DAppVersion=%VERSION% /DArch=x86 "/DPkgDir=%BIN_DIR%\x86" "/DOutputDir=%ZIP_DIR%" "%~dp0..\installer\EasyMICBooster.iss"
    if errorlevel 1 (
        echo x86 Installer Failed!
        pause
        exit /b 1
    )
)

echo.
echo ==========================================
echo Build Success!
echo ==========================================
echo User: %USERNAME%
echo Date: %DATE% %TIME%
echo.
echo Output:
echo   %ZIP_DIR%\EasyMICBooster-x64-v%VERSION%-portable.zip
echo   %ZIP_DIR%\EasyMICBooster-x86-v%VERSION%-portable.zip
if defined ISCC (
    echo   %ZIP_DIR%\EasyMICBooster-x64-v%VERSION%-setup.exe
    echo   %ZIP_DIR%\EasyMICBooster-x86-v%VERSION%-setup.exe
)
echo.
REM Clean up src/bin and src/obj if they were created
if exist ..\src\bin rmdir /s /q ..\src\bin
if exist ..\src\obj rmdir /s /q ..\src\obj

REM Clean up build/bin/Release (byproduct)
if exist "%BIN_DIR%\Release" rmdir /s /q "%BIN_DIR%\Release"

pause
