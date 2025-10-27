use zint_rs::{
    options::{symbology::Symbology, DisplayOptions},
    output::SvgPlot,
    symbol::Symbol, GenericOptions,
};

pub fn main() {
    let encoded_text = "A12345B";
    let symbol = Symbol::encode_ascii(
        GenericOptions::from_symbology(Symbology::Code128),
        encoded_text,
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
