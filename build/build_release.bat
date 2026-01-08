@echo off
cd /d "%~dp0"

echo ==========================================
echo Building Release Configuration (Self-Contained)
echo ==========================================

REM Read version from version.txt in root (parent dir)
set /p VERSION=<..\version.txt
echo Version: %VERSION%

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
powershell -Command "Compress-Archive -Path '%BIN_DIR%\x64\*' -DestinationPath '%ZIP_DIR%\EasyMICBooster-x64-v%VERSION%.zip' -Force"

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
powershell -Command "Compress-Archive -Path '%BIN_DIR%\x86\*' -DestinationPath '%ZIP_DIR%\EasyMICBooster-x86-v%VERSION%.zip' -Force"

echo.
echo ==========================================
echo Build Success!
echo ==========================================
echo User: %USERNAME%
echo Date: %DATE% %TIME%
echo.
echo Output zips:
echo   %ZIP_DIR%\EasyMICBooster-x64-v%VERSION%.zip
echo   %ZIP_DIR%\EasyMICBooster-x86-v%VERSION%.zip
echo.
REM Clean up src/bin and src/obj if they were created
if exist ..\src\bin rmdir /s /q ..\src\bin
if exist ..\src\obj rmdir /s /q ..\src\obj

REM Clean up build/bin/Release (byproduct)
if exist "%BIN_DIR%\Release" rmdir /s /q "%BIN_DIR%\Release"

pause
