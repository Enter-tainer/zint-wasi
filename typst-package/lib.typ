#let zint-wasm = plugin("./zint_typst_plugin.wasm")

/// Draw a barcode of any supported `symbology`.
///
/// - data (str): Data to encode.
/// - symbology (str): Symbology type name; must be one of #l(<symbology>)[supported types].
///
///     Example values: #typst-val("\"Code11\""), #typst-val("\"C25Standard\""), ...
/// - options (dictionary): Additional options to pass to Zint.
///
///     See the #l(<options>)[configuration section] for details on available options and how to use them.
/// - fg-color (color): Foreground color for barcode elements.
/// - bg-color (color, none): Background color. Set to `none` for transparent.
/// - ..args (any): Any additional arguments to forward to the container.
/// -> content
#let barcode(data, symbology, options: (:), fg-color: black, bg-color: none, ..args) = {
  let data = data
  if type(data) == str {
    data = bytes(data)
  } else if type(data) == array {
    data = bytes(data)
  }
  
  let zint-options = options
  let ultracode-colors = zint-options.remove("ultracode-colors", default: (
    "1": cmyk(100%, 0%, 0%, 0%),    // Cyan
    "2": cmyk(100%, 100%, 0%, 0%),  // Blue
    "3": cmyk(0%, 100%, 0%, 0%),    // Magenta
    "4": cmyk(0%, 100%, 100%, 0%),  // Red
    "5": cmyk(0%, 0%, 100%, 0%),    // Yellow
    "6": cmyk(100%, 0%, 100%, 0%),  // Green
    "7": cmyk(0%, 0%, 0%, 100%),    // Black
    "8": cmyk(0%, 0%, 0%, 0%),      // White
  ))
  ultracode-colors.insert("-1", fg-color)
  ultracode-colors.insert("0", bg-color)

  // Extract typst-side options before passing to zint
  let color(code) = {
    let key = str(code)
    ultracode-colors.at(key, default: fg-color)
  }
  let result = cbor(zint-wasm.zint_encode(
    cbor.encode((symbology: symbology, ..zint-options)),
    data,
  ))

  // Unwrap Result envelope
  let plot = if "Ok" in result {
    result.Ok
  } else if "Err" in result {
    panic(result.Err.message)
  } else {
    panic("unexpected zint response: " + repr(result))
  }

  let w = plot.width
  let h = plot.height
  let geo = plot.geometry

  box(width: w * 1pt, height: h * 1pt, fill: bg-color, ..args.named())[
    // Rectangles
    #for r in geo.at("rectangles", default: ()) {
      place(dx: r.x * 1pt, dy: r.y * 1pt,
        rect(width: r.width * 1pt, height: r.height * 1pt, fill: color(r.color)))
    }
    // Circles
    #for c in geo.at("circles", default: ()) {
      let fill = color(c.color)
      place(dx: (c.x - c.diameter / 2) * 1pt, dy: (c.y - c.diameter / 2) * 1pt,
        circle(radius: c.diameter / 2 * 1pt,
          fill: if c.width == 0.0 { fill } else { none },
          stroke: if c.width > 0.0 { fill + c.width * 1pt } else { none }))
    }
    // Hexagons (MaxiCode)
    #for hex in geo.at("hexagons", default: ()) {
      let r = hex.diameter / 2
      place(dx: hex.x * 1pt, dy: hex.y * 1pt,
        path(fill: color(hex.color), closed: true,
          (r * calc.cos(0deg) * 1pt, r * calc.sin(0deg) * 1pt),
          (r * calc.cos(60deg) * 1pt, r * calc.sin(60deg) * 1pt),
          (r * calc.cos(120deg) * 1pt, r * calc.sin(120deg) * 1pt),
          (r * calc.cos(180deg) * 1pt, r * calc.sin(180deg) * 1pt),
          (r * calc.cos(240deg) * 1pt, r * calc.sin(240deg) * 1pt),
          (r * calc.cos(300deg) * 1pt, r * calc.sin(300deg) * 1pt),
        ))
    }
    // Text (HRT) — zint returns (x, y) as (center, baseline)
    #for s in geo.at("strings", default: ()) {
      let tw = s.width * 1pt
      let halign = if s.horizontal_align == "Left" { left }
        else if s.horizontal_align == "Right" { right }
        else { center }
      place(dx: s.x * 1pt - tw / 2, dy: (s.y - s.font_size) * 1pt,
        box(width: tw,
          align(halign,
            text(
              size: s.font_size * 1pt,
              top-edge: "ascender",
              bottom-edge: "descender",
              fill: color(s.color),
              s.text
            )
          )
        )
      )
    }
  ]
}

