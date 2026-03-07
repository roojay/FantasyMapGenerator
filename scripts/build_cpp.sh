#!/bin/bash

# Native build script with automatic CMake check

set -e

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Project root: $PROJECT_ROOT"
cd "$PROJECT_ROOT"

# Check CMake
echo "Checking CMake..."
if ! command -v cmake &> /dev/null; then
    echo "Error: CMake not found"
    echo ""
    echo "Please install CMake 3.10 or higher:"
    echo ""
    echo "Ubuntu/Debian:"
    echo "  sudo apt-get update"
    echo "  sudo apt-get install cmake build-essential"
    echo ""
    echo "macOS (with Homebrew):"
    echo "  brew install cmake"
    echo ""
    echo "Or download from: https://cmake.org/download/"
    exit 1
fi

CMAKE_VERSION=$(cmake --version | head -n 1 | grep -oP '\d+\.\d+\.\d+')
echo "CMake version: $CMAKE_VERSION"

# Check compiler
if ! command -v g++ &> /dev/null && ! command -v clang++ &> /dev/null; then
    echo "Error: No C++ compiler found"
    echo ""
    echo "Please install a C++17 compatible compiler:"
    echo ""
    echo "Ubuntu/Debian:"
    echo "  sudo apt-get install build-essential"
    echo ""
    echo "macOS:"
    echo "  xcode-select --install"
    exit 1
fi

echo "Preparing build directory..."
if [ -d "build" ]; then
    echo "Build directory exists, attempting to clean..."
    # Try to remove, but continue if it fails
    rm -rf build 2>/dev/null || {
        echo "Warning: Could not remove build directory (may be in use)"
        echo "Attempting to clean build contents instead..."
        rm -rf build/* 2>/dev/null || true
    }
fi

if [ ! -d "build" ]; then
    echo "Creating build directory..."
    mkdir -p build
fi

cd build

echo "Configuring with CMake..."
cmake ..

echo "Building..."
cmake --build . --config Release

echo ""
echo "=========================================="
echo "Build successful!"
echo "=========================================="
echo ""
echo "Executable location: build/map_generation"
echo ""
echo "Usage examples:"
echo "  ./build/map_generation -o output.json"
echo "  ./build/map_generation --seed 12345 -r 0.08 -o map.json"
echo "  ./build/map_generation --cities 10 --towns 30 -o populated.json"
echo ""
echo "Output: JSON file with map data (no PNG rendering in C++ version)"
echo ""
echo "Note: The C++ version outputs JSON only."
echo "      Use the Rust version for PNG rendering, or implement your own renderer."
