@echo off
REM Rust build script with automatic Cargo check (Windows)

setlocal enabledelayedexpansion

REM Get script directory and project root
set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..
set RUST_DIR=%PROJECT_ROOT%rust

cd /d "%PROJECT_ROOT%"
echo Project root: %CD%
echo Rust directory: %RUST_DIR%

REM Check if rust directory exists
if not exist "%RUST_DIR%" (
    echo Error: Rust directory not found at %RUST_DIR%
    exit /b 1
)

cd /d "%RUST_DIR%"

REM Check Cargo
echo Checking Cargo...
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Error: Cargo not found
    echo.
    echo Please install Rust and Cargo:
    echo   Download from: https://rustup.rs/
    echo.
    echo Or use winget:
    echo   winget install Rustlang.Rustup
    echo.
    echo Or use Chocolatey:
    echo   choco install rust
    exit /b 1
)

for /f "tokens=2" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo Cargo version: %CARGO_VERSION%

for /f "tokens=2" %%i in ('rustc --version') do set RUST_VERSION=%%i
echo Rust version: %RUST_VERSION%

echo Cleaning previous build ^(if needed^)...
cargo clean 2>nul || echo Note: cargo clean skipped

echo Building Rust project (release mode)...
cargo build --release --features render

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ==========================================
    echo Build successful!
    echo ==========================================
    echo.
    echo Executable location: rust\target\release\map_generation.exe
    echo.
    echo Usage examples:
    echo   cd rust
    echo   cargo run --release --features render -- --seed 12345
    echo   cargo run --release --features render -- -r 0.08 --output examples\my_map
    echo   cargo run --release --features render -- --cities 10 --towns 30
    echo.
    echo Or run directly:
    echo   .\rust\target\release\map_generation.exe --seed 12345
    echo.
    echo Output files: rust\examples\output.json and rust\examples\output.png
    echo.
    echo To view the map:
    echo   Open rust\examples\index.html in a modern browser
    echo   The viewer will load output.json automatically
) else (
    echo.
    echo Build failed!
    exit /b 1
)

cd ..
