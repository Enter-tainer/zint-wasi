#let zint-wasm = plugin("./zint_typst_plugin.wasm")
#import "./glyphs.typ": _arimo, _ocrb

// Zint's vector coordinates use the same unit as SVG px.
// Typst interprets SVG px as 1px = 0.75pt (72pt/inch ÷ 96px/inch).
// To match the old SVG output size, we scale all coordinates by this factor.
#let _px = 0.75pt

// Default palette for Ultracode and other multi-color symbologies.
// Keys are string representations of zint's internal color indices.
#let _default_palette = (
  "1": rgb(0, 255, 255),    // Cyan
  "2": rgb(0, 0, 255),      // Blue
  "3": rgb(255, 0, 255),    // Magenta
  "4": rgb(255, 0, 0),      // Red
  "5": rgb(255, 255, 0),    // Yellow
  "6": rgb(0, 255, 0),      // Green
  "7": rgb(0, 0, 0),        // Black
  "8": rgb(255, 255, 255),  // White
)

// Resolve a semantic color tag to a typst color value.
//
// Color tags from WASM are one of:
// - "foreground" -> use fg
// - "background" -> use bg
// - (palette: <int>) -> look up in palette
#let _resolve_color(c, fg, bg) = {
  if c == "foreground" { fg }
  else if c == "background" { bg }
  else if type(c) == dictionary and "palette" in c {
    _default_palette.at(str(c.palette), default: black)
  }
  else { black }
}

// Handles option conversion: extracts native typst colors and converts to hex
// strings for WASM. Returns (processed_options, fg_native, bg_native).
#let _proc_options(options) = {
  let result = options
  let fg-native = black
  let bg-native = white

  let proc_color(opt, name) = {
    let c = opt.at(name, default: none)
    if c != none {
      if type(c) == color {
        return (c.to-hex().slice(1), c)
      } else if type(c) == str {
        if c.at(0) == "#" {
          return (c.slice(1), color.rgb(c))
        } else {
          return (c, color.rgb("#" + c))
        }
      } else {
        panic(name + " must be a color or HEX color str; found: " + type(c))
      }
    }
    return none
  }

  let fg-result = proc_color(result, "fg-color")
  if fg-result != none {
    result.insert("fg-color", fg-result.at(0))
    fg-native = fg-result.at(1)
  }
  let bg-result = proc_color(result, "bg-color")
  if bg-result != none {
    result.insert("bg-color", bg-result.at(0))
    bg-native = bg-result.at(1)
  }

  return (result, fg-native, bg-native)
}

// Draw a hexagon polygon centered at (x, y) with given diameter.
// dx-offset/dy-offset are in zint px units, applied before _px conversion.
#let _draw_hexagon(hex, fg, bg, dx-offset: 0, dy-offset: 0) = {
  let r = hex.diameter / 2.0
  let cx = hex.x
  let cy = hex.y
  // Regular hexagon vertices (flat-topped by default, rotation applied)
  let rot-deg = if hex.rotation == 90 { 90 } else if hex.rotation == 180 { 180 } else if hex.rotation == 270 { 270 } else { 0 }
  let rot-rad = rot-deg * calc.pi / 180.0
  let vertices = ()
  for i in range(6) {
    // Pointy-topped hexagon: starts at top (angle = -90deg offset)
    let angle = calc.pi / 3.0 * i - calc.pi / 2.0 + rot-rad
    let vx = cx + r * calc.cos(angle)
    let vy = cy + r * calc.sin(angle)
    vertices.push(((vx + dx-offset) * _px, (vy + dy-offset) * _px))
  }
  place(polygon(fill: _resolve_color(hex.color, fg, bg), ..vertices))
}

