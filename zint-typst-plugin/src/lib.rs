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
}
type Result<T> = std::result::Result<T, crate::Error>;

#[wasm_func]
pub fn gen_with_options(options: &[u8], text: &[u8]) -> Result<Vec<u8>> {
    let options: Options = ciborium::from_reader(options)?;
    // The payload travels as bytes and is handed to zint as bytes: Typst's
    // `bytes` may hold anything, and a symbology in DATA mode is meant to take
    // it.
    let svg = Symbol::new(&options)?.encode_svg(text, 0)?;
    Ok(svg.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::{gen_with_options, Error};
    use ciborium::{cbor, Value};

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

    /// The size of the drawing, in the user units of the SVG root element.
    #[derive(Debug, PartialEq)]
    struct SvgSize {
        width: f64,
        height: f64,
    }

    /// Reads the size off the root element of a rendered SVG. Only the two
    /// attributes are looked at, so the order zint writes them in, the other
    /// attributes it adds and the way it breaks lines do not matter.
    ///
    /// Input:  a document containing
    ///         `<svg width="224" height="117" version="1.1" ...>`
    /// Output: `SvgSize { width: 224.0, height: 117.0 }`
    fn svg_size(document: &str) -> SvgSize {
        let start = document.find("<svg").expect("SVG root element");
        let tag = &document[start..];
        let tag = &tag[..tag.find('>').expect("SVG root element is closed")];

        SvgSize {
            width: attribute(tag, "width"),
            height: attribute(tag, "height"),
        }
    }

    /// The numeric value of one attribute of an element's start tag.
    ///
    /// Input:  `<svg width="224" height="117" version="1.1"`, `height`
    /// Output: `117.0`
    fn attribute(tag: &str, name: &str) -> f64 {
        let value = tag
            .split_whitespace()
            .find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='))
            .unwrap_or_else(|| panic!("the root element has no {name} attribute: {tag}"));
        let value = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .unwrap_or_else(|| panic!("the {name} attribute is not quoted: {value}"));
        value.parse().unwrap_or_else(|error| {
            panic!("the {name} attribute is not a number: {value}: {error}")
        })
    }

    fn svg(options: Vec<u8>, text: &[u8]) -> String {
        let rendered = gen_with_options(&options, text)
            .unwrap_or_else(|error| panic!("the plugin should have rendered a barcode: {error}"));
        String::from_utf8(rendered).expect("zint renders SVG as text")
    }

    #[test]
    fn a_barcode_comes_back_as_an_svg_document() {
        let rendered = svg(
            options(cbor!({"symbology" => "Code128"}).unwrap()),
            b"A12345B",
        );

        assert!(rendered.starts_with("<?xml version=\"1.0\""));
        assert!(rendered.contains("<svg "));
        assert!(rendered.lines().any(|line| line.trim() == "A12345B"));
    }

    /// Typst spells the option names with dashes and names the colors the way
    /// its own drawing functions do, so this is the shape the plugin actually
    /// receives.
    #[test]
    fn the_option_names_are_the_ones_typst_sends() {
        let rendered = svg(
            options(
                cbor!({
                    "symbology" => "Code128",
                    "show-hrt" => false,
                    "stroke" => "#1F4788",
                })
                .unwrap(),
            ),
            b"A12345B",
        );

        assert!(!rendered.contains("<text"), "the text was switched off");
        assert!(
            rendered.contains("#1F4788"),
            "the bars took the chosen color"
        );
    }

    /// A Typst `bytes` may hold anything, and the payload used to be read as
    /// text on the way in, so a byte that is not UTF-8 took the whole plugin
    /// down with it.
    ///
    /// Input:  `bytes((0xFF, 0xFE, 0x00, 0x41))` in DATA input mode
    /// Output: a QR code carrying those four bytes
    #[test]
    fn a_payload_that_is_not_text_is_encoded() {
        let rendered = svg(
            options(cbor!({"symbology" => "QRCode", "input-mode" => 0}).unwrap()),
            &[0xFF, 0xFE, 0x00, 0x41],
        );

        assert!(rendered.contains("<svg "));
    }

    /// The payload used to be handed over as a C string, which ends at the
    /// first NUL. Everything after it has to be encoded too.
    #[test]
    fn a_nul_byte_is_part_of_the_payload() {
        let truncated = svg(
            options(cbor!({"symbology" => "QRCode", "input-mode" => 0}).unwrap()),
            b"hz",
        );
        let whole = svg(
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
        let default_size = svg(
            options(cbor!({"symbology" => "DataMatrix"}).unwrap()),
            b"12345",
        );
        // 16 is what `dm-size(64, 64)` returns, far larger than this payload
        // needs, so zint would never have chosen it on its own.
        let asked_for = svg(
            options(cbor!({"symbology" => "DataMatrix", "option-2" => 16}).unwrap()),
            b"12345",
        );

        assert_ne!(
            svg_size(&default_size),
            svg_size(&asked_for),
            "the requested Data Matrix size did not reach zint"
        );

        let chosen_by_zint = svg(
            options(cbor!({"symbology" => "EANXCC", "primary" => "331234567890"}).unwrap()),
            b"[99]1234-abcd",
        );
        // CC-B, where zint picks CC-A for a message this short.
        let cc_b = svg(
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

        let rendered = svg(
            options(cbor!({"symbology" => "DataMatrix", "option-2" => 28}).unwrap()),
            b"A12345B",
        );

        assert_eq!(
            svg_size(&rendered),
            SvgSize {
                width: 72.0,
                height: 24.0
            },
            "option-2 draws the 12x36 symbol that was asked for"
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
        let rendered = svg(
            options(
                cbor!({
                    "symbology" => "EANXCC",
                    "primary" => "331234567890",
                })
                .unwrap(),
            ),
            b"[99]1234-abcd",
        );

        assert!(rendered.contains("<svg "));
    }
}
