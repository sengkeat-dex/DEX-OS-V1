# DEX-OS Web UI

This directory hosts the WASM-driven web interface scaffolding for the DEX-OS core.

## Prerequisites

- Node.js 18+
- npm 9+ (or pnpm/yarn if you prefer)
- `wasm-pack` (install with `cargo install wasm-pack --locked`)

## Quick Start

```bash
# from the repository root
cd dex-ui
npm install
npm run wasm:build  # compiles ../dex-wasm into src/wasm-pkg
npm run dev         # launches Vite on http://localhost:5173
```

To produce an optimized build:

```bash
npm run build
npm run preview
```

## Project Layout

- `src/wasmBridge.ts` lazy-loads the wasm-bindgen bundle and re-exports typed helpers.
- `src/hooks/useDex.ts` wires the `WasmOrderBook` and `WasmAMM` bindings into React state.
- `src/App.tsx` renders a minimal dashboard that can execute a demo trade.
- `src/wasm-pkg/` is populated by `npm run wasm:build`; the directory is git-ignored.
- `public/index.html` bootstraps the Vite client and serves static assets.

## Regenerating the WASM Bundle

`npm run wasm:build` calls:

```bash
wasm-pack build ../dex-wasm --target web --out-dir src/wasm-pkg --release
```

Run this after changes inside `dex-wasm/`. Remove the `src/wasm-pkg` directory if you want to force a rebuild.

## Next Steps

- Expand `App.tsx` with real order entry forms, market data panels, and wallet connections.
- Integrate REST calls against `dex-api` (e.g., using `fetch` or a thin API client).
- Wrap the WASM demo in error boundaries or feature flags before shipping to production.
