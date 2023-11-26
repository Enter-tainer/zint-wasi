use zint_wasm_rs::options::Options;
pub fn main() {
    let encoded_text = "6975004310001";
    let options = Options::with_symbology(zint_wasm_rs::options::symbology::Symbology::EANXChk);
    let symbol = options.to_zint_symbol();
    match symbol.encode(encoded_text, 0, 0) {
        Ok(svg) => println!("{}", svg),
        Err(err) => println!("{:#?}", err),
    }
}
