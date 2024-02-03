#import "./lib.typ": *
#import "./lib.typ"
#import "@preview/tidy:0.2.0"
#import "@preview/tablex:0.0.8": tablex
#import "./tidy_style.typ"

// TODO: replace tidy_style with tidy.styles.default next version
#set page(
  paper: "a4",
  margin: (
    y: 1em,
    x: 2em,
  )
)

#show link: underline

#{
  set align(center)
  heading(level: 1, text(size: 17pt)[tiaoma])
  [A barcode generator for typst.]
}

#show heading.where(level: 3): it => {
  text(size: 13pt, it)
}

#let entry-height = 5em
#let entry-gutter = 1em

#let left-right(..args) = grid(columns: (1fr, auto), gutter: 1%, ..args)
#let example-entry(name, code-block, preview, ..extra) = block(breakable: false)[
  #heading(level: 3, name)
  #if extra.pos().len() > 0 {
    stack(dir:ttb, spacing: 5pt, ..extra)
  }

  *Example:*

  #block(inset: 5pt, stroke: 2pt + gray.lighten(80%), radius: 2pt, width: 100%,
    text(size: 8pt, par(linebreaks: "simple", code-block))
  )

  *Result:*

  #block(
    height: entry-height,
    inset: (
      left: 10pt,
      right: 10pt,
      top: 5pt,
      bottom: 5pt,
    ),
    fill: gray.lighten(90%),
    radius: 5pt,
    width: 100%,
    align(center + horizon, preview),
  )
]

#let example-of-simple(name, func-name, func, data, ..extra) = example-entry(
  name,
  raw("#" + func-name + "(\"" + data + "\")",
    block: true,
    lang: "typ"
  ),
  func(data, fit: "contain"),
  ..extra
)


// Only works when entire content fits on a single page
#let section(..content) = [
  #v(1em)
  #grid(
    columns: (1fr, 1fr),
    column-gutter: entry-gutter,
    row-gutter: 10pt,
    ..content
  )
  #v(2em)
]

It relies on #link("https://zint.org.uk")[Zint] (#link("https://github.com/zint/zint")[GitHub]) as a backend for code generation and provides type safe API bindings in Typst code through a WASM #link("https://typst.app/docs/reference/foundations/plugin/")[plugin].

#align(horizon, left-right[
  ```typ
  #ean("6975004310001", width: 10em, options: (
    fg_color: red
  ))
  ```
][
  #align(right)[#ean("6975004310001", width: 10em, options: (
    fg_color: red
  ))]
])

= Package API

While most barcodes are supported through shortcut functions, some require additional configuration/options (such as composite codes). Check out the #link("https://zint.org.uk/manual")[official Zint manual] for a complete description of supported codes.

== Zint configuration <additional_options>

All exported functions support optionally providing the `options` dictionary which is passed to Zint. This provides means to fully configure generated images.

The following values are valid for the `options` dictionary:

#let typst-type(v) = {
  let ty-box(v) = tidy_style.show-type(v, style-args: (colors: tidy_style.colors))

  for (i, e) in v.split(",").map(ty-box).enumerate() {
    if i > 0 {
      h(2pt)
      text(size: 9pt)[or]
      h(2pt)
    }
    e
  }
}
#let typst-val(text, block: false) = raw(text, block: block, lang: "typ")

