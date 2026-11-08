#!/usr/bin/env bash
set -euo pipefail

echo "[1/3] Compiling Rust voxel engine (src/lib.rs) -> web/minecraft_rust.wasm..."
RUSTC_BOOTSTRAP=1 rustc --target wasm32-unknown-unknown --crate-type cdylib -O src/lib.rs -o web/minecraft_rust.wasm

echo "[2/3] Embedding compiled WebAssembly binary into web/index.html..."
node -e '
const fs = require("fs");
const wasmB64 = fs.readFileSync("web/minecraft_rust.wasm").toString("base64");
const tpl = fs.readFileSync("web/template.html", "utf8");
const out = tpl.replace("__WASM_BASE64_PLACEHOLDER__", wasmB64);
fs.writeFileSync("web/index.html", out);
console.log("Generated web/index.html (" + (out.length / 1024).toFixed(1) + " KB) with embedded Rust WASM (" + (wasmB64.length / 1024).toFixed(1) + " KB base64)");
'

echo "[3/3] Running native Rust unit tests..."
cargo test
