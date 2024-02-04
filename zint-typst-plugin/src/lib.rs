use wasm_minimal_protocol::*;
use zint_wasm_rs::{options::Options, symbol::Symbol};

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