/// Returns #typst-type("int") option value for given Data Matrix _width_ and _height_.
///
/// Zint allows square and rectangular values to be enforced with `DM_SQUARE` and `DM_DMRE` #l(<opt_3>, "Option 3") values.
///
/// - width (int): Data Matrix width
/// - height (int): Data Matrix height
/// -> int
#let dm-size(height, width) = {
  // Copied from DM size table
  if height == 10 and width == 10 {
    return int(1)
  }
  if height == 12 and width == 12 {
    return int(2)
  }
  if height == 14 and width == 14 {
    return int(3)
  }
  if height == 16 and width == 16 {
    return int(4)
  }
  if height == 18 and width == 18 {
    return int(5)
  }
  if height == 20 and width == 20 {
    return int(6)
  }
  if height == 22 and width == 22 {
    return int(7)
  }
  if height == 24 and width == 24 {
    return int(8)
  }
  if height == 26 and width == 26 {
    return int(9)
  }
  if height == 32 and width == 32 {
    return int(10)
  }
  if height == 36 and width == 36 {
    return int(11)
  }
  if height == 40 and width == 40 {
    return int(12)
  }
  if height == 44 and width == 44 {
    return int(13)
  }
  if height == 48 and width == 48 {
    return int(14)
  }
  if height == 52 and width == 52 {
    return int(15)
  }
  if height == 64 and width == 64 {
    return int(16)
  }
  if height == 72 and width == 72 {
    return int(17)
  }
  if height == 80 and width == 80 {
    return int(18)
  }
  if height == 88 and width == 88 {
    return int(19)
  }
  if height == 96 and width == 96 {
    return int(20)
  }
  if height == 104 and width == 104 {
    return int(21)
  }
  if height == 120 and width == 120 {
    return int(22)
  }
  if height == 132 and width == 132 {
    return int(23)
  }
  if height == 144 and width == 144 {
    return int(24)
  }
  if height == 8 and width == 18 {
    return int(25)
  }
  if height == 8 and width == 32 {
    return int(26)
  }
  if height == 12 and width == 26 {
    return int(28)
  }
  if height == 12 and width == 36 {
    return int(28)
  }
  if height == 16 and width == 36 {
    return int(29)
  }
  if height == 16 and width == 48 {
    return int(30)
  }

  // Copied from DMRE table
  if height == 8 and width == 48 {
    return int(31)
  }
  if height == 8 and width == 64 {
    return int(32)
  }
  if height == 8 and width == 80 {
    return int(33)
  }
  if height == 8 and width == 96 {
    return int(34)
  }
  if height == 8 and width == 120 {
    return int(35)
  }
  if height == 8 and width == 144 {
    return int(36)
  }
  if height == 12 and width == 64 {
    return int(37)
  }
  if height == 12 and width == 88 {
    return int(38)
  }
  if height == 16 and width == 64 {
    return int(39)
  }
  if height == 20 and width == 36 {
    return int(40)
  }
  if height == 20 and width == 44 {
    return int(41)
  }
  if height == 20 and width == 64 {
    return int(42)
  }
  if height == 22 and width == 48 {
    return int(43)
  }
  if height == 24 and width == 48 {
    return int(44)
  }
  if height == 24 and width == 64 {
    return int(45)
  }
  if height == 26 and width == 40 {
    return int(46)
  }
  if height == 26 and width == 48 {
    return int(47)
  }
  if height == 26 and width == 64 {
    return int(48)
  }
  panic("Data Matrix with dimensions " + str(width) + "x" + str(height) + " not supported")
}

