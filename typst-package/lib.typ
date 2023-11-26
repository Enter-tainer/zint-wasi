#let zint-wasm = plugin("./zint_typst_plugin.wasm")
#let ean(data, ..args) = image.decode(str(zint-wasm.ean(bytes(data))), ..args)

