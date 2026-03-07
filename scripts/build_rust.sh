#!/bin/bash

# Rust build script with automatic Cargo check

set -e

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RUST_DIR="$PROJECT_ROOT/rust"

echo "Project root: $PROJECT_ROOT"
echo "Rust directory: $RUST_DIR"

# Check if rust directory exists
if [ ! -d "$RUST_DIR" ]; then
    echo "Error: Rust directory not found at $RUST_DIR"
    exit 1
fi

cd "$RUST_DIR"

# Check Cargo
echo "Checking Cargo..."
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found"
    echo ""
    echo "Please install Rust and Cargo:"
    echo ""
    echo "Linux/macOS:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "Or visit: https://rustup.rs/"
    exit 1
fi

CARGO_VERSION=$(cargo --version | grep -oP '\d+\.\d+\.\d+')
echo "Cargo version: $CARGO_VERSION"

# Check Rust version
RUST_VERSION=$(rustc --version | grep -oP '\d+\.\d+\.\d+')
echo "Rust version: $RUST_VERSION"

echo "Cleaning previous build (if needed)..."
cargo clean 2>/dev/null || echo "Note: cargo clean skipped"

echo "Building Rust project (release mode)..."
cargo build --release --features render

if [ $? -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "Build successful!"
    echo "=========================================="
    echo ""
    echo "Executable location: rust/target/release/map_generation"
    echo ""
    echo "Usage examples:"
    echo "  cd rust"
    echo "  cargo run --release --features render -- --seed 12345"
    echo "  cargo run --release --features render -- -r 0.08 --output examples/my_map"
    echo "  cargo run --release --features render -- --cities 10 --towns 30"
    echo ""
    echo "Or run directly:"
    echo "  ./rust/target/release/map_generation --seed 12345"
    echo ""
    echo "Output files: rust/examples/output.json and rust/examples/output.png"
    echo ""
    echo "To view the map:"
    echo "  Open rust/examples/index.html in a modern browser"
    echo "  The viewer will load output.json automatically"
else
    echo ""
    echo "Build failed!"
    exit 1
fi
