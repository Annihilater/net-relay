@echo off
REM Start net-relay in foreground mode (development)
REM Usage: scripts\dev.bat

setlocal

set SCRIPT_DIR=%~dp0
set INSTALL_DIR=%SCRIPT_DIR%..
set BINARY=%INSTALL_DIR%\net-relay.exe
set CONFIG_FILE=%INSTALL_DIR%\config.toml

cd /d "%INSTALL_DIR%"

REM Create logs directory
if not exist logs mkdir logs

REM Check if binary exists
if not exist "%BINARY%" (
    echo Error: Binary not found: %BINARY%
    exit /b 1
)

REM Check if config exists, if not copy from example
if not exist "%CONFIG_FILE%" (
    if exist "%INSTALL_DIR%\config.example.toml" (
        echo Config file not found, copying from example...
        copy "%INSTALL_DIR%\config.example.toml" "%CONFIG_FILE%"
    ) else (
        echo Error: Config file not found: %CONFIG_FILE%
        exit /b 1
    )
)

echo Starting net-relay in foreground (development mode)...
echo   Binary:  %BINARY%
echo   Config:  %CONFIG_FILE%
echo.

set RUST_LOG=info
"%BINARY%" -c "%CONFIG_FILE%"
