#import "./lib.typ": *
#import "./lib.typ"

#set page(
  paper: "a4",
  margin: (
    y: 1em,
    x: 2em,
  ),
)

// --- Type badge utilities ---

#let _rainbow = gradient.linear(angle: 7deg,
  (rgb("#7cd5ff"), 0%), (rgb("#a6fbca"), 33%),
  (rgb("#fff37c"), 66%), (rgb("#ffa49d"), 100%),
)

#let _type-colors = (
  "content": rgb("#a6ebe6"),
  "str": rgb("#d1ffe2"), "string": rgb("#d1ffe2"),
  "none": rgb("#ffcbc4"), "auto": rgb("#ffcbc4"),
  "bool": rgb("#ffedc1"), "boolean": rgb("#ffedc1"),
  "int": rgb("#e7d9ff"), "integer": rgb("#e7d9ff"),
  "float": rgb("#e7d9ff"), "length": rgb("#e7d9ff"),
  "array": rgb("#eff0f3"), "dictionary": rgb("#eff0f3"),
  "color": _rainbow, "gradient": _rainbow,
  "function": rgb("#f9dfff"),
  "stroke": rgb("#eff0f3"),
)

#let _type-docs = (
  "content": "https://typst.app/docs/reference/foundations/content/",
  "str": "https://typst.app/docs/reference/foundations/str/",
  "string": "https://typst.app/docs/reference/foundations/str/",
  "none": "https://typst.app/docs/reference/foundations/none/",
  "auto": "https://typst.app/docs/reference/foundations/auto/",
  "bool": "https://typst.app/docs/reference/foundations/bool/",
  "boolean": "https://typst.app/docs/reference/foundations/bool/",
  "int": "https://typst.app/docs/reference/foundations/int/",
  "integer": "https://typst.app/docs/reference/foundations/int/",
  "float": "https://typst.app/docs/reference/foundations/float/",
  "length": "https://typst.app/docs/reference/foundations/length/",
  "array": "https://typst.app/docs/reference/foundations/array/",
  "dictionary": "https://typst.app/docs/reference/foundations/dictionary/",
  "color": "https://typst.app/docs/reference/visualize/color/",
  "gradient": "https://typst.app/docs/reference/visualize/gradient/",
  "stroke": "https://typst.app/docs/reference/visualize/stroke/",
  "function": "https://typst.app/docs/reference/foundations/function/",
)

/// Render a colored type badge, linked to typst docs when available.
#let typst-type(..types, docs: (:)) = {
  let all-docs = _type-docs + docs
  for (i, ty) in types.pos().enumerate() {
    if i > 0 {
      h(2pt)
      text(size: 9pt)[or]
      h(2pt)
    }
    let clr = _type-colors.at(ty, default: rgb("#eff0f3"))
    let badge = box(outset: 2pt, fill: clr, radius: 2pt, raw(ty))
    let dest = all-docs.at(ty, default: none)
    h(2pt)
    if dest != none { link(dest, badge) } else { badge }
    h(2pt)
  }
}

/// Render a colored value literal.
#let typst-val(v) = {
  let fg = black
  if v.starts-with("\"") { fg = _type-colors.at("str").darken(50%).saturate(70%) }
  else if v == "true" or v == "false" { fg = _type-colors.at("bool").darken(50%).saturate(70%) }
  else if v == "none" { fg = _type-colors.at("none").darken(50%).saturate(70%) }
  else { fg = _type-colors.at("float").darken(50%).saturate(70%) }
  text(fill: fg, raw(v))
}

// --- Shortcuts ---

#let l(dest, body) = underline(link(dest, body))

#let ref-table(columns: (auto, 100pt, 1fr, auto), head-rows: 1, key-column: true, ..cells) = {
  table(
    columns: columns,
    align: (center + horizon, center + horizon, left + horizon, center + horizon),
    stroke: gray.lighten(60%),
    fill: (col, row) => if row < head-rows {
      blue.lighten(80% - 10% * (head-rows - 1 - row))
    } else if col == 0 and key-column {
      blue.lighten(90%)
    },
    ..cells,
  )
}

