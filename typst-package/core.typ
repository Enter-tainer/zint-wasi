#let zint-wasm = plugin("./zint_typst_plugin.wasm")

/// Draw a barcode of any supported `symbology`.
///
/// - data (str): Data to encode.
/// - symbology (str): Symbology type name.
/// - options (dictionary): Additional options to pass to Zint.
/// - foreground (dictionary): Foreground and text styling options.
/// - background (dictionary): Background styling for the container box.
/// - ..args (any): Any additional arguments to forward to the container.
/// -> content
#let barcode(data, symbology, options: (:), foreground: (fill: black), background: (:), ..args) = {
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
  ultracode-colors.insert("-1", foreground.fill)
  ultracode-colors.insert("0", background.at("fill", default: none))

  let color(code) = {
    let key = str(code)
    ultracode-colors.at(key, default: foreground.fill)
  }
  let result = cbor(zint-wasm.zint_encode(
    cbor.encode((symbology: symbology, ..zint-options)),
    data,
  ))

  // Unwrap Result envelope
  let plot = if "Ok" in result {
    result.Ok
  } else if "Err" in result {
    panic(result.Err)
  } else {
    panic("unexpected zint response: " + repr(result))
  }

  let w = plot.width
  let h = plot.height
  let geo = plot.geometry

  box(width: w * 1pt, height: h * 1pt, ..background, ..args.named())[
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
        curve(fill: color(hex.color),
          curve.move((r * calc.cos(0deg) * 1pt, r * calc.sin(0deg) * 1pt)),
          curve.line((r * calc.cos(60deg) * 1pt, r * calc.sin(60deg) * 1pt)),
          curve.line((r * calc.cos(120deg) * 1pt, r * calc.sin(120deg) * 1pt)),
          curve.line((r * calc.cos(180deg) * 1pt, r * calc.sin(180deg) * 1pt)),
          curve.line((r * calc.cos(240deg) * 1pt, r * calc.sin(240deg) * 1pt)),
          curve.line((r * calc.cos(300deg) * 1pt, r * calc.sin(300deg) * 1pt)),
          curve.close(),
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
              ..foreground,
              fill: color(s.color),
              s.text
            )
          )
        )
      )
    }
  ]
}
