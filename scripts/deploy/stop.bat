@echo off
REM Stop net-relay
REM Usage: scripts\stop.bat

setlocal

echo Stopping net-relay...

taskkill /IM net-relay.exe /F 2>nul
if errorlevel 1 (
    echo net-relay is not running
) else (
    echo net-relay stopped
)
