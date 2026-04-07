#!/bin/bash
set -e

echo "Building WASM..."

if ! command -v wasm-pack >/dev/null 2>&1; then
    echo "wasm-pack not found."
    echo "Install it first:"
    echo "  cargo install wasm-pack"
    exit 1
fi

wasm-pack build --target web --out-dir examples/pkg --features wasm

echo "✓ Build complete!"
echo ""
echo "Next steps:"
echo "  cd examples && pnpm dev"
echo "  cd examples && pnpm build"
