#import "./lib.typ": *
#import "./lib.typ"
#import "@preview/tidy:0.2.0"

#show link: underline

#{
  set align(center)
  heading(level: 1, text(size: 17pt)[tiaoma])
  par(justify: false)[
    A barcode generator for typst. Using #link("https://github.com/zint/zint")[zint] as backend.
  ]
}

#show heading.where(level: 2): it => {
  text(size: 14pt, it)
}

#let entry-height = 5em
#let entry-gutter = 1em

#let left-right(..args) = grid(columns: (1fr, auto), gutter: 1%, ..args)
#let example-entry(name, code-block, preview, ..extra) = block(breakable: false,
  left-right(
    stack(dir: ttb, spacing: 10pt,
      heading(level: 2, name),
      code-block,
      ..extra,
    ),
    align(right+horizon, block(height: entry-height, width: 7em, preview))
  )
)
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
    column-gutter: 5pt,
    row-gutter: entry-gutter,
    ..content.pos()
  )
  #v(2em)
]

This package provides shortcuts for commonly used barcode types. It also supports all barcode types supported by zint. All additional arguments will be passed to #link("https://typst.app/docs/reference/visualize/image/#definitions-decode", raw("image.decode", lang: "typst")) function. Therefore you may customize the barcode image by passing additional arguments.

#left-right[
  ```typ
  #ean("6975004310001", width: 10em)
  ```
][
  #align(right)[#ean("6975004310001", width: 10em)]
]

For more examples, please refer to the following sections.

#line(length: 100%)

#show "<dbar>": [GS1 DataBar]
#show "(chk)": [w/ Check Digit]
#show "(cc)": [Composite Code]
#show "(omni)": [Omnidirectional]
#show "(omn)": [Omnidirectional]
#show "(stk)": [Stacked]
#show "(exp)": [Expanded]
#show "(ltd)": [Limited]

= EAN (European Article Number)
#section(
example-of-simple("EAN", "eanx", eanx, "1234567890"),
example-of-simple("EAN-14", "ean14", ean14, "1234567890"),
example-of-simple("EAN-13", "ean", ean, "6975004310001"),
example-of-simple("EAN-8", "ean", ean, "12345564"),
example-of-simple("EAN-5", "ean", ean, "12345"),
example-of-simple("EAN-2", "ean", ean, "12"),
example-of-simple("EAN 128 (Legacy)", "ean128", ean128, "[02]12345678901234")[
  EAN 128 is superseded by #link(<GS1>)[GS1].
],
//example-of-simple("EAN (cc)", "eanx-cc", eanx-cc, "3312345768903"),
//example-of-simple("EAN128 (cc)", "ean128-cc", ean128-cc, "[01]95012345678903[3103]000123")
)

= PDF417
#section(
example-of-simple("Micro PDF417", "micro-pdf417", micro-pdf417, "1234567890"),
example-of-simple("PDF417", "pdf417", pdf417, "1234567890"),
example-of-simple("Compact PDF417", "pdf417-comp", pdf417-comp, "1234567890"),
example-of-simple("Truncated PDF417 (Legacy)", "pdf417-trunc", pdf417-trunc, "1234567890"),
)

= GS1 <GS1>

#section(
example-of-simple("GS1-128", "gs1-128", gs1-128, "[01]98898765432106[3202]012345[15]991231"),
example-of-simple("<dbar> Omnidirectional", "dbar-omn", dbar-omn, "1234567890"),
example-of-simple("<dbar> (ltd)", "dbar-ltd", dbar-ltd, "1234567890"),
//example-of-simple("dbar (exp)", "dbar-exp", dbar-exp, "1234567890"),
example-of-simple("<dbar> (stk)", "dbar-stk", dbar-stk, "1234567890"),
example-of-simple("<dbar> (stk) (omn)", "dbar-omn-stk", dbar-omn-stk, "1234567890"),
// example for "<dbar> (exp) (stk)",
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

= C25

#section(
example-of-simple("Standard", "c25-standard", c25-standard, "123"),
example-of-simple("Matrix (Legacy)", "c25-matrix", c25-matrix, "1234567890"),
example-of-simple("Interleaved", "c25-inter", c25-inter, "1234567890"),
example-of-simple("IATA", "c25-iata", c25-iata, "1234567890"),
example-of-simple("Data Logic", "c25-logic", c25-logic, "1234567890"),
example-of-simple("Industrial", "c25-ind", c25-ind, "1234567890"),
)

= Code 128

#section(
example-of-simple("Code 128", "code128", code128, "1234567890"),
example-of-simple("Code 128 (AB)", "code128ab", code128ab, "1234567890"),
example-of-simple("Code 128 (B)", "code128b", code128b, "1234567890"),
)

= UPC (Universal Product Code)