// ============================================================
// Document
// ============================================================

#{
  set align(center)
  heading(level: 1, text(size: 17pt)[tiaoma])
  [A barcode generator for typst that provides type safe API bindings for #l("https://zint.org.uk")[Zint] (#l("https://github.com/zint/zint")[GitHub]) library through a WASM #l("https://typst.app/docs/reference/foundations/plugin/")[plugin].]
}

#v(10pt)

See #l("https://zint.org.uk/manual")[official Zint manual] for a more in-depth description of supported functionality.

= API

== `barcode` function

#block(inset: (left: 12pt))[
  ```typ
  #barcode(data, symbology, options: (:), foreground: (fill: black), background: (:), ..args)
  ```
]

Encodes `data` into a barcode of the given `symbology` and renders it using native typst drawing primitives.

#ref-table(
  columns: (auto, auto, 1fr, auto),
  [*Parameter*], [*Type*], [*Description*], [*Default*],
  [`data`], typst-type("str", "array"), [Data to encode.], [--],
  [`symbology`], typst-type("str"), [Symbology name. See #l("https://zint.org.uk/manual")[Zint manual] for supported types.], [--],
  [`options`], typst-type("dictionary"), [Zint encoding options. See #l(<options>)[configuration].], typst-val("(:)"),
  [`foreground`], typst-type("dictionary"), [Foreground and HRT text styling. `fill` sets bar color; all keys are spread into typst's `text()` for HRT rendering.], [(fill: black)],
  [`background`], typst-type("dictionary"), [Background styling for the container box. Accepts `fill`, `stroke`, `radius`, `outset`, `inset`.], typst-val("(:)"),
  [`..args`], [any], [Forwarded to the outer `box()`.], [--],
)

=== `foreground` dictionary

The `foreground` dictionary controls both the color of barcode elements (bars, dots, hexagons) and the styling of Human Readable Text (HRT). Its contents are spread into typst's #l("https://typst.app/docs/reference/text/text/")[`text()`] function, so any valid `text()` parameter can be used.

The `fill` key is special: it is used as the foreground color for _all_ barcode elements, not just text.

#ref-table(
  columns: (auto, auto, 1fr, auto),
  [*Key*], [*Type*], [*Description*], [*Default*],
  [`fill`], typst-type("color", "gradient"), [Foreground color for bars and text.], typst-val("black"),
  [`font`], typst-type("str", "array"), [Font family for HRT.], [inherited],
  [`size`], typst-type("length"), [Overrides HRT font size.], [from zint],
  [`weight`], typst-type("str", "int"), [Font weight for HRT.], [inherited],
  [_..._], [any], [Any other `text()` parameter.], [--],
)

=== `background` dictionary

The `background` dictionary is spread into the outer `box()` that contains the barcode.

#ref-table(
  columns: (auto, auto, 1fr, auto),
  [*Key*], [*Type*], [*Description*], [*Default*],
  [`fill`], typst-type("color", "gradient", "none"), [Background fill.], typst-val("none"),
  [`stroke`], typst-type("stroke", "none"), [Border stroke.], typst-val("none"),
  [`radius`], typst-type("length", "dictionary"), [Corner radius.], typst-val("0pt"),
  [`outset`], typst-type("length", "dictionary"), [Outset around box.], typst-val("0pt"),
  [`inset`], typst-type("length", "dictionary"), [Inset padding.], typst-val("0pt"),
)

== Shortcut functions

Shortcut functions accept the same arguments as `barcode` but don't require `symbology` to be specified. See `lib.typ` for the full list.

// TODO: Generate shortcut list from symbology.rs

== Zint configuration <options>

All exported functions support optionally providing the `options` dictionary which is passed to Zint. This provides means to fully configure generated symbols.

The following values are valid for the `options` dictionary:

