@echo off
REM fake-nvms.cmd — Windows equivalent of fake-nvms.sh.
echo FAKE-NVMS-ALIVE rev-1
REM Long sleep won't terminate under SIGTERM in Win32; this script
REM just needs to be alive long enough for the test to spawn it,
REM read stdout, then send start_kill via tokio. After that, the
REM test reads the child's exit status — we don't actually need
REM the script to exit on its own.
timeout /t 30 /nobreak > nul
echo FAKE-NVMS-DONE
exit /b 0
