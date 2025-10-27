use wasm_minimal_protocol::*;

use zint_rs::{output::VectorPlot, *};

use crate::serde::decode_options;

mod output;
mod serde;

initiate_protocol!();

#[wasm_func]
pub fn zint_encode(options: &[u8], text: &[u8]) -> Vec<u8> {
    output::pack_result((|| {
        let value: ciborium::Value =
            ciborium::from_reader(options).map_err(|e| error!("invalid options: {e}"))?;
        let options = decode_options(&value)?;
        let mut symbol = match options.format {
            serde::Format::Utf8 => {
                let text = std::str::from_utf8(text)?; // bytes(data) always creates a utf8 slice
                Symbol::encode_utf8(options.generic, text)?
            }
            // TODO: ECI handling is literally just missing deserialization
            serde::Format::ECI => return Err(error!("ECI handling not yet implemented")),
        };
        let plot: VectorPlot = symbol.plot(&options.display)?;
        Ok(plot)
    })())
}
