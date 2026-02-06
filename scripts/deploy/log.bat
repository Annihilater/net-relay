@echo off
REM View net-relay logs
REM Usage: scripts\log.bat [lines]

setlocal

set SCRIPT_DIR=%~dp0
set INSTALL_DIR=%SCRIPT_DIR%..
set LOG_FILE=%INSTALL_DIR%\logs\net-relay.log

if not exist "%LOG_FILE%" (
    echo Log file not found: %LOG_FILE%
    echo Start the server first with: scripts\start.bat
    exit /b 1
)

set LINES=%1
if "%LINES%"=="" set LINES=50

echo =======================================
echo   Last %LINES% lines of log
echo =======================================
echo.

powershell -Command "Get-Content '%LOG_FILE%' -Tail %LINES%"
