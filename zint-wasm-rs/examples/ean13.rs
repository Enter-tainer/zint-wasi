use zint_rs::{
    options::{
        rotation::Rotation, symbology::EAN13Options, DisplayOptions,
        WarningHandling,
    }, output::SvgPlot, segment::{Segment, ECI}, symbol::Symbol, Color, SymbolOptions
};

pub fn main() {
    let mut symbol = Symbol::encode_segments(
        SymbolOptions::EAN13(EAN13Options::default()),
        &[Segment::new(b"6975004310001", ECI::NONE)],
    )
    .expect("unable to encode EAN-13 symbol");
    let plot: SvgPlot = symbol
        .plot(&DisplayOptions {
            scale: 2.0,
            foreground: Color::new(0x12, 0x34, 0x56, 0xFF),
            background: Color::TRANSPARENT,
            rotation: Rotation::Deg270,
            show_hrt: true,
            warnings: WarningHandling::LogWarnings(log::Level::Info),
        })
        .expect("unable to plot symbol");
    println!("{plot}");
}