#ref-table(
  columns: (auto, 100pt, 1fr, auto),
  [*Field*], [*Type*], [*Description*], [*Default*],
  [height], typst-type("float"), [Barcode height in X-dimensions (ignored for fixed-width barcodes).], typst-val("none"),
  [whitespace-width], typst-type("int"), [Width in X-dimensions of whitespace to left & right of barcode.], typst-val("0"),
  [whitespace-height], typst-type("int"), [Height in X-dimensions of whitespace above & below the barcode.], typst-val("0"),
  [border-width], typst-type("int"), [Size of border in X-dimensions.], typst-val("0"),
  [show-hrt], typst-type("bool"), [Whether to generate Human Readable Text.], typst-val("true"),
  link(<output_options>, underline[output-options]),
  typst-type("int", "array", "dictionary"),
  [Various output parameters (bind, box, etc; see below).], typst-val("0"),
  [primary], typst-type("str"), [Primary message data (MaxiCode, Composite).], typst-val("\"\""),
  [option-1], typst-type("int"), [Symbol-specific option (see #l("https://zint.org.uk/manual")[manual]).], typst-val("-1"),
  [option-2], typst-type("int"), [Symbol-specific option (see #l("https://zint.org.uk/manual")[manual]).], typst-val("0"),
  link(<opt_3>, underline[option-3]), typst-type("int", "str"), [Symbol-specific option (see #l("https://zint.org.uk/manual")[manual]).], typst-val("0"),
  link(<input_mode>, underline[input-mode]),
  typst-type("int", "str", "array", "dictionary"),
  [Encoding of input data.], typst-val("0"),
  [dot-size], typst-type("float"), [Size of dots used in `BARCODE_DOTTY_MODE`.], typst-val("4.0 / 5.0"),
  [text-gap], typst-type("float"), [Gap between barcode and text (HRT) in X-dimensions.], typst-val("1.0"),
  [guard-descent], typst-type("float"), [Height in X-dimensions that EAN/UPC guard bars descend.], typst-val("5.0"),
  [ultracode-colors], typst-type("dictionary"), [Custom Ultracode color palette. Maps string codes `"1"`--`"8"` to colors.], [CMYK defaults],
)

#pagebreak()
=== Input Mode <input_mode>

Input mode options specify how Zint handles input data. Zint uses #typst-type("int") bitflags, but tiaoma accepts several formats.

==== Input format (mutually exclusive)

#ref-table(
  columns: (auto, auto, auto, 1fr),
  [*Constant*], typst-type("int"), typst-type("str"), [*Description*],
  raw("DATA_MODE"), typst-val("0"), typst-val("\"data\""), [Use full 8-bit range interpreted as binary data.],
  raw("UNICODE_MODE"), typst-val("1"), typst-val("\"unicode\""), [Use UTF-8 input.],
  raw("GS1_MODE"), typst-val("2"), typst-val("\"gs1\""), [Encode GS1 data using FNC1 characters.],
)

==== Behavior customization

#ref-table(
  columns: (auto, auto, auto, 1fr),
  [*Constant*], typst-type("int"), typst-type("str"), [*Description*],
  raw("ESCAPE_MODE"), typst-val("8"), typst-val("\"escape\""), [Process input data for escape sequences.],
  raw("GS1PARENS_MODE"), typst-val("16"), typst-val("\"gs1-parentheses\""), [Parentheses used in GS1 data instead of square brackets.],
  raw("GS1NOCHECK_MODE"), typst-val("32"), typst-val("\"gs1-no-check\""), [Do not check GS1 data for validity.],
  raw("HEIGHTPERROW_MODE"), typst-val("64"), typst-val("\"height-per-row\""), [Interpret `height` as per-row rather than overall height.],
  raw("FAST_MODE"), typst-val("128"), typst-val("\"fast\""), [Use faster if less optimal encodation (currently Data Matrix only).],
  raw("EXTRA_ESCAPE_MODE"), typst-val("256"), typst-val("\"extra-escape\""), [Process special symbology-specific escape sequences.],
)

==== String Value <input_mode_str>

`input-mode` of #typst-type("str") type is assumed to be an _input format_ value from the first table.

==== Array Value <input_mode_arr>

`input-mode` of #typst-type("array") type is assumed to be a list of #typst-type("str") values; individual constants will be OR'd together.

==== Dictionary Value <input_mode_dict>

`input-mode` of #typst-type("dictionary") type is assumed to be #typst-type("str")--#typst-type("bool") pairs where keys are constants from the above tables.


=== Output Options <output_options>

Output options specify how Zint generates the barcode.

#ref-table(
  columns: (auto, auto, auto, 1fr),
  [*Constant*], typst-type("int"), typst-type("str"), [*Description*],
  raw("BARCODE_BIND_TOP"), typst-val("1"), typst-val("\"barcode-bind-top\""), [Boundary bar _above_ the symbol.],
  raw("BARCODE_BIND"), typst-val("2"), typst-val("\"barcode-bind\""), [Boundary bars _above_ and _below_ the symbol.],
  raw("BARCODE_BOX"), typst-val("4"), typst-val("\"barcode-box\""), [Box surrounding the symbol and whitespace.],
  raw("SMALL_TEXT"), typst-val("32"), typst-val("\"small-text\""), [Use a smaller font for HRT.],
  raw("BOLD_TEXT"), typst-val("64"), typst-val("\"bold-text\""), [Embolden the HRT.],
  raw("BARCODE_DOTTY_MODE"), typst-val("256"), typst-val("\"barcode-dotty-mode\""), [Plot matrix symbol using dots rather than squares.],
  raw("GS1_GS_SEPARATOR"), typst-val("512"), typst-val("\"gs1-gs-separator\""), [Use GS instead of FNC1 as GS1 separator (Data Matrix).],
  raw("BARCODE_QUIET_ZONES"), typst-val("2048"), typst-val("\"barcode-quiet-zones\""), [Add compliant quiet zones.],
  raw("BARCODE_NO_QUIET_ZONES"), typst-val("4096"), typst-val("\"barcode-no-quiet-zones\""), [Disable quiet zones.],
  raw("COMPLIANT_HEIGHT"), typst-val("8192"), typst-val("\"compliant-height\""), [Use standard height as default.],
  raw("EANUPC_GUARD_WHITESPACE"), typst-val("16384"), typst-val("\"ean-upc-guard-whitespace\""), [Add quiet zone indicators to HRT whitespace (EAN/UPC).],
)

==== Array Value <output_options_arr>

`output-options` of #typst-type("array") type: list of #typst-type("str") constant names.

==== Dictionary Value <output_options_dict>

`output-options` of #typst-type("dictionary") type: #typst-type("str")--#typst-type("bool") pairs where keys are constant names.

=== Option 3 <opt_3>

Constants associated with `option-3` values. Can be specified as #typst-type("int") or #typst-type("str").

#ref-table(
  columns: (auto, auto, auto, 1fr),
  [*Constant*], typst-type("int"), typst-type("str"), [*Description*],
  raw("DM_SQUARE"), typst-val("100"), typst-val("\"square\""), [Only consider square versions on automatic symbol size selection.],
  raw("DM_DMRE"), typst-val("101"), typst-val("\"rect\""), [Consider DMRE versions on automatic symbol size selection.],
  raw("DM_ISO_144"), typst-val("128"), typst-val("\"iso-144\""), [Use ISO instead of "de facto" format for 144×144.],
  raw("ZINT_FULL_MULTIBYTE"), typst-val("200"), typst-val("\"full-multibyte\""), [Enable Kanji/Hanzi compression for Latin-1 & binary data.],
  raw("ULTRA_COMPRESSION"), typst-val("128"), typst-val("\"compression\""), [Enable Ultracode compression _(experimental)_.],
)

#pagebreak()
= Examples <examples>

#barcode("Hello, World!", "QRCode")

#barcode("12345678", "Code128")

#barcode("9780201379624", "EANX")

#barcode("This is a MaxiCode", "MaxiCode")

#barcode("Ultracode test", "Ultra")