// Render text string using typst native curve() with embedded font glyph paths.
// Returns a typst box element containing the rendered text, or none if no glyphs
// could be rendered. Supports native typst colors (CMYK, Oklab, etc.) for fill.
#let _render_hrt(text-str, font-size-px, color, font) = {
  let upm = font.units-per-em
  let ascent = font.ascent
  let descent = font.descent  // negative
  let glyphs = font.glyphs
  let total-height = ascent - descent  // total height in font units

  // Scale factor: font units -> display units (in zint px)
  let s = font-size-px / upm

  // Build curve elements for all glyphs and calculate total advance
  let elements = ()
  let x-cursor = 0
  for ch in text-str {
    let g = glyphs.at(ch, default: none)
    if g == none { continue }
    if g.path.len() > 0 {
      for cmd in g.path {
        let t = cmd.at(0)
        if t == "M" {
          elements.push(curve.move((
            (x-cursor + cmd.at(1)) * s * _px,
            (ascent - cmd.at(2)) * s * _px,
          )))
        } else if t == "L" {
          elements.push(curve.line((
            (x-cursor + cmd.at(1)) * s * _px,
            (ascent - cmd.at(2)) * s * _px,
          )))
        } else if t == "Q" {
          elements.push(curve.quad(
            (
              (x-cursor + cmd.at(1)) * s * _px,
              (ascent - cmd.at(2)) * s * _px,
            ),
            (
              (x-cursor + cmd.at(3)) * s * _px,
              (ascent - cmd.at(4)) * s * _px,
            ),
          ))
        } else if t == "C" {
          elements.push(curve.cubic(
            (
              (x-cursor + cmd.at(1)) * s * _px,
              (ascent - cmd.at(2)) * s * _px,
            ),
            (
              (x-cursor + cmd.at(3)) * s * _px,
              (ascent - cmd.at(4)) * s * _px,
            ),
            (
              (x-cursor + cmd.at(5)) * s * _px,
              (ascent - cmd.at(6)) * s * _px,
            ),
          ))
        } else if t == "Z" {
          elements.push(curve.close(mode: "straight"))
        }
      }
    }
    x-cursor = x-cursor + g.advance
  }

  if x-cursor == 0 { return none }

  let render-width = font-size-px * x-cursor / upm * _px
  let render-height = font-size-px * total-height / upm * _px
  box(width: render-width, height: render-height,
    curve(fill: color, stroke: none, ..elements)
  )
}

// Calculate the rendered text width in zint coordinate units.
#let _hrt_width(text-str, font-size-px, font) = {
  let upm = font.units-per-em
  let total-advance = 0
  for ch in text-str {
    let g = font.glyphs.at(ch, default: none)
    if g != none { total-advance = total-advance + g.advance }
  }
  font-size-px * total-advance / upm
}