#let code11(data, options: (:), ..args) = barcode(
  data,
  "Code11",
  options: options,
  ..args,
)
#let c25-standard(data, options: (:), ..args) = barcode(
  data,
  "C25Standard",
  options: options,
  ..args,
)
#let c25-inter(data, options: (:), ..args) = barcode(
  data,
  "C25Inter",
  options: options,
  ..args,
)
#let c25-iata(data, options: (:), ..args) = barcode(
  data,
  "C25IATA",
  options: options,
  ..args,
)
#let c25-logic(data, options: (:), ..args) = barcode(
  data,
  "C25Logic",
  options: options,
  ..args,
)
#let c25-ind(data, options: (:), ..args) = barcode(
  data,
  "C25Ind",
  options: options,
  ..args,
)
#let code39(data, options: (:), ..args) = barcode(
  data,
  "Code39",
  options: options,
  ..args,
)
#let ex-code39(data, options: (:), ..args) = barcode(
  data,
  "ExCode39",
  options: options,
  ..args,
)
#let eanx(data, options: (:), ..args) = barcode(
  data,
  "EANX",
  options: options,
  ..args,
)
#let eanx-chk(data, options: (:), ..args) = barcode(
  data,
  "EANXChk",
  options: options,
  ..args,
)
#let ean(data, options: (:), ..args) = eanx-chk(data, options: options, ..args)
#let gs1-128(data, options: (:), ..args) = barcode(
  data,
  "GS1128",
  options: options,
  ..args,
)
#let codabar(data, options: (:), ..args) = barcode(
  data,
  "Codabar",
  options: options,
  ..args,
)
#let code128(data, options: (:), ..args) = barcode(
  data,
  "Code128",
  options: options,
  ..args,
)
#let dp-leitcode(data, options: (:), ..args) = barcode(
  data,
  "DPLEIT",
  options: options,
  ..args,
)
#let dp-ident(data, options: (:), ..args) = barcode(
  data,
  "DPIDENT",
  options: options,
  ..args,
)
#let code16k(data, options: (:), ..args) = barcode(
  data,
  "Code16k",
  options: options,
  ..args,
)
#let code49(data, options: (:), ..args) = barcode(
  data,
  "Code49",
  options: options,
  ..args,
)
#let code93(data, options: (:), ..args) = barcode(
  data,
  "Code93",
  options: options,
  ..args,
)
#let flat(data, options: (:), ..args) = barcode(
  data,
  "Flat",
  options: options,
  ..args,
)
#let dbar-omn(data, options: (:), ..args) = barcode(
  data,
  "DBarOmn",
  options: options,
  ..args,
)
#let dbar-ltd(data, options: (:), ..args) = barcode(
  data,
  "DBarLtd",
  options: options,
  ..args,
)
#let dbar-exp(data, options: (:), ..args) = barcode(
  data,
  "DBarExp",
  options: options,
  ..args,
)
#let telepen(data, options: (:), ..args) = barcode(
  data,
  "Telepen",
  options: options,
  ..args,
)
#let upca(data, options: (:), ..args) = barcode(
  data,
  "UPCA",
  options: options,
  ..args,
)
#let upca-chk(data, options: (:), ..args) = barcode(
  data,
  "UPCAChk",
  options: options,
  ..args,
)
#let upce(data, options: (:), ..args) = barcode(
  data,
  "UPCE",
  options: options,
  ..args,
)
#let upce-chk(data, options: (:), ..args) = barcode(
  data,
  "UPCEChk",
  options: options,
  ..args,
)
#let postnet(data, options: (:), ..args) = barcode(
  data,
  "Postnet",
  options: options,
  ..args,
)
#let msi-plessey(data, options: (:), ..args) = barcode(
  data,
  "MSIPlessey",
  options: options,
  ..args,
)
#let fim(data, options: (:), ..args) = barcode(
  data,
  "FIM",
  options: options,
  ..args,
)
#let logmars(data, options: (:), ..args) = barcode(
  data,
  "Logmars",
  options: options,
  ..args,
)
#let pharma(data, options: (:), ..args) = barcode(
  data,
  "Pharma",
  options: options,
  ..args,
)
#let pzn(data, options: (:), ..args) = barcode(
  data,
  "PZN",
  options: options,
  ..args,
)
#let pharma-two(data, options: (:), ..args) = barcode(
  data,
  "PharmaTwo",
  options: options,
  ..args,
)
#let cepnet(data, options: (:), ..args) = barcode(
  data,
  "CEPNet",
  options: options,
  ..args,
)
#let pdf417(data, options: (:), ..args) = barcode(
  data,
  "PDF417",
  options: options,
  ..args,
)
#let pdf417-comp(data, options: (:), ..args) = barcode(
  data,
  "PDF417Comp",
  options: options,
  ..args,
)
#let maxicode(data, options: (:), ..args) = barcode(
  data,
  "MaxiCode",
  options: options,
  ..args,
)
#let qrcode(data, options: (:), ..args) = barcode(
  data,
  "QRCode",
  options: options,
  ..args,
)
#let code128ab(data, options: (:), ..args) = barcode(
  data,
  "Code128AB",
  options: options,
  ..args,
)
#let aus-post(data, options: (:), ..args) = barcode(
  data,
  "AusPost",
  options: options,
  ..args,
)
#let aus-reply(data, options: (:), ..args) = barcode(
  data,
  "AusReply",
  options: options,
  ..args,
)
#let aus-route(data, options: (:), ..args) = barcode(
  data,
  "AusRoute",
  options: options,
  ..args,
)
#let aus-redirect(data, options: (:), ..args) = barcode(
  data,
  "AusRedirect",
  options: options,
  ..args,
)
#let isbnx(data, options: (:), ..args) = barcode(
  data,
  "ISBNX",
  options: options,
  ..args,
)
#let rm4scc(data, options: (:), ..args) = barcode(
  data,
  "RM4SCC",
  options: options,
  ..args,
)
#let data-matrix(data, options: (:), ..args) = barcode(
  data,
  "DataMatrix",
  options: options,
  ..args,
)
#let ean14(data, options: (:), ..args) = barcode(
  data,
  "EAN14",
  options: options,
  ..args,
)
#let vin(data, options: (:), ..args) = barcode(
  data,
  "VIN",
  options: options,
  ..args,
)
#let codablock-f(data, options: (:), ..args) = barcode(
  data,
  "CodablockF",
  options: options,
  ..args,
)
#let nve18(data, options: (:), ..args) = barcode(
  data,
  "NVE18",
  options: options,
  ..args,
)
#let japan-post(data, options: (:), ..args) = barcode(
  data,
  "JapanPost",
  options: options,
  ..args,
)
#let korea-post(data, options: (:), ..args) = barcode(
  data,
  "KoreaPost",
  options: options,
  ..args,
)
#let dbar-stk(data, options: (:), ..args) = barcode(
  data,
  "DBarStk",
  options: options,
  ..args,
)
#let dbar-omn-stk(data, options: (:), ..args) = barcode(
  data,
  "DBarOmnStk",
  options: options,
  ..args,
)
#let dbar-exp-stk(data, options: (:), ..args) = barcode(
  data,
  "DBarExpStk",
  options: options,
  ..args,
)
#let planet(data, options: (:), ..args) = barcode(
  data,
  "Planet",
  options: options,
  ..args,
)
#let micro-pdf417(data, options: (:), ..args) = barcode(
  data,
  "MicroPDF417",
  options: options,
  ..args,
)
#let usps-imail(data, options: (:), ..args) = barcode(
  data,
  "USPSIMail",
  options: options,
  ..args,
)
#let plessey(data, options: (:), ..args) = barcode(
  data,
  "Plessey",
  options: options,
  ..args,
)
#let telepen-num(data, options: (:), ..args) = barcode(
  data,
  "TelepenNum",
  options: options,
  ..args,
)
#let itf14(data, options: (:), ..args) = barcode(
  data,
  "ITF14",
  options: options,
  ..args,
)
#let kix(data, options: (:), ..args) = barcode(
  data,
  "KIX",
  options: options,
  ..args,
)
#let aztec(data, options: (:), ..args) = barcode(
  data,
  "Aztec",
  options: options,
  ..args,
)
#let daft(data, options: (:), ..args) = barcode(
  data,
  "DAFT",
  options: options,
  ..args,
)
#let dpd(data, options: (:), ..args) = barcode(
  data,
  "DPD",
  options: options,
  ..args,
)
#let micro-qr(data, options: (:), ..args) = barcode(
  data,
  "MicroQR",
  options: options,
  ..args,
)
#let hibc-128(data, options: (:), ..args) = barcode(
  data,
  "HIBC128",
  options: options,
  ..args,
)
#let hibc-39(data, options: (:), ..args) = barcode(
  data,
  "HIBC39",
  options: options,
  ..args,
)
#let hibc-dm(data, options: (:), ..args) = barcode(
  data,
  "HIBCDM",
  options: options,
  ..args,
)
#let hibc-qr(data, options: (:), ..args) = barcode(
  data,
  "HIBCQR",
  options: options,
  ..args,
)
#let hibc-pdf(data, options: (:), ..args) = barcode(
  data,
  "HIBCPDF",
  options: options,
  ..args,
)
#let hibc-mic-pdf(data, options: (:), ..args) = barcode(
  data,
  "HIBCMicPDF",
  options: options,
  ..args,
)
#let hibc-codablock-f(data, options: (:), ..args) = barcode(
  data,
  "HIBCCodablockF",
  options: options,
  ..args,
)
#let hibc-aztec(data, options: (:), ..args) = barcode(
  data,
  "HIBCAztec",
  options: options,
  ..args,
)
#let dotcode(data, options: (:), ..args) = barcode(
  data,
  "DotCode",
  options: options,
  ..args,
)
#let hanxin(data, options: (:), ..args) = barcode(
  data,
  "HanXin",
  options: options,
  ..args,
)
#let upus10(data, options: (:), ..args) = barcode(
  data,
  "UPUS10",
  options: options,
  ..args,
)
#let mailmark-4s(data, options: (:), ..args) = barcode(
  data,
  "Mailmark4S",
  options: options,
  ..args,
)
#let azrune(data, options: (:), ..args) = barcode(
  data,
  "AzRune",
  options: options,
  ..args,
)
#let code32(data, options: (:), ..args) = barcode(
  data,
  "Code32",
  options: options,
  ..args,
)
#let channel(data, options: (:), ..args) = barcode(
  data,
  "Channel",
  options: options,
  ..args,
)
#let code-one(data, options: (:), ..args) = barcode(
  data,
  "CodeOne",
  options: options,
  ..args,
)
#let grid-matrix(data, options: (:), ..args) = barcode(
  data,
  "GridMatrix",
  options: options,
  ..args,
)
#let upnqr(data, options: (:), ..args) = barcode(
  data,
  "UPNQR",
  options: options,
  ..args,
)
#let ultra(data, options: (:), ..args) = barcode(
  data,
  "Ultra",
  options: options,
  ..args,
)
#let rmqr(data, options: (:), ..args) = barcode(
  data,
  "RMQR",
  options: options,
  ..args,
)
#let bc412(data, options: (:), ..args) = barcode(
  data,
  "BC412",
  options: options,
  ..args,
)

#let mailmark-2d(height, width, data, options: (:), ..args) = barcode(
  data,
  "Mailmark2D",
  options: (
    option_2: dm-size(height, width),
    ..options,
  ),
  ..args,
)

#let barcode-primary(primary, data, type, options: (:), ..args) = barcode(
  data,
  type,
  options: (
    primary: primary,
    ..options,
  ),
  ..args,
)

#let barcode-composite(
  primary,
  data,
  mode,
  type,
  options: (:),
  ..args,
) = barcode-primary(
  primary,
  data,
  type,
  options: (
    option_1: int(mode),
    ..options,
  ),
  ..args,
)
