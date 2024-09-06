#!/bin/bash
set -euxo pipefail
cargo build --release --target wasm32-wasip1
cargo about generate about.hbs > typst-package/license.html
wasi-stub -r 0 ./target/wasm32-wasip1/release/zint_typst_plugin.wasm -o typst-package/zint_typst_plugin.wasm
wasm-opt typst-package/zint_typst_plugin.wasm -O3 --enable-bulk-memory -o typst-package/zint_typst_plugin.wasm
cp LICENSE typst-package/LICENSE
typst compile typst-package/manual.typ typst-package/manual.pdf
typst compile typst-package/example.typ typst-package/example.svg
# We're only showing the first page
# rm assets/*.svg
# typst compile typst-package/manual.typ 'assets/doc-{n}.svg'
