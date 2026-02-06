@echo off
REM Check net-relay status
REM Usage: scripts\status.bat

setlocal

echo =======================================
echo        Net-Relay Status
echo =======================================
echo.

REM Check if running
tasklist /FI "IMAGENAME eq net-relay.exe" 2>nul | find "net-relay.exe" >nul
if errorlevel 1 (
    echo Status:  STOPPED
) else (
    echo Status:  RUNNING
    echo.
    echo Process details:
    tasklist /FI "IMAGENAME eq net-relay.exe" /V
)

echo.
echo =======================================

REM Check ports using netstat
echo Checking ports...
netstat -an | findstr ":1080 " >nul && echo   SOCKS5: :1080 OK || echo   SOCKS5: :1080 NOT LISTENING
netstat -an | findstr ":8080 " >nul && echo   HTTP:   :8080 OK || echo   HTTP:   :8080 NOT LISTENING  
netstat -an | findstr ":3000 " >nul && echo   API:    :3000 OK || echo   API:    :3000 NOT LISTENING

echo.
echo Dashboard: http://localhost:3000
