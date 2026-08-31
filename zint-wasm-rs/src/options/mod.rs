use serde::Deserialize;

use self::{
    color::Color, input_mode::InputMode, option3::Option3, output_options::OutputOptions,
    symbology::Symbology,
};

pub mod capability;
pub mod color;
pub mod input_mode;
pub mod option3;
pub mod output_options;
pub mod symbology;

/// The options a caller may set on a symbol.
///
/// Every field carries its own `default` rather than the container carrying one
/// for all of them, because `symbology` has to stay required: a document that
/// leaves it out has to hear about it instead of quietly being given a Code 128.
#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "typst", serde(rename_all = "kebab-case"))]
#[serde(deny_unknown_fields)]
pub struct Options {
    /// Barcode symbol to use
    pub symbology: Symbology,
    /// Barcode height in X-dimensions (ignored for fixed-width barcodes)
    #[serde(default)]
    pub height: Option<f32>,
    /// Scale factor when printing barcode, i.e. adjusts X-dimension. Default 1
    #[serde(default)]
    pub scale: Option<f32>,
    /// Width in X-dimensions of whitespace to left & right of barcode
    #[serde(default)]
    pub whitespace_width: Option<i32>,
    /// Height in X-dimensions of whitespace above & below the barcode
    #[serde(default)]
    pub whitespace_height: Option<i32>,
    /// Size of border in X-dimensions
    #[serde(default)]
    pub border_width: Option<i32>,
    /// Various output parameters (bind, box etc, see below)
    #[serde(default)]
    pub output_options: Option<OutputOptions>,
    /// foreground color
    #[serde(alias = "fg_colour", default)]
    #[cfg_attr(feature = "typst", serde(alias = "stroke", alias = "fg-colour"))]
    pub fg_color: Option<Color>,
    /// background color
    #[serde(alias = "bg_colour", default)]
    #[cfg_attr(feature = "typst", serde(alias = "fill", alias = "bg-colour"))]
    pub bg_color: Option<Color>,
    /// Primary message data (MaxiCode, Composite)
    #[serde(default)]
    pub primary: Option<String>,
    /// Symbol-specific options
    #[serde(default)]
    pub option_1: Option<i32>,
    /// Symbol-specific options
    #[serde(default)]
    pub option_2: Option<i32>,
    /// Symbol-specific options
    #[serde(default)]
    pub option_3: Option<Option3>,
    /// Show (1) or hide (0) Human Readable Text (HRT). Default 1
    #[serde(default)]
    pub show_hrt: Option<bool>,
    /// Encoding of input data
    #[serde(default)]
    pub input_mode: Option<InputMode>,
    /// Extended Channel Interpretation.
    #[serde(default)]
    pub eci: Option<i32>,
    /// Size of dots used in BARCODE_DOTTY_MODE.
    #[serde(default)]
    pub dot_size: Option<f32>,
    /// Gap between barcode and text (HRT) in X-dimensions.
    #[serde(default)]
    pub text_gap: Option<f32>,
    /// Height in X-dimensions that EAN/UPC guard bars descend.
    #[serde(default)]
    pub guard_descent: Option<f32>,
}

