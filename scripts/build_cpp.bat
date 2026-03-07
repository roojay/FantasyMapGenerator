@echo off
REM Native build script with automatic CMake check (Windows)

setlocal enabledelayedexpansion

REM Get script directory and project root
set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..
cd /d "%PROJECT_ROOT%"

echo Project root: %CD%

REM Check CMake
echo Checking CMake...
where cmake >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Error: CMake not found
    echo.
    echo Please install CMake 3.10 or higher:
    echo   Download from: https://cmake.org/download/
    echo.
    echo Or use winget:
    echo   winget install Kitware.CMake
    echo.
    echo Or use Chocolatey:
    echo   choco install cmake
    exit /b 1
)

for /f "tokens=3" %%i in ('cmake --version ^| findstr /C:"cmake version"') do set CMAKE_VERSION=%%i
echo CMake version: %CMAKE_VERSION%

REM Check for Visual Studio or other compiler
where cl >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Warning: MSVC compiler not found in PATH
    echo CMake will try to detect available compilers...
    echo.
    echo If build fails, please install Visual Studio 2017 or later:
    echo   https://visualstudio.microsoft.com/downloads/
    echo.
)

echo Preparing build directory...
if exist build (
    echo Build directory exists, attempting to clean...
    rmdir /s /q build 2>nul
    if exist build (
        echo Warning: Could not remove build directory ^(may be in use^)
        echo Attempting to clean build contents instead...
        del /s /q build\* 2>nul
        for /d %%p in (build\*) do rmdir /s /q "%%p" 2>nul
    )
)

if not exist build (
    echo Creating build directory...
    mkdir build
)

cd build

echo Configuring with CMake...
cmake ..

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo CMake configuration failed!
    echo Please ensure you have a C++17 compatible compiler installed.
    exit /b 1
)

echo Building...
cmake --build . --config Release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ==========================================
    echo Build successful!
    echo ==========================================
    echo.
    echo Executable location: build\map_generation.exe
    echo.
    echo Usage examples:
    echo   .\build\map_generation.exe -o output.json
    echo   .\build\map_generation.exe --seed 12345 -r 0.08 -o map.json
    echo   .\build\map_generation.exe --cities 10 --towns 30 -o populated.json
    echo.
    echo Output: JSON file with map data ^(no PNG rendering in C++ version^)
    echo.
    echo Note: The C++ version outputs JSON only.
    echo       Use the Rust version for PNG rendering, or implement your own renderer.
) else (
    echo.
    echo Build failed!
    exit /b 1
)

cd ..
