use wasm_minimal_protocol::*;
use zint_wasm_rs::{options::Options, symbol::Symbol};

initiate_protocol!();

/// The protocol macro declares the two typst host functions unconditionally,
/// so anything linking this crate for the host needs them to exist. Only the
/// tests do, and they never call `gen_with_options`, which is the sole caller.
///
/// The GNU linker gets away without these, because it leaves an archive member
/// nothing references unpacked; `link.exe` does not, and fails the test binary.
#[cfg(not(target_arch = "wasm32"))]
mod host_protocol_stubs {
    #[no_mangle]
    extern "C" fn wasm_minimal_protocol_send_result_to_host(_ptr: *const u8, _len: usize) {
        unreachable!("the typst protocol is only available under wasm")
    }

    #[no_mangle]
    extern "C" fn wasm_minimal_protocol_write_args_to_buffer(_ptr: *mut u8) {
        unreachable!("the typst protocol is only available under wasm")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("provided invalid options: {0}")]
    BadOptions(
        #[from]
        #[source]
        ciborium::de::Error<std::io::Error>,
    ),
    #[error(transparent)]
    ZintEncoding(#[from] zint_wasm_rs::error::Error),
    /// The plugin returns shapes without colour and `lib.typ` paints them,
    /// so a colour given to zint would be silently ignored.
    #[error(
        "the colours are drawn by Typst, not by Zint; give `fill` and `background` to \
         the barcode instead of a colour option"
    )]
    ColourInOptions,
    #[error("could not encode the shapes for Typst: {0}")]
    Shapes(
        #[from]
        #[source]
        ciborium::ser::Error<std::io::Error>,
    ),
}
type Result<T> = std::result::Result<T, crate::Error>;