/// Draw a barcode of any supported `symbology`.
///
/// *Example:*
///
/// ```example
/// #tiaoma.barcode("12345678", "QRCode", options: (
///   scale: 2.0,
///   fg-color: blue,
///   bg-color: green.lighten(70%),
///   output-options: (
///     barcode-dotty-mode: true
///   ),
///   dot-size: 1.2,
/// ))
/// ```
///
/// - data (str): Data to encode.
/// - symbology (str): Symbology type name; must be one of #l(<symbology>)[supported types].
///
///     Example values: #typst-val("\"Code11\""), #typst-val("\"C25Standard\""), ...
/// - options (dictionary): Additional options to pass to Zint.
///
///     See the #l(<options>)[configuration section] for details on available options and how to use them.
/// -> content
#let barcode(data, symbology, options: (:), ..args) = {
  let data = data
  if type(data) == str {
    data = bytes(data)
  } else if type(data) == array {
    data = bytes(data)
  }

  // Extract native typst colors before converting to hex for WASM
  let (proc-options, fg-native, bg-native) = _proc_options(options)

  let raw-cbor = zint-wasm.gen_with_options(
    cbor.encode((symbology: symbology, ..proc-options)),
    data,
  )
  let plot = cbor(raw-cbor)
  let geom = plot.geometry

  // Compute true bounding box from ALL element coordinates.
  // Initialize with plot dimensions as minimum bounds.
  let bbox-min-x = 0
  let bbox-min-y = 0
  let bbox-max-x = plot.width
  let bbox-max-y = plot.height

  // Rectangles
  for r in geom.rectangles {
    if r.x < bbox-min-x { bbox-min-x = r.x }
    if r.y < bbox-min-y { bbox-min-y = r.y }
    let rx = r.x + r.width
    let ry = r.y + r.height
    if rx > bbox-max-x { bbox-max-x = rx }
    if ry > bbox-max-y { bbox-max-y = ry }
  }

  // Circles
  for c in geom.circles {
    // For ring circles (stroke width > 0), the stroke extends outward
    let extra = if c.width > 0 { c.width } else { 0 }
    let cr = c.diameter / 2.0 + extra
    let cl = c.x - cr
    let ct = c.y - cr
    let crr = c.x + cr
    let cb = c.y + cr
    if cl < bbox-min-x { bbox-min-x = cl }
    if ct < bbox-min-y { bbox-min-y = ct }
    if crr > bbox-max-x { bbox-max-x = crr }
    if cb > bbox-max-y { bbox-max-y = cb }
  }

  // Hexagons
  for h in geom.hexagons {
    let r = h.diameter / 2.0
    let rot-deg = if h.rotation == 90 { 90 } else if h.rotation == 180 { 180 } else if h.rotation == 270 { 270 } else { 0 }
    let rot-rad = rot-deg * calc.pi / 180.0
    for i in range(6) {
      let angle = calc.pi / 3.0 * i - calc.pi / 2.0 + rot-rad
      let vx = h.x + r * calc.cos(angle)
      let vy = h.y + r * calc.sin(angle)
      if vx < bbox-min-x { bbox-min-x = vx }
      if vy < bbox-min-y { bbox-min-y = vy }
      if vx > bbox-max-x { bbox-max-x = vx }
      if vy > bbox-max-y { bbox-max-y = vy }
    }
  }

  // Text (HRT strings)
  let font = _arimo
  for s in geom.strings {
    let txt-w = _hrt_width(s.text, s.font_size, font)
    let left-edge = 0
    let right-edge = 0
    if s.horizontal_align == 1 {
      left-edge = s.x
      right-edge = s.x + txt-w
    } else if s.horizontal_align == 2 {
      left-edge = s.x - txt-w
      right-edge = s.x
    } else {
      left-edge = s.x - txt-w / 2.0
      right-edge = s.x + txt-w / 2.0
    }
    let ascent-px = s.font_size * font.ascent / font.units-per-em
    let descent-px = s.font_size * calc.abs(font.descent) / font.units-per-em
    let top-edge = s.y - ascent-px
    let bottom-edge = s.y + descent-px
    if left-edge < bbox-min-x { bbox-min-x = left-edge }
    if right-edge > bbox-max-x { bbox-max-x = right-edge }
    if top-edge < bbox-min-y { bbox-min-y = top-edge }
    if bottom-edge > bbox-max-y { bbox-max-y = bottom-edge }
  }

  // Compute box dimensions and offsets (all in zint px units)
  let dx-offset = if bbox-min-x < 0 { -bbox-min-x } else { 0 }
  let dy-offset = if bbox-min-y < 0 { -bbox-min-y } else { 0 }
  let natural-width = (bbox-max-x - bbox-min-x) * _px
  let natural-height = (bbox-max-y - bbox-min-y) * _px

  // Build the barcode box at natural size.
  let barcode-box = box(width: natural-width, height: natural-height, clip: false)[
    // Background — covers the full bounding box so nothing bleeds through
    #place(rect(width: natural-width, height: natural-height, fill: bg-native, stroke: none))

    // Rectangles
    #for r in geom.rectangles {
      place(
        dx: (r.x + dx-offset) * _px,
        dy: (r.y + dy-offset) * _px,
        rect(width: r.width * _px, height: r.height * _px, fill: _resolve_color(r.color, fg-native, bg-native), stroke: none),
      )
    }

    // Circles
    #for c in geom.circles {
      let radius = c.diameter / 2.0 * _px
      if c.width > 0 {
        // Ring/annulus: stroked circle
        place(
          dx: (c.x - c.diameter / 2.0 + dx-offset) * _px,
          dy: (c.y - c.diameter / 2.0 + dy-offset) * _px,
          circle(radius: radius, fill: none, stroke: c.width * _px + _resolve_color(c.color, fg-native, bg-native)),
        )
      } else {
        // Filled circle
        place(
          dx: (c.x - c.diameter / 2.0 + dx-offset) * _px,
          dy: (c.y - c.diameter / 2.0 + dy-offset) * _px,
          circle(radius: radius, fill: _resolve_color(c.color, fg-native, bg-native), stroke: none),
        )
      }
    }

    // Hexagons
    #for h in geom.hexagons {
      _draw_hexagon(h, fg-native, bg-native, dx-offset: dx-offset, dy-offset: dy-offset)
    }

    // Text (HRT - human readable text)
    // Rendered using typst native curve() with embedded font glyph paths so we
    // don't depend on the user having Arimo/OCRB installed.
    // Zint provides text Y as baseline position.
    // The curve box covers ascent+|descent| range; we position so baseline aligns.
    #for s in geom.strings {
      let halign = if s.horizontal_align == 1 { left } else if s.horizontal_align == 2 { right } else { center }
      let font = _arimo  // TODO: use _ocrb for UPC/EAN when font info is available
      let txt-img = _render_hrt(s.text, s.font_size, _resolve_color(s.color, fg-native, bg-native), font)
      if txt-img != none {
        // ascent_px = how far above baseline the top of the curve box extends
        let ascent-px = s.font_size * font.ascent / font.units-per-em
        let ty = (s.y - ascent-px + dy-offset) * _px
        // Rendered text width in zint units for alignment
        let txt-w = _hrt_width(s.text, s.font_size, font)
        if halign == center {
          place(
            dx: (s.x - txt-w / 2.0 + dx-offset) * _px,
            dy: ty,
            txt-img,
          )
        } else if halign == left {
          place(
            dx: (s.x + dx-offset) * _px,
            dy: ty,
            txt-img,
          )
        } else {
          place(
            dx: (s.x - txt-w + dx-offset) * _px,
            dy: ty,
            txt-img,
          )
        }
      }
    }
  ]

  // Support image-compatible parameters via ..args for backwards compatibility.
  // The old SVG-based version forwarded ..args to image(), so we emulate the same
  // API: width, height, fit ("contain"/"cover"/"stretch").
  let named = args.named()
  let target-width = named.at("width", default: none)
  let target-height = named.at("height", default: none)
  let fit-value = named.at("fit", default: none)

  if target-width != none or target-height != none or fit-value != none {
    // Explicit dimensions or fit requested — wrap in a sized box with scaling.
    let tw = if target-width != none { target-width } else { auto }
    let th = if target-height != none { target-height } else { auto }
    let fit = if fit-value != none { fit-value } else { "contain" }

    layout(size => {
      // Resolve target dimensions: auto means use available space from layout
      let w = if tw == auto { size.width } else { tw }
      let h = if th == auto { size.height } else { th }

      if fit == "stretch" {
        // Non-uniform scale to fill exactly
        let sx = if w > 0pt { w / natural-width } else { 1.0 }
        let sy = if h > 0pt { h / natural-height } else { 1.0 }
        scale(x: sx * 100%, y: sy * 100%, reflow: true, origin: top + left, barcode-box)
      } else if fit == "cover" {
        // Uniform scale to cover entire area (may crop)
        let sx = if w > 0pt { w / natural-width } else { 1.0 }
        let sy = if h > 0pt { h / natural-height } else { 1.0 }
        let s = calc.max(sx, sy)
        box(width: w, height: h, clip: true,
          scale(x: s * 100%, y: s * 100%, reflow: true, origin: top + left, barcode-box)
        )
      } else {
        // "contain" (default): uniform scale to fit within area, never scale up
        let sx = if w > 0pt and natural-width > w { w / natural-width } else { 1.0 }
        let sy = if h > 0pt and natural-height > h { h / natural-height } else { 1.0 }
        let s = calc.min(sx, sy) * 100%
        scale(x: s, y: s, reflow: true, origin: top + left, barcode-box)
      }
    })
  } else {
    barcode-box
  }
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
  "LOGMARS",
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
  "HIBCMicroPDF",
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
#let dxfilm-edge(data, options: (:), ..args) = barcode(
  data,
  "DXFilmEdge",
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

