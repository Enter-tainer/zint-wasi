#let zint-wasm = plugin("./zint_typst_plugin.wasm")

/// Option to use a square Data Matrix.
#let DM_SQUARE = int(100);
/// Option to use a rectangular Data Matrix.
#let DM_DMRE = int(101);

#let DM_ISO_144 = int(128);
#let ULTRA_COMPRESSION = int(128);

/// Draw a barcode SVG of any supported `type`.
///
/// *Example:*
/// #example(`tiaoma.barcode("12345678", "AusPost")`)
///
/// - data (str): Data to encode in the 
/// - type (str): Symbology type name; must be one of #link("https://github.com/Enter-tainer/zint-wasi/blob/master/zint-wasm-rs/src/options/symbology.rs")[supported types].
///
///     Example values: `"Code11"`, `"C25Standard"`, ...
/// - options (dictionary): Any additional options to pass to Zint.
///     
///     See #link("https://zint.org.uk/manual/chapter/5")[Zint manual: Using API] for details on available options and how to use them.
/// - ..args (any): Any additional arguments to forward to #link("https://typst.app/docs/reference/visualize/image/#definitions-decode", raw("image.decode", lang: "typ")) function.
/// -> content
#let barcode(data, type, options: (:), ..args) = image.decode(
  zint-wasm.gen_with_options(
    cbor.encode(
      (symbology: (type: type), ..options)
    ), bytes(data)
  ),
  format: "svg",
  ..args
)

/// Returns option value for given Data Matrix _width_ and _height_.
///
/// Zint allows square and rectangular values to be enforced with `DM_SQUARE` and `DM_DMRE` (respectively); see Zint manual for details.
///
/// - width (int): Data Matrix width
/// - height (int): Data Matrix height
/// -> int
#let dm-size(height, width) = {
  // Copied from DM size table
  if height == 10 and width == 10 { return int(1) }
  if height == 12 and width == 12 { return int(2) }
  if height == 14 and width == 14 { return int(3) }
  if height == 16 and width == 16 { return int(4) }
  if height == 18 and width == 18 { return int(5) }
  if height == 20 and width == 20 { return int(6) }
  if height == 22 and width == 22 { return int(7) }
  if height == 24 and width == 24 { return int(8) }
  if height == 26 and width == 26 { return int(9) }
  if height == 32 and width == 32 { return int(10) }
  if height == 36 and width == 36 { return int(11) }
  if height == 40 and width == 40 { return int(12) }
  if height == 44 and width == 44 { return int(13) }
  if height == 48 and width == 48 { return int(14) }
  if height == 52 and width == 52 { return int(15) }
  if height == 64 and width == 64 { return int(16) }
  if height == 72 and width == 72 { return int(17) }
  if height == 80 and width == 80 { return int(18) }
  if height == 88 and width == 88 { return int(19) }
  if height == 96 and width == 96 { return int(20) }
  if height == 104 and width == 104 { return int(21) }
  if height == 120 and width == 120 { return int(22) }
  if height == 132 and width == 132 { return int(23) }
  if height == 144 and width == 144 { return int(24) }
  if height == 8 and width == 18 { return int(25) }
  if height == 8 and width == 32 { return int(26) }
  if height == 12 and width == 26 { return int(28) }
  if height == 12 and width == 36 { return int(28) }
  if height == 16 and width == 36 { return int(29) }
  if height == 16 and width == 48 { return int(30) }

  // Copied from DMRE table
  if height == 8 and width == 48 { return int(31) }
  if height == 8 and width == 64 { return int(32) }
  if height == 8 and width == 80 { return int(33) }
  if height == 8 and width == 96 { return int(34) }
  if height == 8 and width == 120 { return int(35) }
  if height == 8 and width == 144 { return int(36) }
  if height == 12 and width == 64 { return int(37) }
  if height == 12 and width == 88 { return int(38) }
  if height == 16 and width == 64 { return int(39) }
  if height == 20 and width == 36 { return int(40) }
  if height == 20 and width == 44 { return int(41) }
  if height == 20 and width == 64 { return int(42) }
  if height == 22 and width == 48 { return int(43) }
  if height == 24 and width == 48 { return int(44) }
  if height == 24 and width == 64 { return int(45) }
  if height == 26 and width == 40 { return int(46) }
  if height == 26 and width == 48 { return int(47) }
  if height == 26 and width == 64  { return int(48) }
  panic("Data Matrix with dimensions " + str(width) + "x" + str(height) + " not supported")
}