#tablex(
  columns: (auto, auto, 1fr, auto),
  align: (center + horizon, center + horizon, left + horizon, center + horizon),
  auto-vlines: false,
  fill: (col, row) => if row == 0 {
    gray.lighten(80%)
  } else if col == 0 {
    gray.lighten(90%)
  } else {
    none
  },
  stroke: gray.lighten(60%),
  [*Field*], [*Type*], [*Description*], [*Default*],
  [height], typst-type("float"), [Barcode height in X-dimensions (ignored for fixed-width barcodes)], typst-val("none"),
  [scale], typst-type("float"), [Scale factor when printing barcode, i.e. adjusts X-dimension], typst-val("1.0"),
  [whitespace_width], typst-type("int"), [Width in X-dimensions of whitespace to left & right of barcode], typst-val("0"),
  [whitespace_height], typst-type("int"), [Height in X-dimensions of whitespace above & below the barcode], typst-val("0"),
  [border_width], typst-type("int"), [Size of border in X-dimensions], typst-val("0"),
  [output_options], typst-type("int"), [Various output parameters (bind, box etc, see below)], typst-val("0"),
  [fg_color], typst-type("color"), [foreground color], typst-val("black"),
  [bg_color], typst-type("color"), [background color], typst-val("white"),
  [primary], typst-type("str"), [Primary message data (MaxiCode, Composite)], typst-val("\"\""),
  [option_1], typst-type("int"), [Symbol-specific options (see #link("https://zint.org.uk/manual")[manual])], typst-val("-1"),
  [option_2], typst-type("int"), [Symbol-specific options (see #link("https://zint.org.uk/manual")[manual])], typst-val("0"),
  [option_3], typst-type("int,str"), [Symbol-specific options (see #link("https://zint.org.uk/manual")[manual])], typst-val("0"),
  [show_hrt], typst-type("bool"), [Whether to show Human Readable Text (HRT)], typst-val("true"),
  [input_mode], typst-type("int"), [Encoding of input data], typst-val("0"),
  [eci], typst-type("int"), [Extended Channel Interpretation.], typst-val("0"),
  [dot_size], typst-type("float"), [Size of dots used in BARCODE_DOTTY_MODE.], typst-val("4.0 / 5.0"),
  [text_gap], typst-type("float"), [Gap between barcode and text (HRT) in X-dimensions.], typst-val("1.0"),
  [guard_decent], typst-type("float"), [Height in X-dimensions that EAN/UPC guard bars descend.], typst-val("5.0"),
)

Supported `option_3` string values are:
- #typst-val("square") - Only consider square versions on automatic symbol size selection
- #typst-val("rect") - Consider DMRE versions on automatic symbol size selection
- #typst-val("iso-144") - Use ISO instead of "de facto" format for 144x144 (i.e. don't skew ECC)
- #typst-val("full-multibyte") - Enable Kanji/Hanzi compression for Latin-1 & binary data
- #typst-val("compression") - Enable Ultracode compression *(experimental)*

#pagebreak()

== Module Exports

#let docs = tidy.parse-module(
  read("lib.typ"),
  name: "tiaoma",
  scope: (tiaoma: lib, typst-type: typst-type)
)

#tidy.show-module(
  docs,
  first-heading-level: 2,
  style: tidy_style,
  show-module-name: false,
  show-outline: false,
)

#show "<dbar>": [GS1 DataBar]
#show "(chk)": [w/ Check Digit]
#show "(cc)": [Composite Code]
#show "(omni)": [Omnidirectional]
#show "(omn)": [Omnidirectional]
#show "(stk)": [Stacked]
#show "(exp)": [Expanded]
#show "(ltd)": [Limited]

#pagebreak()

= Examples

== EAN (European Article Number)
#section(
example-of-simple("EANX", "eanx", eanx, "1234567890"),
example-of-simple("EAN-14", "ean14", ean14, "1234567890"),
example-of-simple("EAN-13", "eanx", eanx, "6975004310001"),
example-of-simple("EAN-8", "eanx", eanx, "12345564"),
example-of-simple("EAN-5", "eanx", eanx, "12345"),
example-of-simple("EAN-2", "eanx", eanx, "12"),
// example for "EAN (cc)"
)

#pagebreak()
== PDF417
#section(
example-of-simple("Micro PDF417", "micro-pdf417", micro-pdf417, "1234567890"),
example-of-simple("PDF417", "pdf417", pdf417, "1234567890"),
example-of-simple("Compact PDF417", "pdf417-comp", pdf417-comp, "1234567890"),
)

#pagebreak()
== GS1

#section(
example-of-simple("GS1-128", "gs1-128", gs1-128, "[01]98898765432106[3202]012345[15]991231"),
example-of-simple("<dbar> Omnidirectional", "dbar-omn", dbar-omn, "1234567890"),
example-of-simple("<dbar> (ltd)", "dbar-ltd", dbar-ltd, "1234567890"),
// example for "dbar (exp)"
example-of-simple("<dbar> (stk)", "dbar-stk", dbar-stk, "1234567890"),
example-of-simple("<dbar> (stk) (omn)", "dbar-omn-stk", dbar-omn-stk, "1234567890"),
// example for "<dbar> (exp) (stk)"
// example for "<dbar> (omn) (cc)"
// example for "<dbar> (omn) (cc)"
// example for "<dbar> (ltd) (cc)"
// example for "<dbar> (exp) (cc)"
// example for "<dbar> (stk) (cc)"
// example for "<dbar> (omn) (stk) (cc)"
// example for "<dbar> (exp) (stk) (cc)"
)

