# C++ Build Guide

## Quick Start

**One command to build:**

```bash
# Linux/Mac
./scripts/build_cpp.sh

# Windows
.\scripts\build_cpp.bat
```

The script will automatically clean old builds, check for CMake and compiler, then build the project from scratch.

## Requirements

The build script will check for:
- CMake 3.10+
- C++17 compiler (GCC/Clang/MSVC)

If missing, the script will show installation instructions.

## Manual Installation (if needed)

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install cmake build-essential
```

**macOS:**
```bash
brew install cmake
xcode-select --install
```

**Windows:**
- CMake: https://cmake.org/download/ or `winget install Kitware.CMake`
- Visual Studio 2017+: https://visualstudio.microsoft.com/downloads/

## Generate Map

**Linux/Mac:**
```bash
# Default settings
./build/map_generation -o output.json

# Custom parameters
./build/map_generation --seed 12345 -r 0.08 -o map.json

# High-resolution
./build/map_generation -r 0.05 --size 2560:1440 -o hires.json

# More cities/towns
./build/map_generation --cities 10 --towns 30 -o populated.json

# Minimal map
./build/map_generation --no-labels --no-borders -o minimal.json
```

**Windows:**
```cmd
REM Default settings
.\build\map_generation.exe -o output.json

REM Custom parameters
.\build\map_generation.exe --seed 12345 -r 0.08 -o map.json

REM High-resolution
.\build\map_generation.exe -r 0.05 --size 2560:1440 -o hires.json
```

**Note:** The C++ version outputs JSON data only (no PNG rendering). Use the Rust version for PNG output, or implement your own renderer using the JSON data.

## View Map

The C++ version outputs JSON data only. To visualize the map:

**Option 1: Use Rust version's viewer**
```bash
# Copy JSON to Rust examples directory
cp output.json rust/examples/

# Open rust/examples/index.html in a modern browser
# The viewer supports WebGPU/WebGL/Canvas/SVG rendering
```

**Option 2: Implement your own renderer**
The JSON file contains all drawing primitives (lines, paths, circles, labels) with normalized coordinates (0-1 range). See the Rust implementation for reference.

## Command Options

```
-h, --help                     display this help and exit
-s, --seed=<uint>              set random generator seed
--timeseed                     set seed from system time
-r, --resolution=<float>       level of map detail
-o, --output=filename          output file
-e, --erosion-amount=<float>   erosion amount
--erosion-steps=<int>          number of erosion iterations
-c, --cities=<int>             number of generated cities
-t, --towns=<int>              number of generated towns
--size=<widthpx:heightpx>      set output image size
--draw-scale=<float>           set scale of drawn lines/points
--no-slopes                    disable slope drawing
--no-rivers                    disable river drawing
--no-contour                   disable contour drawing
--no-borders                   disable border drawing
--no-cities                    disable city drawing
--no-towns                     disable town drawing
--no-labels                    disable label drawing
--no-arealabels                disable area label drawing
-v, --verbose                  output additional information
```

## Troubleshooting

**Build fails**
- Check CMake version: `cmake --version` (need 3.10+)
- Check compiler supports C++17
- Clean build: `rm -rf build` and rebuild

**JSON file is empty or invalid**
- Check console output for errors
- Verify all required parameters are valid
- Try with default settings first
