use zint_wasm_rs::{
    options::{symbology::Symbology, DisplayOptions, EncodingOptions},
    output::{EmfPlot, PsPlot, SvgPlot},
    symbol::Symbol,
};

pub fn main() {
    let encoded_text = "A12345B";
    let symbol = Symbol::encode_ascii(
        Symbology::Code128,
        encoded_text,
        &EncodingOptions::default(),
    );
    let mut symbol = match symbol {
        Ok(it) => it,
        Err(err) => {
            eprintln!("unable to encode code 128 symbol: {err}");
            std::process::exit(1);
        },
    };
    let plot: SvgPlot = symbol
        .plot(&DisplayOptions::default())
        .expect("unable to plot symbol");
    let plot: String = plot.into();
    println!("{plot}");
}