/// Encodes `text` with `options` and returns, as CBOR, the shapes zint lays
/// the symbol out as. `lib.typ` draws them with Typst's own primitives, which
/// is what lets a document's colours reach the PDF as they were given.
#[wasm_func]
pub fn gen_with_options(options: &[u8], text: &[u8]) -> Result<Vec<u8>> {
    let options: Options = ciborium::from_reader(options)?;
    if options.fg_color.is_some() || options.bg_color.is_some() {
        return Err(Error::ColourInOptions);
    }
    // The payload travels as bytes and is handed to zint as bytes: Typst's
    // `bytes` may hold anything, and a symbology in DATA mode is meant to take
    // it.
    let vector = Symbol::new(&options)?.encode_vector(text, 0)?;
    let mut encoded = Vec::new();
    ciborium::into_writer(&vector, &mut encoded)?;
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::{gen_with_options, Error};
    use ciborium::{cbor, Value};
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use zint_wasm_rs::vector::Ink;

    /// Encodes options the way Typst's `cbor.encode` does, which is the only
    /// way this function is ever called.
    ///
    /// Input:  `cbor!({"symbology" => "Code128"})`
    /// Output: the CBOR bytes `gen_with_options` receives as its first argument
    fn options(value: Value) -> Vec<u8> {
        let mut encoded = Vec::new();
        ciborium::into_writer(&value, &mut encoded).expect("CBOR value is encodable");
        encoded
    }

    /// The plugin's answer, read the way `lib.typ` reads it: these are the
    /// fields it uses, and `deny_unknown_fields` makes a renamed or added one
    /// fail here before it fails a document.
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Drawing {
        width: f64,
        height: f64,
        rectangles: Vec<Rectangle>,
        hexagons: Vec<Hexagon>,
        circles: Vec<Circle>,
        texts: Vec<Text>,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        ink: String,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Hexagon {
        x: f64,
        y: f64,
        diameter: f64,
        rotation: i64,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Circle {
        x: f64,
        y: f64,
        diameter: f64,
        stroke: f64,
        background: bool,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Text {
        x: f64,
        y: f64,
        size: f64,
        width: f64,
        rotation: i64,
        align: String,
        text: String,
        bold: bool,
    }

    impl Drawing {
        fn size(&self) -> (f64, f64) {
            (self.width, self.height)
        }

        fn texts(&self) -> Vec<&str> {
            self.texts.iter().map(|it| it.text.as_str()).collect()
        }
    }

    fn drawing(options: Vec<u8>, text: &[u8]) -> Drawing {
        let rendered = gen_with_options(&options, text)
            .unwrap_or_else(|error| panic!("the plugin should have drawn a barcode: {error}"));
        ciborium::from_reader(rendered.as_slice()).unwrap_or_else(|error| {
            panic!("the plugin answered something lib.typ cannot read: {error}")
        })
    }

    #[test]
    fn a_barcode_comes_back_as_the_shapes_to_draw() {
        let drawn = drawing(
            options(cbor!({"symbology" => "Code128"}).unwrap()),
            b"A12345B",
        );

        assert_eq!(drawn.width, 224.0);
        assert!(!drawn.rectangles.is_empty(), "the bars are there");
        assert!(drawn.rectangles.iter().all(|it| it.ink == "foreground"));
        assert_eq!(drawn.texts(), ["A12345B"]);
    }

    /// Ultracode is the one symbology that draws in colours of its own, and
    /// the names it uses for them are the ones `lib.typ` looks up.
    #[test]
    fn ultracode_names_its_inks() {
        let drawn = drawing(
            options(cbor!({"symbology" => "Ultra"}).unwrap()),
            b"https://aimglobal.org/jcrv3tX",
        );
        let inks: std::collections::BTreeSet<&str> =
            drawn.rectangles.iter().map(|it| it.ink.as_str()).collect();

        assert_eq!(
            inks,
            ["black", "cyan", "green", "magenta", "white", "yellow"].into(),
            "unexpected inks"
        );
    }

    /// Every ink Ultracode can name, with an exhaustive match so that one
    /// added to [`Ink`] cannot be left out.
    const INKS: [Ink; 9] = [
        Ink::Foreground,
        Ink::Cyan,
        Ink::Blue,
        Ink::Magenta,
        Ink::Red,
        Ink::Yellow,
        Ink::Green,
        Ink::Black,
        Ink::White,
    ];

    #[allow(dead_code)]
    fn every_ink_is_listed(ink: Ink) {
        match ink {
            Ink::Foreground
            | Ink::Cyan
            | Ink::Blue
            | Ink::Magenta
            | Ink::Red
            | Ink::Yellow
            | Ink::Green
            | Ink::Black
            | Ink::White => (),
        }
    }

    /// The names `_inks` in `lib.typ` gives a colour to.
    ///
    /// Input:  the lines
    ///         `#let _inks = (`
    ///         `  cyan: cmyk(100%, 0%, 0%, 0%),`
    ///         `)`
    /// Output: `["cyan"]`
    fn parse_inks(lib: &str) -> Vec<&str> {
        lib.lines()
            .map(str::trim)
            .skip_while(|line| *line != "#let _inks = (")
            .skip(1)
            .take_while(|line| *line != ")")
            .map(|line| {
                line.split_once(':')
                    .unwrap_or_else(|| panic!("not an ink: {line}"))
                    .0
            })
            .collect()
    }

    /// `lib.typ` looks up the colour of every ink but the foreground by name,
    /// and fails the document on one it does not know. The names are written
    /// twice, once by serde here and once by hand there, so this is what keeps
    /// them the same.
    #[test]
    fn every_ink_has_a_colour_in_lib_typ() {
        const LIB: &str = include_str!("../../typst-package/lib.typ");

        let mut named: Vec<String> = INKS
            .iter()
            .filter(|ink| **ink != Ink::Foreground)
            .map(|ink| match Value::serialized(ink) {
                Ok(Value::Text(name)) => name,
                other => panic!("{ink:?} is not serialized as a name: {other:?}"),
            })
            .collect();
        named.sort();
        let mut coloured = parse_inks(LIB);
        coloured.sort();

        assert_eq!(coloured, named);
    }

    /// Git hands a Windows checkout CRLF unless something pins the file, and
    /// nothing pins `lib.typ`, so the same source has to parse either way.
    #[test]
    fn the_inks_parse_whatever_line_endings_the_checkout_used() {
        const LIB: &str = include_str!("../../typst-package/lib.typ");

        let unix = LIB.replace("\r\n", "\n");
        let windows = unix.replace('\n', "\r\n");

        assert!(!parse_inks(&unix).is_empty(), "no inks were parsed at all");
        assert_eq!(parse_inks(&unix), parse_inks(&windows));
    }

    /// `lib.typ` reads every spelling of the colour options as `fill` or
    /// `background` and sends none of them. One that arrives anyway would set
    /// a colour in zint that the shapes do not carry, so it is refused rather
    /// than the barcode quietly coming out black.
    #[test]
    fn a_colour_option_is_refused_because_typst_draws_the_colours() {
        for key in [
            "fg-color",
            "fg-colour",
            "stroke",
            "bg-color",
            "bg-colour",
            "fill",
        ] {
            let error = gen_with_options(
                &options(cbor!({"symbology" => "Code128", key => "1F4788"}).unwrap()),
                b"A12345B",
            )
            .expect_err("the plugin does not draw colours");

            assert!(
                matches!(error, Error::ColourInOptions),
                "{key}: unexpected error: {error:?}"
            );
        }
    }

    /// Typst spells the option names with dashes, so this is the shape the
    /// plugin actually receives.
    #[test]
    fn the_option_names_are_the_ones_typst_sends() {
        let drawn = drawing(
            options(
                cbor!({
                    "symbology" => "Code128",
                    "show-hrt" => false,
                })
                .unwrap(),
            ),
            b"A12345B",
        );

        assert!(drawn.texts.is_empty(), "the text was switched off");
    }

    /// A Typst `bytes` may hold anything, and the payload used to be read as
    /// text on the way in, so a byte that is not UTF-8 took the whole plugin
    /// down with it.
    ///
    /// Input:  `bytes((0xFF, 0xFE, 0x00, 0x41))` in DATA input mode
    /// Output: a QR code carrying those four bytes
    #[test]
    fn a_payload_that_is_not_text_is_encoded() {
        let drawn = drawing(
            options(cbor!({"symbology" => "QRCode", "input-mode" => 0}).unwrap()),
            &[0xFF, 0xFE, 0x00, 0x41],
        );

        assert!(!drawn.rectangles.is_empty());
    }

    /// The payload used to be handed over as a C string, which ends at the
    /// first NUL. Everything after it has to be encoded too.
    #[test]
    fn a_nul_byte_is_part_of_the_payload() {
        let truncated = drawing(
            options(cbor!({"symbology" => "QRCode", "input-mode" => 0}).unwrap()),
            b"hz",
        );
        let whole = drawing(
            options(cbor!({"symbology" => "QRCode", "input-mode" => 0}).unwrap()),
            b"hz\0bl",
        );

        assert_ne!(
            truncated, whole,
            "the bytes after the NUL did not reach zint"
        );
    }

    /// An empty payload is zint's to refuse, and it has to arrive there as an
    /// error rather than as a read of an address that holds nothing.
    #[test]
    fn an_empty_payload_is_reported_as_an_encoding_error() {
        let error = gen_with_options(&options(cbor!({"symbology" => "QRCode"}).unwrap()), b"")
            .expect_err("there is nothing to encode");

        assert!(
            matches!(error, Error::ZintEncoding(_)),
            "unexpected error: {error:?}"
        );
    }

    /// Zint keeps `primary` in 128 bytes; a longer one used to take the plugin
    /// down instead of being reported.
    #[test]
    fn a_primary_that_does_not_fit_is_reported() {
        let error = gen_with_options(
            &options(
                cbor!({
                    "symbology" => "EANXCC",
                    "primary" => "3".repeat(200),
                })
                .unwrap(),
            ),
            b"[99]1234-abcd",
        )
        .expect_err("zint has no room for a primary that long");

        assert!(
            matches!(error, Error::ZintEncoding(_)),
            "unexpected error: {error:?}"
        );
        assert!(
            error.to_string().contains("primary"),
            "the error should name the option: {error}"
        );
    }

    /// Anything zint refuses has to reach the document as an error, not as an
    /// empty or broken image.
    #[test]
    fn a_symbol_zint_refuses_is_reported_as_an_encoding_error() {
        let error = gen_with_options(
            &options(cbor!({"symbology" => "EANXChk"}).unwrap()),
            b"6975004310002",
        )
        .expect_err("the check digit does not match");

        assert!(
            matches!(error, Error::ZintEncoding(_)),
            "unexpected error: {error:?}"
        );
        // What the document is shown is zint's own sentence about the payload
        // it was given, which is more than the return code alone can say.
        let message = error.to_string();
        assert_ne!(
            message,
            zint_wasm_rs::error::ZintError::InvalidCheck.to_string(),
            "only the meaning of the return code crossed the plugin boundary"
        );
        assert!(
            message.contains("check digit"),
            "unexpected explanation: {message}"
        );
    }

    #[test]
    fn an_unknown_symbology_is_reported_as_bad_options() {
        let error = gen_with_options(
            &options(cbor!({"symbology" => "Code129"}).unwrap()),
            b"A12345B",
        )
        .expect_err("Code129 does not exist");

        assert!(
            matches!(error, Error::BadOptions(_)),
            "unexpected error: {error:?}"
        );
    }

    /// A misspelled key used to be dropped, which left the document with a
    /// barcode that had quietly ignored what it was told.
    #[test]
    fn a_misspelled_option_is_reported_rather_than_ignored() {
        let error = gen_with_options(
            &options(cbor!({"symbology" => "Code128", "show_hrt" => false}).unwrap()),
            b"A12345B",
        )
        .expect_err("the option is spelled `show-hrt`");

        assert!(
            matches!(error, Error::BadOptions(_)),
            "unexpected error: {error:?}"
        );
        assert!(
            error.to_string().contains("show_hrt"),
            "the error should name the key that was not understood: {error}"
        );
    }

    /// `mailmark-2d` and `barcode-composite` in `lib.typ` are the only helpers
    /// that set an option themselves, and both do it by number. What they ask
    /// for has to reach zint, or the symbol is quietly drawn the way zint would
    /// have drawn it anyway.
    #[test]
    fn the_numbered_options_reach_zint() {
        // The Mailmark 2D example from the manual, spaces and all.
        let default_size = drawing(
            options(cbor!({"symbology" => "DataMatrix"}).unwrap()),
            b"12345",
        );
        // 16 is what `dm-size(64, 64)` returns, far larger than this payload
        // needs, so zint would never have chosen it on its own.
        let asked_for = drawing(
            options(cbor!({"symbology" => "DataMatrix", "option-2" => 16}).unwrap()),
            b"12345",
        );

        assert_ne!(
            default_size.size(),
            asked_for.size(),
            "the requested Data Matrix size did not reach zint"
        );

        let chosen_by_zint = drawing(
            options(cbor!({"symbology" => "EANXCC", "primary" => "331234567890"}).unwrap()),
            b"[99]1234-abcd",
        );
        // CC-B, where zint picks CC-A for a message this short.
        let cc_b = drawing(
            options(
                cbor!({
                    "symbology" => "EANXCC",
                    "primary" => "331234567890",
                    "option-1" => 2,
                })
                .unwrap(),
            ),
            b"[99]1234-abcd",
        );

        assert_ne!(
            chosen_by_zint, cc_b,
            "the requested composite mode did not reach zint"
        );
    }

    /// A fixed Data Matrix size is an `option-2` value, but `dm-size` reads as
    /// though it were an `option-3` one, so documents pass it there. The error
    /// is where that gets sorted out.
    ///
    /// Input:  `option-3: dm-size(12, 36)`, which is 28
    /// Output: an error naming `option-2`, rather than a symbol of some other
    ///         size or a rejection that says only that 28 is wrong
    #[test]
    fn a_data_matrix_size_given_to_option_3_says_where_it_belongs() {
        let error = gen_with_options(
            &options(cbor!({"symbology" => "DataMatrix", "option-3" => 28}).unwrap()),
            b"A12345B",
        )
        .expect_err("a size is not an option-3 value");

        assert!(
            matches!(error, Error::BadOptions(_)),
            "unexpected error: {error:?}"
        );
        assert!(
            error.to_string().contains("option-2"),
            "the error should say where the size belongs: {error}"
        );

        let rendered = drawing(
            options(cbor!({"symbology" => "DataMatrix", "option-2" => 28}).unwrap()),
            b"A12345B",
        );

        assert_eq!(
            rendered.size(),
            (72.0, 24.0),
            "option-2 draws the 12x36 symbol that was asked for"
        );
    }

    /// Every entry of `dm-size`, against the symbol zint actually draws for it.
    ///
    /// The table in `lib.typ` is a hand-copied transcription of zint's size
    /// table, so nothing but a check like this one says whether an entry names
    /// the size it claims. A wrong entry is silent: the document asks for one
    /// size and gets another that encodes just as happily.
    #[test]
    fn every_data_matrix_size_draws_the_symbol_it_names() {
        // Data Matrix draws two units per module, and the default output
        // carries no quiet zone, so the drawing is exactly the symbol. A change
        // in either would fail every entry at once rather than one of them,
        // which is what tells the two apart.
        const UNITS_PER_MODULE: f64 = 2.0;
        // Short enough for the 3 codewords of the smallest symbol; every larger
        // one pads.
        const DATA: &[u8] = b"1";

        let table = dm_size_table();
        for &(height, width, option_2) in &table {
            let rendered = drawing(
                options(cbor!({"symbology" => "DataMatrix", "option-2" => option_2}).unwrap()),
                DATA,
            );

            assert_eq!(
                rendered.size(),
                (
                    f64::from(width) * UNITS_PER_MODULE,
                    f64::from(height) * UNITS_PER_MODULE,
                ),
                "dm-size({height}, {width}) returns {option_2}, which draws something else"
            );
        }
    }

    /// `option_2` indexes zint's size table from 1, so the values `dm-size`
    /// returns have to be every number from 1 to the length of the table, each
    /// once. A repeat means one size names another's entry, and a gap means a
    /// size zint supports cannot be asked for at all; both are what a hand
    /// transcription gets wrong, and neither is visible in a rendered symbol.
    #[test]
    fn the_data_matrix_sizes_use_every_option_2_value_once() {
        let table = dm_size_table();
        let mut by_value: BTreeMap<u32, (u32, u32)> = BTreeMap::new();

        for &(height, width, option_2) in &table {
            if let Some((other_height, other_width)) = by_value.insert(option_2, (height, width)) {
                panic!(
                    "dm-size returns {option_2} for both {other_height}x{other_width} \
                     and {height}x{width}"
                );
            }
        }

        let expected: Vec<u32> = (1..=table.len() as u32).collect();
        let actual: Vec<u32> = by_value.keys().copied().collect();
        assert_eq!(
            actual,
            expected,
            "the {} sizes should cover option-2 1 to {} with no gaps",
            table.len(),
            table.len()
        );
    }

    /// The `(height, width, option-2)` entries of `dm-size` in `lib.typ`.
    ///
    /// Input:  the lines
    ///         `  if height == 12 and width == 26 {`
    ///         `    return int(27)`
    /// Output: the entry `(12, 26, 27)`
    fn dm_size_table() -> Vec<(u32, u32, u32)> {
        parse_dm_size_table(include_str!("../../typst-package/lib.typ"))
    }

    fn parse_dm_size_table(lib: &str) -> Vec<(u32, u32, u32)> {
        let (_, body) = lib
            .split_once("#let dm-size(")
            .expect("lib.typ defines dm-size");

        let mut entries = Vec::new();
        let mut size = None;
        // The arms are indented, so a closing brace of its own ends the
        // function. This walks the lines rather than slicing on a newline
        // because the file's line endings are whatever the checkout produced,
        // and a Windows one holds no `\n}\n` at all.
        let body = body
            .lines()
            .map(str::trim_end)
            .take_while(|line| *line != "}");
        for line in body.map(str::trim_start) {
            if let Some(condition) = line.strip_prefix("if height == ") {
                let (height, width) = condition
                    .strip_suffix(" {")
                    .and_then(|condition| condition.split_once(" and width == "))
                    .unwrap_or_else(|| panic!("unexpected condition in dm-size: {line}"));
                let previous = size.replace((number(height), number(width)));
                assert!(
                    previous.is_none(),
                    "a size is named by no return before: {line}"
                );
            } else if let Some(value) = line.strip_prefix("return int(") {
                let (height, width) = size
                    .take()
                    .unwrap_or_else(|| panic!("a size is returned by no condition: {line}"));
                let value = value
                    .strip_suffix(')')
                    .unwrap_or_else(|| panic!("unexpected return in dm-size: {line}"));
                entries.push((height, width, number(value)));
            }
        }

        assert!(size.is_none(), "dm-size ends on a condition with no return");
        assert!(!entries.is_empty(), "dm-size returns no sizes at all");
        entries
    }

    /// A decimal integer out of the table, so a non-numeric one names itself
    /// rather than failing as a parse somewhere up the call stack.
    ///
    /// Input:  `"27"`
    /// Output: `27`
    fn number(text: &str) -> u32 {
        text.parse()
            .unwrap_or_else(|error| panic!("{text} is not a number: {error}"))
    }

    /// Git hands a Windows checkout CRLF unless something pins the file, and
    /// nothing pins `lib.typ`, so the same source has to parse either way.
    #[test]
    fn the_size_table_parses_whatever_line_endings_the_checkout_used() {
        const LIB: &str = include_str!("../../typst-package/lib.typ");

        let unix = LIB.replace("\r\n", "\n");
        let windows = unix.replace('\n', "\r\n");

        assert_eq!(parse_dm_size_table(&unix), parse_dm_size_table(&windows));
        assert_eq!(parse_dm_size_table(&unix), dm_size_table());
    }

    /// A symbol that does not meet its standard renders exactly like one that
    /// does, so a document that cannot ship the first has to be able to ask to
    /// be stopped rather than to be shown it.
    ///
    /// Input:  a Code 39 one X-dimension high, with the height held to the
    ///         standard, which zint answers with a symbol and a warning
    /// Output: nothing at all, and zint's reason for it
    #[test]
    fn a_document_can_ask_for_a_warning_to_stop_it() {
        let too_short = cbor!({
            "symbology" => "Code39",
            "height" => 1.0,
            "output-options" => {"compliant-height" => true},
        })
        .unwrap();

        let rendered = drawing(options(too_short.clone()), b"ASSET-0042189");
        assert!(
            !rendered.rectangles.is_empty(),
            "a warning on its own still draws the symbol"
        );

        let mut strict = too_short;
        strict
            .as_map_mut()
            .expect("the options are a CBOR map")
            .push((Value::from("warn-level"), Value::from("fail-all")));

        let error = gen_with_options(&options(strict), b"ASSET-0042189")
            .expect_err("the height was asked to be compliant and is not");

        let message = error.to_string();
        assert!(
            message.contains("not compliant"),
            "the document should be told which standard was missed: {message}"
        );
    }

    #[test]
    fn options_that_are_not_cbor_are_reported_as_bad_options() {
        let error = gen_with_options(&[0xFF, 0xFF, 0xFF], b"A12345B")
            .expect_err("those bytes are not a CBOR document");

        assert!(
            matches!(error, Error::BadOptions(_)),
            "unexpected error: {error:?}"
        );
    }

    /// The options travel separately from the payload, so a symbology that
    /// needs a second field takes it from the options rather than from the
    /// text.
    #[test]
    fn a_composite_symbol_takes_its_linear_part_from_the_options() {
        let rendered = drawing(
            options(
                cbor!({
                    "symbology" => "EANXCC",
                    "primary" => "331234567890",
                })
                .unwrap(),
            ),
            b"[99]1234-abcd",
        );

        assert!(!rendered.rectangles.is_empty());
    }
}