/// Builds `output_options` value from boolean values.
///
/// - barcode-bind-top (bool): Boundary bar _above_ the symbol and between rows if stacking multiple symbols.
/// - barcode-bind (bool): Boundary bars _above_ and _below_ the symbol and between rows if stacking multiple symbols.
/// - barcode-box (bool): Add a box surrounding the symbol and whitespace.
/// - small-text (bool): Use a smaller font for the Human Readable Text.
/// - bold-text (bool): Embolden the Human Readable Text.
/// - cmyk-colour (bool): Select the CMYK colour space option for Encapsulated PostScript and TIF files.
/// - barcode-dotty-mode (bool): Plot a matrix symbol using dots rather than squares.
/// - gs1-gs-separator (bool): Use GS instead of FNC1 as GS1 separator (Data Matrix only).
/// - barcode-quiet-zones (bool): Add compliant quiet zones (additional to any specified whitespace).
/// - barcode-no-quiet-zones (bool): Disable quiet zones, notably those with defaults.
/// - compliant-height (bool): Warn if height not compliant and use standard height (if any) as default.
///
/// -> int
#let output-options(
  barcode-bind-top: false,
  barcode-bind: false,
  barcode-box: false,
  small-text: false,
  bold-text: false,
  cmyk-colour: false,
  barcode-dotty-mode: false,
  gs1-gs-separator: false,
  barcode-quiet-zones: false,
  barcode-no-quiet-zones: false,
  compliant-height: false,
  eanupc-guard-whitespace: false,
  embed-vector-font: false,
) = {
  let BARCODE_BIND_TOP = int(1);
  let BARCODE_BIND = int(2);
  let BARCODE_BOX = int(4);
  let SMALL_TEXT = int(32);
  let BOLD_TEXT = int(64);
  let CMYK_COLOUR = int(128);
  let BARCODE_DOTTY_MODE = int(256);
  let GS1_GS_SEPARATOR = int(512);
  let OUT_BUFFER_INTERMEDIATE = int(1024); // Doesn't make sense in context of tiaoma
  let BARCODE_QUIET_ZONES = int(2048);
  let BARCODE_NO_QUIET_ZONES = int(4096);
  let COMPLIANT_HEIGHT = int(8192);
  let EANUPC_GUARD_WHITESPACE = int(16384); // TODO: Missing documentation
  let EMBED_VECTOR_FONT = int(32768); // TODO: Missing documentation

  // TODO: Replace addition with int.bit-or once it's released
  let result = int(0)
  if barcode-bind-top {
    result += BARCODE_BIND_TOP
  }
  if barcode-bind {
    result += BARCODE_BIND
  }
  if barcode-box {
    result += BARCODE_BOX
  }
  if small-text {
    result += SMALL_TEXT
  }
  if bold-text {
    result += BOLD_TEXT
  }
  if cmyk-colour {
    result += CMYK_COLOUR
  }
  if barcode-dotty-mode {
    result += BARCODE_DOTTY_MODE
  }
  if gs1-gs-separator {
    result += GS1_GS_SEPARATOR
  }
  if out-buffer-intermediate {
    result += OUT_BUFFER_INTERMEDIATE
  }
  if barcode-quiet-zones {
    result += BARCODE_QUIET_ZONES
  }
  if barcode-no-quiet-zones {
    result += BARCODE_NO_QUIET_ZONES
  }
  if compliant-height {
    result += COMPLIANT_HEIGHT
  }
  if eanupc-guard-whitespace {
    result += EANUPC_GUARD_WHITESPACE
  }
  if embed-vector-font {
    result += EMBED_VECTOR_FONT
  }
  return result
}