impl Options {
    pub fn with_symbology(symbology: Symbology) -> Self {
        Self {
            symbology,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        input_mode::InputMode, output_options::OutputOptions, symbology::Symbology, Options,
    };
    use crate::test_support::from_cbor;
    use ciborium::cbor;

    /// The Typst package renames the option keys to kebab-case; the library on
    /// its own keeps the Rust field names.
    ///
    /// Input:  `"show_hrt"`
    /// Output: `"show-hrt"` for the Typst plugin, `"show_hrt"` otherwise
    fn key(field: &str) -> String {
        if cfg!(feature = "typst") {
            field.replace('_', "-")
        } else {
            field.to_string()
        }
    }

    /// A barcode with everything turned on at once, so that a field that stops
    /// being read cannot hide behind the ones that still are.
    #[test]
    fn every_option_reaches_the_struct() {
        let options: Options = from_cbor(
            cbor!({
                "symbology" => "QRCode",
                key("height") => 30.0,
                key("scale") => 2.0,
                key("whitespace_width") => 4,
                key("whitespace_height") => 2,
                key("border_width") => 3,
                key("output_options") => {"barcode-box" => true},
                key("fg_color") => "#112233",
                key("bg_color") => [255, 255, 255, 0],
                key("primary") => "331234567890",
                key("option_1") => 2,
                key("option_2") => 5,
                key("option_3") => "full-multibyte",
                key("show_hrt") => false,
                key("input_mode") => {"format" => "gs1"},
                key("eci") => 26,
                key("dot_size") => 0.75,
                key("text_gap") => 1.5,
                key("guard_descent") => 4.0,
            })
            .unwrap(),
        )
        .expect("a document with every option set");

        assert_eq!(options.symbology as i32, Symbology::QRCode as i32);
        assert_eq!(options.height, Some(30.0));
        assert_eq!(options.scale, Some(2.0));
        assert_eq!(options.whitespace_width, Some(4));
        assert_eq!(options.whitespace_height, Some(2));
        assert_eq!(options.border_width, Some(3));
        assert_eq!(
            options.output_options.map(|it| it.bits()),
            Some(OutputOptions::BARCODE_BOX.bits())
        );
        assert_eq!(
            options.fg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("112233ff")
        );
        assert_eq!(
            options.bg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("ffffff00")
        );
        assert_eq!(options.primary.as_deref(), Some("331234567890"));
        assert_eq!(options.option_1, Some(2));
        assert_eq!(options.option_2, Some(5));
        assert_eq!(options.option_3.map(|it| it.as_i32()), Some(200));
        assert_eq!(options.show_hrt, Some(false));
        assert_eq!(
            options.input_mode.map(|it| it.bits()),
            Some(InputMode::GS1.bits())
        );
        assert_eq!(options.eci, Some(26));
        assert_eq!(options.dot_size, Some(0.75));
        assert_eq!(options.text_gap, Some(1.5));
        assert_eq!(options.guard_descent, Some(4.0));
    }

    /// Anything left out has to stay unset, because zint's own defaults are
    /// what fills in for it.
    #[test]
    fn options_that_are_left_out_stay_unset() {
        let options: Options =
            from_cbor(cbor!({"symbology" => "Code128"}).unwrap()).expect("only a symbology");

        assert_eq!(options.symbology as i32, Symbology::Code128 as i32);
        assert!(options.height.is_none());
        assert!(options.scale.is_none());
        assert!(options.whitespace_width.is_none());
        assert!(options.whitespace_height.is_none());
        assert!(options.border_width.is_none());
        assert!(options.output_options.is_none());
        assert!(options.fg_color.is_none());
        assert!(options.bg_color.is_none());
        assert!(options.primary.is_none());
        assert!(options.option_1.is_none());
        assert!(options.option_2.is_none());
        assert!(options.option_3.is_none());
        assert!(options.show_hrt.is_none());
        assert!(options.input_mode.is_none());
        assert!(options.eci.is_none());
        assert!(options.dot_size.is_none());
        assert!(options.text_gap.is_none());
        assert!(options.guard_descent.is_none());
    }

    /// Zint spells colour the British way, so documents written against its
    /// documentation keep working.
    #[test]
    fn the_british_spelling_of_colour_is_accepted() {
        let options: Options = from_cbor(
            cbor!({
                "symbology" => "Code128",
                "fg_colour" => "#112233",
                "bg_colour" => "#445566",
            })
            .unwrap(),
        )
        .expect("colour spelled the zint way");

        assert_eq!(
            options.fg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("112233ff")
        );
        assert_eq!(
            options.bg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("445566ff")
        );
    }

    /// Zint's own documentation spells the colours the British way, and inside
    /// Typst every key is spelled with dashes, so that combination has to be
    /// understood as well.
    #[cfg(feature = "typst")]
    #[test]
    fn the_british_spelling_is_accepted_with_dashes_too() {
        let options: Options = from_cbor(
            cbor!({
                "symbology" => "Code128",
                "fg-colour" => "#112233",
                "bg-colour" => "#445566",
            })
            .unwrap(),
        )
        .expect("colour spelled the zint way, keyed the Typst way");

        assert_eq!(
            options.fg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("112233ff")
        );
        assert_eq!(
            options.bg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("445566ff")
        );
    }

    /// Inside Typst a barcode is drawn like any other shape, so it takes the
    /// `stroke` and `fill` names the drawing functions use.
    #[cfg(feature = "typst")]
    #[test]
    fn the_typst_drawing_names_are_accepted_for_the_colors() {
        let options: Options = from_cbor(
            cbor!({
                "symbology" => "Code128",
                "stroke" => "#112233",
                "fill" => "#445566",
            })
            .unwrap(),
        )
        .expect("stroke and fill name the colors");

        assert_eq!(
            options.fg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("112233ff")
        );
        assert_eq!(
            options.bg_color.map(|it| it.to_hex_string()).as_deref(),
            Some("445566ff")
        );
    }

    #[test]
    fn with_symbology_leaves_every_other_option_unset() {
        let options = Options::with_symbology(Symbology::DataMatrix);

        assert_eq!(options.symbology as i32, Symbology::DataMatrix as i32);
        assert!(options.scale.is_none());
        assert!(options.fg_color.is_none());
        assert!(options.primary.is_none());
    }

    /// A key the library does not know is a typo, and a typo that is dropped
    /// leaves a barcode that looks right and is not the one that was asked for.
    ///
    /// Input:  `{"symbology": "Code128", "show_hrt": false}` for the Typst
    ///         plugin, which spells its keys with dashes
    /// Output: an error naming `show_hrt`, rather than a barcode with its text
    #[test]
    fn an_unknown_option_is_rejected() {
        // The separator the other build uses, which is the likeliest typo.
        let misspelled = if cfg!(feature = "typst") {
            "show_hrt"
        } else {
            "show-hrt"
        };

        let error =
            from_cbor::<Options>(cbor!({"symbology" => "Code128", misspelled => false}).unwrap())
                .expect_err("the key is a misspelling of one this library knows");

        assert!(
            error.contains(misspelled),
            "the error should name the key that was not understood: {error}"
        );
    }

    /// Every other option falls back to what zint chose, but there is no
    /// sensible barcode to fall back to, so leaving the symbology out is an
    /// error rather than a silent Code 128.
    #[test]
    fn a_document_without_a_symbology_is_rejected() {
        let error = from_cbor::<Options>(cbor!({key("scale") => 2.0}).unwrap())
            .expect_err("a barcode has to say which symbology it is");

        assert!(
            error.contains("symbology"),
            "the error should name the missing field: {error}"
        );
    }

    #[test]
    fn an_option_of_the_wrong_type_is_rejected() {
        let error = from_cbor::<Options>(
            cbor!({"symbology" => "Code128", key("scale") => "large"}).unwrap(),
        )
        .expect_err("a scale is a number");
        assert!(!error.is_empty(), "the error should name the problem");
    }
}
