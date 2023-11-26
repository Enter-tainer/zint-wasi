use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use wasm_minimal_protocol::*;
use zint_wasm_rs::options::Options;

initiate_protocol!();

#[wasm_func]
pub fn ean(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    let options = Options::with_symbology(zint_wasm_rs::options::symbology::Symbology::EANXChk);
    let symbol = options.to_zint_symbol();
    let svg = symbol
        .encode(text, 0, 0)
        .map_err(|e| anyhow!(format!("{:#?}", e)))?;
    Ok(svg.into_bytes())
}
