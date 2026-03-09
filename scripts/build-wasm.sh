#!/usr/bin/env bash
# build-wasm.sh — Build the renderer-wasm crate with wasm-pack and copy the
# output into the web app's public directory so the browser can load it at
# runtime without any bundler involvement.
#
# Usage:
#   ./scripts/build-wasm.sh            # release build (default)
#   ./scripts/build-wasm.sh --dev      # debug / unoptimised build
#
# Prerequisites:
#   * Rust toolchain  (https://rustup.rs)
#   * wasm-pack       (cargo install wasm-pack  OR  https://rustwasm.github.io/wasm-pack/installer/)
#
# Output:
#   rust/apps/web/public/wasm/
#     fantasy_map_renderer_wasm.js        – wasm-bindgen ES-module glue
#     fantasy_map_renderer_wasm_bg.wasm   – compiled WebAssembly binary
#     fantasy_map_renderer_wasm.d.ts      – TypeScript type declarations
#     package.json / README.md            – wasm-pack metadata (safe to ignore)

set -euo pipefail

# ── Paths ──────────────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_CRATE="$REPO_ROOT/rust/crates/renderer-wasm"
OUT_DIR="$REPO_ROOT/rust/apps/web/public/wasm"

# ── Parse arguments ────────────────────────────────────────────────────────
BUILD_MODE="release"
WASM_PACK_FLAGS=""
for arg in "$@"; do
    case "$arg" in
        --dev)
            BUILD_MODE="dev"
            WASM_PACK_FLAGS="--dev"
            ;;
        --help|-h)
            sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
    esac
done

echo "==> Building WASM (${BUILD_MODE}) …"

# ── Ensure wasm-pack is available ──────────────────────────────────────────
if ! command -v wasm-pack &>/dev/null; then
    echo "    wasm-pack not found – installing via cargo …"
    cargo install wasm-pack
fi

# ── Build ──────────────────────────────────────────────────────────────────
# --target web   → ES-module output that works in any modern browser via
#                  dynamic import('/wasm/…').
# --features wasm → activates the wasm-bindgen feature gate in the crate.
# --out-dir       → destination for the generated JS/WASM files.
wasm-pack build "$WASM_CRATE" \
    --target web \
    --features wasm \
    --out-dir "$OUT_DIR" \
    $WASM_PACK_FLAGS

# ── Remove files that aren't needed at runtime ─────────────────────────────
rm -f "$OUT_DIR/.gitignore"

echo ""
echo "==> WASM build complete!"
echo "    Output: $OUT_DIR"
echo ""
echo "    Files:"
ls -lh "$OUT_DIR"
echo ""
echo "    Next steps:"
echo "      cd rust/apps/web && npm run dev    # start dev server with WASM"
echo "      cd rust/apps/web && npm run build  # production build with WASM"