// EAN standalone variants
#let ean2(data, options: (:), ..args) = barcode(
  data,
  "EAN2",
  options: options,
  ..args,
)
#let ean5(data, options: (:), ..args) = barcode(
  data,
  "EAN5",
  options: options,
  ..args,
)
#let ean8(data, options: (:), ..args) = barcode(
  data,
  "EAN8",
  options: options,
  ..args,
)
#let ean13(data, options: (:), ..args) = barcode(
  data,
  "EAN13",
  options: options,
  ..args,
)

// Composite symbology convenience functions
#let ean8-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "EAN8CC", options: options, ..args,
)
#let ean13-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "EAN13CC", options: options, ..args,
)
#let gs1-128-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "GS1128CC", options: options, ..args,
)
#let dbar-omn-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "DBarOmnCC", options: options, ..args,
)
#let dbar-ltd-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "DBarLtdCC", options: options, ..args,
)
#let dbar-exp-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "DBarExpCC", options: options, ..args,
)
#let upca-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "UPCACC", options: options, ..args,
)
#let upce-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "UPCECC", options: options, ..args,
)
#let dbar-stk-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "DBarStkCC", options: options, ..args,
)
#let dbar-omn-stk-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "DBarOmnStkCC", options: options, ..args,
)
#let dbar-exp-stk-cc(primary, data, mode, options: (:), ..args) = barcode-composite(
  primary, data, mode, "DBarExpStkCC", options: options, ..args,
)
