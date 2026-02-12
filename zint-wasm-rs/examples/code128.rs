use zint_rs::{
    Symbol,
    options::{DisplayOptions, GenericOptions, symbology::Symbology},
    output::SvgPlot,
};

pub fn main() {
    let encoded_text = "A12345B";
    let mut symbol = Symbol::encode_ascii(
        GenericOptions::from_symbology(Symbology::Code128),
        encoded_text,
    )
    .unwrap_or_else(|err| {
        eprintln!("unable to encode code 128 symbol: {err}");
        std::process::exit(1);
    });
    let plot: SvgPlot = symbol
        .plot(&DisplayOptions::default())
        .expect("unable to plot symbol");
    let plot: String = plot.into();
    println!("{plot}");
}
