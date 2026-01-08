@echo off
cd /d "%~dp0"

echo ==========================================
echo Building Debug Configuration...
echo ==========================================

REM Read version from version.txt in root (parent dir)
set /p VERSION=<..\version.txt
echo Version: %VERSION%

REM Clean obj folder
if exist obj rmdir /s /q obj

REM Restore and Build
dotnet restore ..\EasyMICBooster.sln
dotnet build ..\EasyMICBooster.sln -c Debug

if %ERRORLEVEL% NEQ 0 (
    echo Build Failed!
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo Build Success!
pause
