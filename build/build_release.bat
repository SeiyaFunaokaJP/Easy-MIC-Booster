@echo off
REM Double-click wrapper for build_release.ps1 (arguments are passed through,
REM e.g. build_release.bat -Arch x64).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build_release.ps1" %*
exit /b %ERRORLEVEL%