/// Builds `input_mode` value from boolean values.
///
/// Note that `data-mode`, `unicode-mode`, `gs1-mode` are mutually exclusive and one must be set
///
/// - data-mode (bool): Uses full 8-bit range interpreted as binary data.
/// - unicode-mode (bool): Uses UTF-8 input.
/// - gs1-mode (bool): Encodes GS1 data using FNC1 characters.
///
/// - escape-mode (bool): Process input data for escape sequences.
/// - gs1parens-mode (bool): Parentheses (round brackets) used in GS1 data instead of square brackets to delimit Application Identifiers (parentheses must not otherwise occur in the data).
/// - gs1nocheck-mode (bool): Do not check GS1 data for validity, i.e. suppress checks for valid AIs and data lengths. Invalid characters (e.g. control characters, extended ASCII characters) are still checked for.
/// - heightperrow-mode (bool): Interpret the `height` variable as per-row rather than as overall height.
/// - fast-mode (bool): Use faster if less optimal encodation for symbologies that support it (currently Data Matrix only).
/// -> int
#let input-mode(
  data-mode: false,
  unicode-mode: false,
  gs1-mode: false,
  escape-mode: false,
  gs1parens-mode: false,
  gs1nocheck-mode: false,
  heightperrow-mode: false,
  fast-mode: false,
  extra-escape-mode: false,
) = {
  let DATA_MODE = int(0);
  let UNICODE_MODE = int(1);
  let GS1_MODE = int(2);
  let ESCAPE_MODE = int(8);
  let GS1PARENS_MODE = int(16);
  let GS1NOCHECK_MODE = int(32);
  let HEIGHTPERROW_MODE = int(64);
  let FAST_MODE = int(128);
  let EXTRA_ESCAPE_MODE = int(256); // TODO: Missing documentation

  // TODO: Replace addition with int.bit-or once it's released
  let result = int(0)
  if data-mode {
    // pass (0 is default)
  } else if unicode-mode {
    result += UNICODE_MODE
  } else if gs1-mode {
    result += GS1_MODE
  }

  if escape-mode {
    result += ESCAPE_MODE
  }
  if gs1parens-mode {
    result += GS1PARENS_MODE
  }
  if gs1nocheck-mode {
    result += GS1NOCHECK_MODE
  }
  if heightperrow-mode {
    result += HEIGHTPERROW_MODE
  }
  if fast-mode {
    result += FAST_MODE
  }
  if extra-escape-mode {
    result += EXTRA_ESCAPE_MODE
  }
  return result
}

