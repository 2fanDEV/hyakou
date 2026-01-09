#!/bin/bash
set -e

echo "Building Hyakou for WebAssembly..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build the WASM package
wasm-pack build --target web --out-dir pkg

echo "WASM build complete! Output in ./pkg directory"
echo ""
echo "To test locally, run a simple HTTP server:"
echo "  python3 -m http.server 8080"
echo "Then open http://localhost:8080 in your browser"
