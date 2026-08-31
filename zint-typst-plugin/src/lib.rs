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
    let text = std::str::from_utf8(text).expect("non-utf8 string"); // bytes(data) always creates a utf8 slice
    let symbol = Symbol::new(&options);
    let svg = symbol.encode_svg(text, 0, 0)?;
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

    fn svg(options: Vec<u8>, text: &str) -> String {
        let rendered = gen_with_options(&options, text.as_bytes())
            .unwrap_or_else(|error| panic!("the plugin should have rendered a barcode: {error}"));
        String::from_utf8(rendered).expect("zint renders SVG as text")
    }

    #[test]
    fn a_barcode_comes_back_as_an_svg_document() {
        let rendered = svg(
            options(cbor!({"symbology" => "Code128"}).unwrap()),
            "A12345B",
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
                    "fg-colour" => "#1F4788",
                })
                .unwrap(),
            ),
            "A12345B",
        );

        assert!(!rendered.contains("<text"), "the text was switched off");
        assert!(
            rendered.contains("#1F4788"),
            "the bars took the chosen color"
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
        assert!(error.to_string().contains("check digit"));
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
            "[99]1234-abcd",
        );

        assert!(rendered.contains("<svg "));
    }
}
