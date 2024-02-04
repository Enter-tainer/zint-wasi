use zint_wasm_rs::{
    options::{symbology::Symbology, Options},
    symbol::Symbol,
};

pub fn main() {
    let encoded_text = "6975004310001";
    let options = Options::with_symbology(Symbology::EANXChk);
    let symbol = Symbol::new(&options);
    match symbol.encode_svg(encoded_text, 0, 0) {
        Ok(svg) => println!("{}", svg),
        Err(err) => println!("{:#?}", err),
    }
}