// TODO: Remove once utilities and above examples are provided
Zint also supports (omn), (ltd), (exp), (stk) and (cc) variants of GS1. See #link(<additional_options>)[additional options] section for information on how to use them.

#pagebreak()
== C25

#section(
example-of-simple("Standard", "c25-standard", c25-standard, "123"),
example-of-simple("Interleaved", "c25-inter", c25-inter, "1234567890"),
example-of-simple("IATA", "c25-iata", c25-iata, "1234567890"),
example-of-simple("Data Logic", "c25-logic", c25-logic, "1234567890"),
example-of-simple("Industrial", "c25-ind", c25-ind, "1234567890"),
)

#pagebreak()
== UPC (Universal Product Code)

#section(
example-of-simple("UPC-A", "upca", upca, "01234500006"),
example-of-simple("UPC-A (chk)", "upca-chk", upca-chk, "012345000065"),
example-of-simple("UPC-E", "upce", upce, "123456"),
example-of-simple("UPC-E (chk)", "upce-chk", upce-chk, "12345670"),
// example for "UPC-A (cc)"
// example for "UPC-E (cc)"
)

#pagebreak()
== HIBC (Health Industry Barcodes)

#section(
example-of-simple("Code 128", "hibc-128", hibc-128, "1234567890"),
example-of-simple("Code 39", "hibc-39", hibc-39, "1234567890"),
example-of-simple("Data Matrix", "hibc-dm", hibc-dm, "1234567890"),
example-of-simple("QR", "hibc-qr", hibc-qr, "1234567890"),
example-of-simple("PDF417", "hibc-pdf", hibc-pdf, "1234567890"),
example-of-simple("Micro PDF417", "hibc-mic-pdf", hibc-mic-pdf, "1234567890"),
example-of-simple("Codablock-F", "hibc-codablock-f", hibc-codablock-f, "1234567890"),
example-of-simple("Aztec", "hibc-aztec", hibc-aztec, "1234567890"),
)

#pagebreak()
== Postal

#section(
example-of-simple("Australia Post Redirection", "aus-redirect", aus-redirect, "12345678"),
example-of-simple("Australia Post Reply Paid", "aus-reply", aus-reply, "12345678"),
example-of-simple("Australia Post Routing", "aus-route", aus-route, "12345678"),
example-of-simple("Australia Post Standard Customer", "aus-post", aus-post, "12345678"),
example-of-simple("Brazilian CEPNet Postal Code", "cepnet", cepnet, "1234567890"),
example-of-simple("DAFT Code", "daft", daft, "DAFTFDATATFDTFAD"),
example-of-simple("Deutsche Post Identcode", "dp-ident", dp-ident, "1234567890"),
example-of-simple("Deutsche Post Leitcode", "dp-leitcode", dp-leitcode, "1234567890"),
example-of-simple("Deutsher Paket Dienst", "dpd", dpd, "0123456789012345678901234567"),
example-of-simple("Dutch Post KIX Code", "kix", kix, "1234567890"),
)

