# frontend

This app hosts the web UI and loads the generated wasm bindings from `../crates/wasm_bindings/pkg`.

## Requirements

- Node.js 22+
- Rust stable
- `wasm-pack`

## Development

```bash
npm ci
npm run dev
```

`npm run dev`, `npm run build`, and `npm run test` regenerate the wasm package before running.

## Scripts

```bash
npm run wasm:build:dev
npm run wasm:build:prod
npm run dev
npm run build
npm run test
npm run lint
npm run format
npm run check
```
