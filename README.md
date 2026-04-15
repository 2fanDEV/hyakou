# hyakou

Hyakou is a Rust renderer with a wasm bridge and a TanStack frontend.

## Project layout

- `crates/core` - shared renderer-facing types and events
- `crates/hyako` - main native and wasm-capable renderer crate
- `crates/wasm_bindings` - `wasm-bindgen` wrapper exported to the frontend
- `frontend` - TanStack Start app that loads the generated wasm package
- `Hyakou` - local notes vault, not part of the application source tree

## Requirements

- Rust stable toolchain
- `wasm-pack`
- Node.js 22+

Install `wasm-pack` with:

```bash
cargo install wasm-pack
```

## First run

```bash
cargo check --workspace
npm ci --prefix frontend
npm run dev --prefix frontend
```

The frontend dev and build scripts automatically regenerate `crates/wasm_bindings/pkg` before they start.

## Common commands

```bash
# Rust workspace
cargo check --workspace
cargo test -p hyako --all-targets --all-features -- --nocapture

# Frontend
npm run dev --prefix frontend
npm run build --prefix frontend
npm run test --prefix frontend
```