#section(
example-of-simple("UPC-A", "upca", upca, "1234567890"),
example-of-simple("UPC-A (chk)", "upca-chk", upca-chk, "1234567890"),
example-of-simple("UPC-E", "upce", upce, "1234567"),
example-of-simple("UPC-E (chk)", "upce-chk", upce-chk, "1234567"),
// example-of-simple("UPC-A (cc)", "upca-cc", upca-cc, "1234567890"),
// example-of-simple("UPC-E (cc)", "upce-cc", upce-cc, "1234567890"),
)

= HIBC (Health Industry Barcodes)

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

= Postal

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
example-of-simple("Japanese Postal Code", "japan-post", japan-post, "1234567890"),
example-of-simple("Korea Post", "korea-post", korea-post, "123456"),
example-of-simple("POSTNET", "postnet", postnet, "1234567890"),
example-entry("Royal Mail 2D Mailmark (CMDM)",
  raw("#mailmark-2d(\n\t32, 32,\n\t\"JGB 0111234567123456...\"\n)", block: true, lang: "typ"), mailmark-2d(
    32, 32,
    "JGB 011123456712345678CW14NJ1T 0EC2M2QS      REFERENCE1234567890QWERTYUIOPASDFGHJKLZXCVBNM"
  ),
),
example-of-simple("Royal Mail 4-State Customer Code", "rm4scc", rm4scc, "1234567890"),
example-of-simple("Royal Mail 4-State Mailmark", "mailmark-4s", mailmark-4s, "21B2254800659JW5O9QA6Y"),
example-of-simple("Universal Postal Union S10", "upus10", upus10, "RR072705659PL"),
example-of-simple("UPNQR (Univerzalnega Plačilnega Naloga QR)", "upnqr", upnqr, "1234567890"),
example-of-simple("USPS Intelligent Mail", "uspsi-mail", uspsi-mail, "01300123456123456789"),
example-of-simple("OneCode (Legacy)", "onecode", onecode, "01300123456123456789", [
  Superseded by USPS Intelligent Mail
]),
)

= Other Generic Codes

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
example-of-simple("Code 93", "code93", code93, "1234567890"),
example-of-simple("Code One", "code-one", code-one, "1234567890"),
example-of-simple("Data Matrix (ECC200)", "data-matrix", data-matrix, "1234567890"),
example-of-simple("DotCode", "dotcode", dotcode, "1234567890"),
example-of-simple("Extended Code 39", "ex-code39", ex-code39, "1234567890"),
example-of-simple("Grid Matrix", "grid-matrix", grid-matrix, "1234567890"),
example-of-simple("Han Xin (Chinese Sensible)", "hanxin", hanxin, "abc123全ň全漄"),
example-of-simple("IBM BC412 (SEMI T1-95)", "bc412", bc412, "1234567890"),
example-of-simple("ISBN", "isbnx", isbnx, "9789861817286"),
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

= RSS14 (Legacy)

These codes are now referred to as GS1. See #link(<GS1>)[GS1 section] for newer implementations.

#section(
example-of-simple("RSS14", "rss14", rss14, "1234567890"),
example-of-simple("RSS14 (ltd)", "rss-ltd", rss-ltd, "1234567890"),
// example-of-simple("RSS14 (exp)", "rss-exp", rss-exp, "1234567890"),
example-of-simple("RSS14 (stk)", "rss14-stack", rss14-stack, "1234567890"),
example-of-simple("RSS14 (stk) (omn)", "rss14-stack-omni", rss14-stack-omni, "1234567890"),
// example-of-simple("RSS14 (exp) (stk)", "rss-exp-stack", rss-exp-stack, "1234567890"),
// example-of-simple("RSS14 (cc)", "rss14-cc", rss14-cc, "1234567890"),
// example-of-simple("RSS14 (exp) (cc)", "rss-exp-cc", rss-exp-cc, "1234567890"),
// example-of-simple("RSS14 (ltd) (cc)", "rss-ltd-cc", rss-ltd-cc, "1234567890"),
// example-of-simple("RSS14 (stk) (cc)", "rss14-stack-cc", rss14-stack-cc, "1234567890"),
// example-of-simple("RSS14 (omni) (cc)", "rss14-omni-cc", rss14-omni-cc, "1234567890"),
// example-of-simple("RSS14 (exp) (stk) (cc)", "rss-exp-stack-cc", rss-exp-stack-cc, "1234567890"),
)

= Additional Code Options <additional_options>

While most barcodes are supported through shortcut functions, some require additional configuration/options (such as composite codes). Check out the #link("https://zint.org.uk/manual")[official Zint manual] for a complete description of supported codes.

#let docs = tidy.parse-module(
  read("lib.typ"),
  name: "tiaoma",
  scope: (tiaoma: lib)
)

#tidy.show-module(
  docs,
  first-heading-level: 1,
  show-module-name: false,
  show-outline: false,
)
