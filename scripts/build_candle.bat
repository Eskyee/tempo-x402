@echo off
REM Build paper-bench inside the MSVC x64 developer environment so nvcc can find cl.exe.
REM Invoke: scripts\build_candle.bat  (from repo root)
REM
REM Extra knobs vs a plain vcvars64 call:
REM   * vswhere.exe's directory is prepended to PATH — Git Bash often strips it.
REM   * NVCC_CCBIN points directly at cl.exe, which makes bindgen_cuda pass
REM     `-allow-unsupported-compiler` so CUDA 11.7 stops rejecting newer MSVC
REM     toolsets (14.42.x here). See bindgen_cuda-0.1.6 src/lib.rs:253.

set "PATH=C:\Program Files (x86)\Microsoft Visual Studio\Installer;%PATH%"
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if errorlevel 1 (
    echo [build_candle] vcvars64.bat failed with errorlevel %errorlevel%
    exit /b %errorlevel%
)

where cl.exe >nul 2>&1
if errorlevel 1 (
    echo [build_candle] cl.exe still not on PATH after vcvars64
    exit /b 1
)

REM Auto-detect the installed MSVC toolset (pick the newest if multiple).
set "MSVC_ROOT=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC"
set "MSVC_DIR="
for /f "delims=" %%D in ('dir /b /ad "%MSVC_ROOT%" 2^>nul') do set "MSVC_DIR=%MSVC_ROOT%\%%D"
if not defined MSVC_DIR (
    echo [build_candle] No MSVC toolset found under %MSVC_ROOT%
    exit /b 1
)
set "NVCC_CCBIN=%MSVC_DIR%\bin\Hostx64\x64\cl.exe"
if not exist "%NVCC_CCBIN%" (
    echo [build_candle] NVCC_CCBIN does not exist: %NVCC_CCBIN%
    exit /b 1
)

echo [build_candle] NVCC_CCBIN=%NVCC_CCBIN%

cargo build --release -p tempo-x402-paper --bin paper-bench
exit /b %errorlevel%