#let code11(data, options: (:), ..args) = barcode(data, "Code11", options: options, ..args)
#let c25-standard(data, options: (:), ..args) = barcode(data, "C25Standard", options: options, ..args)
#let c25-matrix(data, options: (:), ..args) = barcode(data, "C25Matrix", options: options, ..args)
#let c25-inter(data, options: (:), ..args) = barcode(data, "C25Inter", options: options, ..args)
#let c25-iata(data, options: (:), ..args) = barcode(data, "C25IATA", options: options, ..args)
#let c25-logic(data, options: (:), ..args) = barcode(data, "C25Logic", options: options, ..args)
#let c25-ind(data, options: (:), ..args) = barcode(data, "C25Ind", options: options, ..args)
#let code39(data, options: (:), ..args) = barcode(data, "Code39", options: options, ..args)
#let ex-code39(data, options: (:), ..args) = barcode(data, "ExCode39", options: options, ..args)
#let eanx(data, options: (:), ..args) = barcode(data, "EANX", options: options, ..args)
#let ean(data, options: (:), ..args) = barcode(data, "EANXChk", options: options, ..args)
#let gs1-128(data, options: (:), ..args) = barcode(data, "GS1128", options: options, ..args)
#let ean128(data, options: (:), ..args) = barcode(data, "EAN128", options: options, ..args)
#let codabar(data, options: (:), ..args) = barcode(data, "Codabar", options: options, ..args)
#let code128(data, options: (:), ..args) = barcode(data, "Code128", options: options, ..args)
#let dp-leitcode(data, options: (:), ..args) = barcode(data, "DPLEIT", options: options, ..args)
#let dp-ident(data, options: (:), ..args) = barcode(data, "DPIDENT", options: options, ..args)
#let code16k(data, options: (:), ..args) = barcode(data, "Code16k", options: options, ..args)
#let code49(data, options: (:), ..args) = barcode(data, "Code49", options: options, ..args)
#let code93(data, options: (:), ..args) = barcode(data, "Code93", options: options, ..args)
#let flat(data, options: (:), ..args) = barcode(data, "Flat", options: options, ..args)
#let dbar-omn(data, options: (:), ..args) = barcode(data, "DBarOmn", options: options, ..args)
#let rss14(data, options: (:), ..args) = barcode(data, "RSS14", options: options, ..args)
#let dbar-ltd(data, options: (:), ..args) = barcode(data, "DBarLtd", options: options, ..args)
#let rss-ltd(data, options: (:), ..args) = barcode(data, "RSSLtd", options: options, ..args)
#let dbar-exp(data, options: (:), ..args) = barcode(data, "DBarExp", options: options, ..args)
#let rss-exp(data, options: (:), ..args) = barcode(data, "RSSExp", options: options, ..args)
#let telepen(data, options: (:), ..args) = barcode(data, "Telepen", options: options, ..args)
#let upca(data, options: (:), ..args) = barcode(data, "UPCA", options: options, ..args)
#let upca-chk(data, options: (:), ..args) = barcode(data, "UPCAChk", options: options, ..args)
#let upce(data, options: (:), ..args) = barcode(data, "UPCE", options: options, ..args)
#let upce-chk(data, options: (:), ..args) = barcode(data, "UPCEChk", options: options, ..args)
#let postnet(data, options: (:), ..args) = barcode(data, "Postnet", options: options, ..args)
#let msi-plessey(data, options: (:), ..args) = barcode(data, "MSIPlessey", options: options, ..args)
#let fim(data, options: (:), ..args) = barcode(data, "FIM", options: options, ..args)
#let logmars(data, options: (:), ..args) = barcode(data, "Logmars", options: options, ..args)
#let pharma(data, options: (:), ..args) = barcode(data, "Pharma", options: options, ..args)
#let pzn(data, options: (:), ..args) = barcode(data, "PZN", options: options, ..args)
#let pharma-two(data, options: (:), ..args) = barcode(data, "PharmaTwo", options: options, ..args)
#let cepnet(data, options: (:), ..args) = barcode(data, "CEPNet", options: options, ..args)
#let pdf417(data, options: (:), ..args) = barcode(data, "PDF417", options: options, ..args)
#let pdf417-comp(data, options: (:), ..args) = barcode(data, "PDF417Comp", options: options, ..args)
#let pdf417-trunc(data, options: (:), ..args) = barcode(data, "PDF417Trunc", options: options, ..args)
#let maxicode(data, options: (:), ..args) = barcode(data, "MaxiCode", options: options, ..args)
#let qrcode(data, options: (:), ..args) = barcode(data, "QRCode", options: options, ..args)
#let code128ab(data, options: (:), ..args) = barcode(data, "Code128AB", options: options, ..args)
#let code128b(data, options: (:), ..args) = barcode(data, "Code128B", options: options, ..args)
#let aus-post(data, options: (:), ..args) = barcode(data, "AusPost", options: options, ..args)
#let aus-reply(data, options: (:), ..args) = barcode(data, "AusReply", options: options, ..args)
#let aus-route(data, options: (:), ..args) = barcode(data, "AusRoute", options: options, ..args)
#let aus-redirect(data, options: (:), ..args) = barcode(data, "AusRedirect", options: options, ..args)
#let isbnx(data, options: (:), ..args) = barcode(data, "ISBNX", options: options, ..args)
#let rm4scc(data, options: (:), ..args) = barcode(data, "RM4SCC", options: options, ..args)
#let data-matrix(data, options: (:), ..args) = barcode(data, "DataMatrix", options: options, ..args)
#let ean14(data, options: (:), ..args) = barcode(data, "EAN14", options: options, ..args)
#let vin(data, options: (:), ..args) = barcode(data, "VIN", options: options, ..args)
#let codablock-f(data, options: (:), ..args) = barcode(data, "CodablockF", options: options, ..args)
#let nve18(data, options: (:), ..args) = barcode(data, "NVE18", options: options, ..args)
#let japan-post(data, options: (:), ..args) = barcode(data, "JapanPost", options: options, ..args)
#let korea-post(data, options: (:), ..args) = barcode(data, "KoreaPost", options: options, ..args)
#let dbar-stk(data, options: (:), ..args) = barcode(data, "DBarStk", options: options, ..args)
#let rss14-stack(data, options: (:), ..args) = barcode(data, "RSS14Stack", options: options, ..args)
#let dbar-omn-stk(data, options: (:), ..args) = barcode(data, "DBarOmnStk", options: options, ..args)
#let rss14-stack-omni(data, options: (:), ..args) = barcode(data, "RSS14StackOmni", options: options, ..args)
#let dbar-exp-stk(data, options: (:), ..args) = barcode(data, "DBarExpStk", options: options, ..args)
#let rss-exp-stack(data, options: (:), ..args) = barcode(data, "RSSExpStack", options: options, ..args)
#let planet(data, options: (:), ..args) = barcode(data, "Planet", options: options, ..args)
#let micro-pdf417(data, options: (:), ..args) = barcode(data, "MicroPDF417", options: options, ..args)
#let uspsi-mail(data, options: (:), ..args) = barcode(data, "USPSIMail", options: options, ..args)
#let onecode(data, options: (:), ..args) = barcode(data, "OneCode", options: options, ..args)
#let plessey(data, options: (:), ..args) = barcode(data, "Plessey", options: options, ..args)
#let telepen-num(data, options: (:), ..args) = barcode(data, "TelepenNum", options: options, ..args)
#let itf14(data, options: (:), ..args) = barcode(data, "ITF14", options: options, ..args)
#let kix(data, options: (:), ..args) = barcode(data, "KIX", options: options, ..args)
#let aztec(data, options: (:), ..args) = barcode(data, "Aztec", options: options, ..args)
#let daft(data, options: (:), ..args) = barcode(data, "DAFT", options: options, ..args)
#let dpd(data, options: (:), ..args) = barcode(data, "DPD", options: options, ..args)
#let micro-qr(data, options: (:), ..args) = barcode(data, "MicroQR", options: options, ..args)
#let hibc-128(data, options: (:), ..args) = barcode(data, "HIBC128", options: options, ..args)
#let hibc-39(data, options: (:), ..args) = barcode(data, "HIBC39", options: options, ..args)
#let hibc-dm(data, options: (:), ..args) = barcode(data, "HIBCDM", options: options, ..args)
#let hibc-qr(data, options: (:), ..args) = barcode(data, "HIBCQR", options: options, ..args)
#let hibc-pdf(data, options: (:), ..args) = barcode(data, "HIBCPDF", options: options, ..args)
#let hibc-mic-pdf(data, options: (:), ..args) = barcode(data, "HIBCMicPDF", options: options, ..args)
#let hibc-codablock-f(data, options: (:), ..args) = barcode(data, "HIBCCodablockF", options: options, ..args)
#let hibc-aztec(data, options: (:), ..args) = barcode(data, "HIBCAztec", options: options, ..args)
#let dotcode(data, options: (:), ..args) = barcode(data, "DotCode", options: options, ..args)
#let hanxin(data, options: (:), ..args) = barcode(data, "HanXin", options: options, ..args)
#let upus10(data, options: (:), ..args) = barcode(data, "UPUS10", options: options, ..args)
#let mailmark-4s(data, options: (:), ..args) = barcode(data, "Mailmark4S", options: options, ..args)
// #let mailmark(data, options: (:), ..args) = barcode(data, "Mailmark", options: options, ..args) // internal?
#let azrune(data, options: (:), ..args) = barcode(data, "AzRune", options: options, ..args)
#let code32(data, options: (:), ..args) = barcode(data, "Code32", options: options, ..args)
#let channel(data, options: (:), ..args) = barcode(data, "Channel", options: options, ..args)
#let code-one(data, options: (:), ..args) = barcode(data, "CodeOne", options: options, ..args)
#let grid-matrix(data, options: (:), ..args) = barcode(data, "GridMatrix", options: options, ..args)
#let upnqr(data, options: (:), ..args) = barcode(data, "UPNQR", options: options, ..args)
#let ultra(data, options: (:), ..args) = barcode(data, "Ultra", options: options, ..args)
#let rmqr(data, options: (:), ..args) = barcode(data, "RMQR", options: options, ..args)
#let bc412(data, options: (:), ..args) = barcode(data, "BC412", options: options, ..args)

#let mailmark-2d(height, width, data, options: (:), ..args) = barcode(data, "Mailmark2D", options: (
  option_2: dm-size(height, width),
  ..options,
), ..args)

#let barcode-primary(primary, data, type, options: (:), ..args) = barcode(
  data, type,
  options: (
    primary: primary,
    ..options
  ),
  ..args
)

#let barcode-composite(primary, data, mode, type, options: (:), ..args) = barcode-primary(
  primary, data, type,
  options: (
    option_1: int(mode),
    ..options
  ),
  ..args
)
