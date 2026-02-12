use wasm_minimal_protocol::*;
use zint_rs::{
    output::VectorPlot,
    segment::{ECI, Segment},
    symbol::Symbol,
};

mod serde;

initiate_protocol!();

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("provided invalid options: {0}")]
    BadOptions(
        #[from]
        #[source]
        ciborium::de::Error<std::io::Error>,
    ),
    #[error(transparent)]
    ZintEncoding(#[from] zint_rs::error::Error),
    #[error("invalid UTF-8 in text data")]
    InvalidUtf8,
    #[error("invalid ECI value: {0}")]
    InvalidEci(String),
    #[error("CBOR serialization failed: {0}")]
    Serialization(String),
}
type Result<T> = std::result::Result<T, crate::Error>;

#[wasm_func]
pub fn gen_with_options(options: &[u8], text: &[u8]) -> Result<Vec<u8>> {
    let options: serde::PluginOptions = ciborium::from_reader(options)?;
    let text = std::str::from_utf8(text).map_err(|_| Error::InvalidUtf8)?;

    let eci_value = options.eci;
    let (mut generic_options, display_options, primary_text) = options.into_options();
    // ECI::NONE (value 0) means "no ECI mode" - the barcode library uses its default
    // encoding. This matches the master branch behavior where symbol->eci defaults to 0.
    // Note: ECI value 0 is also CP437 in the ECI spec, but when no ECI is specified
    // by the user, we use ECI::NONE to indicate "no explicit ECI", which is the same
    // numeric value but semantically means "use default encoding".
    let eci = match eci_value {
        Some(val) => ECI::new(val as u32).map_err(|e| Error::InvalidEci(e.to_string()))?,
        None => ECI::NONE,
    };

    // Handle primary message on the stack to avoid .leak() memory leak.
    let primary_bytes: Option<Vec<u8>> = primary_text.map(|s| s.into_bytes());
    if let Some(ref bytes) = primary_bytes
        && !bytes.is_empty()
    {
        generic_options.primary_message = Some(Segment::new(bytes.as_slice(), ECI::NONE));
    }

    let mut symbol =
        Symbol::encode_segments(generic_options, &[Segment::new(text.as_bytes(), eci)])?;
    let vector: VectorPlot = symbol.plot(&display_options)?;
    let mut output = Vec::new();
    ciborium::into_writer(&vector, &mut output).map_err(|e| Error::Serialization(e.to_string()))?;
    Ok(output)
}
