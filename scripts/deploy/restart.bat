@echo off
REM Restart net-relay
REM Usage: scripts\restart.bat

setlocal

set SCRIPT_DIR=%~dp0

echo Restarting net-relay...

call "%SCRIPT_DIR%stop.bat"
timeout /t 2 /nobreak > nul
call "%SCRIPT_DIR%start.bat"
