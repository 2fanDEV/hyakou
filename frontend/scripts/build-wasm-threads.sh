#!/usr/bin/env bash
set -euo pipefail

profile="${1:-dev}"

export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-nightly}"
export RUSTFLAGS="${RUSTFLAGS:--C target-feature=+atomics,+bulk-memory -Clink-arg=--shared-memory -Clink-arg=--max-memory=1073741824 -Clink-arg=--import-memory -Clink-arg=--export=__wasm_init_tls -Clink-arg=--export=__tls_size -Clink-arg=--export=__tls_align -Clink-arg=--export=__tls_base}"

args=(build ../crates/wasm_bindings --target web --out-dir pkg)
if [[ "${profile}" == "release" ]]; then
	args+=(--release)
fi

wasm-pack "${args[@]}" -- -Z build-std=panic_abort,std
