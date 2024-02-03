use zint_wasm_rs::options::symbology::Symbology::Code128;
use zint_wasm_rs::options::Options;
pub fn main() {
    let encoded_text = "A12345B";
    let options = Options::with_symbology(Code128);
    let symbol = options.into_zint_symbol();
    match symbol.encode(encoded_text, 0, 0) {
        Ok(svg) => println!("{}", svg),
        Err(err) => println!("{:#?}", err),
    }
}
