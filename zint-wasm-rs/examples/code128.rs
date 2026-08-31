use zint_wasm_rs::{
    options::{symbology::Symbology, Options},
    symbol::Symbol,
};

pub fn main() {
    let encoded_text = "A12345B";
    let options = Options::with_symbology(Symbology::Code128);
    let symbol = match Symbol::new(&options) {
        Ok(symbol) => symbol,
        Err(err) => {
            println!("{:#?}", err);
            return;
        }
    };
    match symbol.encode_svg(encoded_text.as_bytes(), 0) {
        Ok(svg) => println!("{}", svg),
        Err(err) => println!("{:#?}", err),
    }
}
