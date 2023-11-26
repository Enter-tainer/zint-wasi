use anyhow::{anyhow, Context, Result};
use wasm_minimal_protocol::*;
use zint_wasm_rs::options::{symbology::Symbology, Options};

initiate_protocol!();

fn gen_code_simple(text: &str, symbology: Symbology) -> Result<Vec<u8>> {
    let options = Options::with_symbology(symbology);
    let symbol = options.to_zint_symbol();
    let svg = symbol
        .encode(text, 0, 0)
        .map_err(|e| anyhow!(format!("{:#?}", e)))?;
    Ok(svg.into_bytes())
}

#[wasm_func]
pub fn gen_with_options(options: &[u8], text: &[u8]) -> Result<Vec<u8>> {
    let options: Options =
        ciborium::from_reader(options).context("Failed to deserialize options")?;
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    let symbol = options.to_zint_symbol();
    let svg = symbol
        .encode(text, 0, 0)
        .map_err(|e| anyhow!(format!("{:#?}", e)))?;
    Ok(svg.into_bytes())
}

#[wasm_func]
pub fn ean_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::EANXChk)
}

#[wasm_func]
pub fn code128_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::Code128)
}

#[wasm_func]
pub fn code39_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::Code39)
}

#[wasm_func]
pub fn upca_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::UPCAChk)
}

#[wasm_func]
pub fn data_matrix_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::DataMatrix)
}

#[wasm_func]
pub fn channel_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::Channel)
}

#[wasm_func]
pub fn msi_plessey_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::MSIPlessey)
}

#[wasm_func]
pub fn micro_pdf417_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::MicroPDF417)
}

#[wasm_func]
pub fn qrcode_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::QRCode)
}

#[wasm_func]
pub fn aztec_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::Aztec)
}

/// Code 16k
#[wasm_func]
pub fn code16k_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::Code16k)
}

/// maxicode
#[wasm_func]
pub fn maxicode_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::MaxiCode)
}

/// Planet Code
#[wasm_func]
pub fn planet_gen(text: &[u8]) -> Result<Vec<u8>> {
    let text = std::str::from_utf8(text).context("invalid utf8")?;
    gen_code_simple(text, Symbology::Planet)
}
