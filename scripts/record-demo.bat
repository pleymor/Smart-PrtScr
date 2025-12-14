@echo off
echo === Smart PrtScr Demo Recorder ===
echo.
echo This will automate a demo of the app and record it with OBS.
echo.
echo Prerequisites:
echo   1. Smart PrtScr running (npm run tauri:dev)
echo   2. OBS Studio open, ready to record
echo   3. OBS hotkey: Ctrl+F9 = Start/Stop Recording
echo.
pause
powershell -ExecutionPolicy Bypass -File "%~dp0record-demo.ps1"
pause
