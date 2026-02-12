use serde::Deserialize;
use zint_rs::{
    Color,
    options::{
        DisplayOptions, GenericOptions, input_mode::InputMode, option3::Option3,
        output_options::OutputOptions, rotation::Rotation, symbology::Symbology,
    },
};

/// CBOR-deserializable options received from the Typst plugin.
///
/// This struct maps the CBOR dictionary sent by `lib.typ` into
/// the new split `GenericOptions` + `DisplayOptions` API.
#[derive(Debug, Deserialize)]
pub struct PluginOptions {
    pub symbology: Symbology,

    // Display options
    #[serde(rename = "fg-color")]
    pub fg_color: Option<Color>,
    #[serde(rename = "bg-color")]
    pub bg_color: Option<Color>,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default)]
    pub rotation: Option<u16>,

    // Encoding options
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(rename = "output-options")]
    pub output_options: Option<OutputOptions>,
    #[serde(rename = "dot-size")]
    pub dot_size: Option<f32>,
    #[serde(default)]
    pub option_1: Option<i32>,
    #[serde(default)]
    pub option_2: Option<i32>,
    #[serde(default)]
    pub option_3: Option<Option3>,
    #[serde(default)]
    pub primary: Option<String>,

    // Additional encoding options
    #[serde(default)]
    pub eci: Option<i32>,
    #[serde(default, rename = "input-mode")]
    pub input_mode: Option<InputMode>,
    #[serde(default, rename = "whitespace-width")]
    pub whitespace_width: Option<u32>,
    #[serde(default, rename = "whitespace-height")]
    pub whitespace_height: Option<u32>,
    #[serde(default, rename = "border-width")]
    pub border_width: Option<u32>,
    #[serde(default, rename = "text-gap")]
    pub text_gap: Option<f32>,
    #[serde(default, rename = "guard-descent")]
    pub guard_descent: Option<f32>,

    // Display options
    #[serde(default, rename = "show-hrt")]
    pub show_hrt: Option<bool>,

    // Legacy HRT option (accepted to avoid parse errors for old configs)
    #[serde(default, rename = "hrt")]
    pub _hrt: Option<serde::de::IgnoredAny>,
}

impl PluginOptions {
    pub fn into_options(self) -> (GenericOptions<'static>, DisplayOptions, Option<String>) {
        let mut generic = GenericOptions::from_symbology(self.symbology);

        if let Some(height) = self.height {
            generic.height = height;
        }
        if let Some(output_options) = self.output_options {
            generic.output_options = output_options;
        }
        if let Some(dot_size) = self.dot_size {
            generic.dot_size = Some(dot_size);
        }
        if let Some(option_1) = self.option_1 {
            generic.option_1 = Some(option_1);
        }
        if let Some(option_2) = self.option_2 {
            generic.option_2 = Some(option_2);
        }
        if let Some(option_3) = self.option_3 {
            generic.option_3 = Some(option_3);
        }
        if let Some(input_mode) = self.input_mode {
            generic.input_mode = Some(input_mode);
        }
        if let Some(whitespace_width) = self.whitespace_width {
            generic.whitespace_width = whitespace_width;
        }
        if let Some(whitespace_height) = self.whitespace_height {
            generic.whitespace_height = whitespace_height;
        }
        if let Some(border_width) = self.border_width {
            generic.border_width = border_width;
        }
        if let Some(text_gap) = self.text_gap {
            generic.text_gap = text_gap;
        }
        if let Some(guard_descent) = self.guard_descent {
            generic.guard_descent = guard_descent;
        }

        let mut display = DisplayOptions::default();
        if let Some(fg_color) = self.fg_color {
            display.foreground = fg_color;
        }
        if let Some(bg_color) = self.bg_color {
            display.background = bg_color;
        }
        if let Some(scale) = self.scale {
            display.scale = scale;
        }
        if let Some(rotation) = self.rotation {
            display.rotation = match rotation {
                90 => Rotation::Deg90,
                180 => Rotation::Deg180,
                270 => Rotation::Deg270,
                _ => Rotation::Deg0,
            };
        }
        if let Some(show_hrt) = self.show_hrt {
            display.show_hrt = show_hrt;
        }

        (generic, display, self.primary)
    }
}