#section(
example-of-simple("Japanese Postal Code", "japan-post", japan-post, "1234567890"),
example-of-simple("Korea Post", "korea-post", korea-post, "123456"),
example-of-simple("POSTNET", "postnet", postnet, "1234567890"),
example-entry("Royal Mail 2D Mailmark (CMDM)",
  raw("#mailmark-2d(\n\t32, 32,\n\t\"JGB 011123456712345678CW14NJ1T 0EC2M2QS      REFERENCE1234567890QWERTYUIOPASDFGHJKLZXCVBNM\"\n)", block: true, lang: "typ"), mailmark-2d(
    32, 32,
    "JGB 011123456712345678CW14NJ1T 0EC2M2QS      REFERENCE1234567890QWERTYUIOPASDFGHJKLZXCVBNM"
  ),
),
example-of-simple("Royal Mail 4-State Customer Code", "rm4scc", rm4scc, "1234567890"),
example-of-simple("Royal Mail 4-State Mailmark", "mailmark-4s", mailmark-4s, "21B2254800659JW5O9QA6Y"),
example-of-simple("Universal Postal Union S10", "upus10", upus10, "RR072705659PL"),
example-of-simple("UPNQR (Univerzalnega Plačilnega Naloga QR)", "upnqr", upnqr, "1234567890"),
example-of-simple("USPS Intelligent Mail", "usps-imail", usps-imail, "01300123456123456789"),
)

#pagebreak()
== Other Generic Codes

#section(
example-of-simple("Aztec Code", "aztec", aztec, "1234567890"),
example-of-simple("Aztec Rune", "azrune", azrune, "122"),
example-of-simple("Channel Code", "channel", channel, "123456"),
example-of-simple("Codabar", "codabar", codabar, "A123456789B"),
example-of-simple("Codablock-F", "codablock-f", codablock-f, "1234567890"),
example-of-simple("Code 11", "code11", code11, "0123452"),
example-of-simple("Code 16k", "code16k", code16k, "1234567890"),
example-of-simple("Code 32", "code32", code32, "12345678"),
example-of-simple("Code 39", "code39", code39, "1234567890"),
example-of-simple("Code 49", "code49", code49, "1234567890"),
)
#section(
example-of-simple("Code 128", "code128", code128, "1234567890"),
example-of-simple("Code 128 (AB)", "code128ab", code128ab, "1234567890"),
example-of-simple("Code One", "code-one", code-one, "1234567890"),
example-of-simple("Data Matrix (ECC200)", "data-matrix", data-matrix, "1234567890"),
example-of-simple("DotCode", "dotcode", dotcode, "1234567890"),
example-of-simple("Extended Code 39", "ex-code39", ex-code39, "1234567890"),
example-of-simple("Grid Matrix", "grid-matrix", grid-matrix, "1234567890"),
example-of-simple("Han Xin (Chinese Sensible)", "hanxin", hanxin, "abc123全ň全漄"),
example-of-simple("IBM BC412 (SEMI T1-95)", "bc412", bc412, "1234567890"),
example-of-simple("ISBN", "isbnx", isbnx, "9789861817286"),
)
#pagebreak()
#section(
example-of-simple("ITF-14", "itf14", itf14, "1234567890"),
example-of-simple("LOGMARS", "logmars", logmars, "1234567890"),
example-of-simple("MaxiCode", "maxicode", maxicode, "1234567890"),
example-of-simple("Micro QR", "micro-qr", micro-qr, "1234567890"),
example-of-simple("MSI Plessey", "msi-plessey", msi-plessey, "1234567890"),
example-of-simple("NVE-18 (SSCC-18)", "nve18", nve18, "1234567890"),
example-of-simple("Pharmacode One-Track", "pharma", pharma, "123456"),
example-of-simple("Pharmacode Two-Track", "pharma-two", pharma-two, "12345678"),
example-of-simple("Pharmazentralnummer", "pzn", pzn, "12345678"),
example-of-simple("Planet", "planet", planet, "1234567890"),
)
#pagebreak()
#section(
example-of-simple("Plessey", "plessey", plessey, "1234567890"),
example-of-simple("QR Code", "qrcode", qrcode, "1234567890"),
example-of-simple("Rectangular Micro QR Code (rMQR)", "rmqr", rmqr, "1234567890"),
example-of-simple("Telepen Numeric", "telepen-num", telepen-num, "1234567890"),
example-of-simple("Telepen", "telepen", telepen, "ABCD12345"),
example-of-simple("Ultracode", "ultra", ultra, "1234567890"),
example-of-simple("Vehicle Identification Number", "vin", vin, "2GNFLGE30D6201432"),
example-of-simple("Facing Identification Mark", "fim", fim, "A"),
example-of-simple("Flattermarken", "flat", flat, "123")[
  Used for marking book covers to indicate volume order.
],
)
