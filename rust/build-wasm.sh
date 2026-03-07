#!/bin/bash
set -e

echo "Building WASM..."

if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

wasm-pack build --target web --out-dir examples/pkg --features wasm

echo "✓ Build complete!"
echo ""
echo "To run:"
echo "  cd examples && npx serve"
