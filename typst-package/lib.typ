#let zint-wasm = plugin("./zint_typst_plugin.wasm")

#let ean(data, ..args) = image.decode(str(zint-wasm.ean_gen(bytes(data))), ..args)

#let code128(data, ..args) = image.decode(str(zint-wasm.code128_gen(bytes(data))), ..args)

#let code39(data, ..args) = image.decode(str(zint-wasm.code39_gen(bytes(data))), ..args)

#let upca(data, ..args) = image.decode(str(zint-wasm.upca_gen(bytes(data))), ..args)

#let data-matrix(data, ..args) = image.decode(str(zint-wasm.data_matrix_gen(bytes(data))), ..args)

#let qrcode(data, ..args) = image.decode(str(zint-wasm.qrcode_gen(bytes(data))), ..args)

#let channel(data, ..args) = image.decode(str(zint-wasm.channel_gen(bytes(data))), ..args)

#let msi-plessey(data, ..args) = image.decode(str(zint-wasm.msi_plessey_gen(bytes(data))), ..args)

#let micro-pdf417(data, ..args) = image.decode(str(zint-wasm.micro_pdf417_gen(bytes(data))), ..args)

#let aztec(data, ..args) = image.decode(str(zint-wasm.aztec_gen(bytes(data))), ..args)

#let code16k(data, ..args) = image.decode(str(zint-wasm.code16k_gen(bytes(data))), ..args)

#let maxicode(data, ..args) = image.decode(str(zint-wasm.maxicode_gen(bytes(data))), ..args)

#let planet(data, ..args) = image.decode(str(zint-wasm.planet_gen(bytes(data))), ..args)

#let barcode(data, type, ..args) = image.decode(
  str(
    zint-wasm.gen_with_options(
      cbor.encode(
        (symbology: (type: type),)
      ), bytes(data)
    )
  ),
..args)
