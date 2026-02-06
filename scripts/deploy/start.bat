@echo off
REM Start net-relay in background mode
REM Usage: scripts\start.bat

setlocal

set SCRIPT_DIR=%~dp0
set INSTALL_DIR=%SCRIPT_DIR%..
set BINARY=%INSTALL_DIR%\net-relay.exe
set CONFIG_FILE=%INSTALL_DIR%\config.toml
set LOG_DIR=%INSTALL_DIR%\logs
set PID_FILE=%INSTALL_DIR%\net-relay.pid

cd /d "%INSTALL_DIR%"

REM Create logs directory
if not exist "%LOG_DIR%" mkdir "%LOG_DIR%"

REM Check if binary exists
if not exist "%BINARY%" (
    echo Error: Binary not found: %BINARY%
    exit /b 1
)

REM Check if config exists
if not exist "%CONFIG_FILE%" (
    if exist "%INSTALL_DIR%\config.example.toml" (
        echo Config file not found, copying from example...
        copy "%INSTALL_DIR%\config.example.toml" "%CONFIG_FILE%"
    ) else (
        echo Error: Config file not found: %CONFIG_FILE%
        exit /b 1
    )
)

echo Starting net-relay in background...

REM Start in background using start command
set RUST_LOG=info
start /B "" "%BINARY%" -c "%CONFIG_FILE%" >> "%LOG_DIR%\net-relay.log" 2>&1

timeout /t 2 /nobreak > nul

REM Check if running
tasklist /FI "IMAGENAME eq net-relay.exe" 2>nul | find "net-relay.exe" >nul
if errorlevel 1 (
    echo Failed to start net-relay
    exit /b 1
) else (
    echo net-relay started successfully
    echo   Log file: %LOG_DIR%\net-relay.log
)
