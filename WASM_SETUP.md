# WebAssembly Build Setup

This document describes how to build and run Hyakou in a web browser using WebAssembly.

## Prerequisites

1. **Rust toolchain** with the WASM target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. **wasm-pack** for building WASM packages:
   ```bash
   cargo install wasm-pack
   ```

## Building for WASM

### Quick Build

Use the provided build script:

```bash
./build-wasm.sh
```

### Manual Build

```bash
wasm-pack build --target web --out-dir pkg
```

This will create a `pkg/` directory containing:
- `hyako_bg.wasm` - The compiled WebAssembly binary
- `hyako.js` - JavaScript bindings
- `hyako.d.ts` - TypeScript type definitions
- `package.json` - NPM package metadata

## Running Locally

After building, start a local HTTP server to test:

```bash
python3 -m http.server 8080
```

Then open http://localhost:8080 in your browser.

## Web Browser Requirements

### WebGPU Support
For the best experience, use a browser with WebGPU support:
- Chrome 113+ (enabled by default)
- Edge 113+
- Firefox Nightly (enable `dom.webgpu.enabled` in about:config)

### WebGL2 Fallback
If WebGPU is not available, the renderer will fall back to WebGL2:
- Most modern browsers support WebGL2
- Check compatibility at https://caniuse.com/webgl2

## Configuration

### WASM-Specific Dependencies

The following dependencies are added for WASM builds (see `Cargo.toml`):

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["Document", "Window", "Element", "HtmlCanvasElement"] }
console_error_panic_hook = "0.1"
console_log = "1.0"
```

### Backend Selection

The renderer automatically selects the appropriate backend:
- **Native (macOS)**: Metal
- **Native (Linux/Windows)**: Vulkan/DX12/Metal (auto-selected)
- **WASM**: WebGPU or WebGL2 (fallback)

## CI/CD Integration

The WASM build is verified in CI through the `.github/workflows/wasm-build.yml` workflow.

This workflow:
1. Builds the WASM target
2. Verifies the output
3. Uploads build artifacts
4. Runs WASM tests (when available)

## Known Limitations

1. **File System Access**: WASM builds cannot access the local file system directly. Assets must be:
   - Embedded at compile time, or
   - Loaded via HTTP requests

2. **Performance**: Initial WASM builds may be slower than native. Use release builds with optimizations.

3. **Browser Compatibility**: Some browsers may have limited WebGPU support. Always test across different browsers.

## Optimization

For production builds, the following optimizations are enabled in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
```

To further reduce WASM binary size:

```bash
wasm-pack build --target web --release -- --features wasm-opt
```

## Troubleshooting

### "Module not found" errors
Ensure you're serving the page over HTTP, not opening the HTML file directly (file://).

### WebGPU initialization fails
1. Check browser console for errors
2. Verify WebGPU support in your browser
3. The app should automatically fall back to WebGL2

### Blank screen
1. Open browser DevTools (F12)
2. Check the Console for JavaScript errors
3. Verify the WASM file was loaded successfully in the Network tab

## Resources

- [wasm-pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [wgpu Web Examples](https://github.com/gfx-rs/wgpu/tree/trunk/examples)
- [WebGPU Specification](https://www.w3.org/TR/webgpu/)
