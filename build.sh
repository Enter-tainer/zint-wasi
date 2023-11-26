#!/bin/bash
set -euxo pipefail
cargo build --release --target wasm32-wasi
# cargo about generate about.hbs > license.html
# cp license.html typst-package/
wasi-stub -r 0 ./target/wasm32-wasi/release/zint_typst_plugin.wasm -o typst-package/zint_typst_plugin.wasm
wasm-opt typst-package/zint_typst_plugin.wasm -O3 --enable-bulk-memory -o typst-package/zint_typst_plugin.wasm
