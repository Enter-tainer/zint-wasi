use ciborium::Value;
use zint_rs::{
    options::DisplayOptions,
    segment::{Segment, ECI},
    GenericOptions, Symbology,
};

use crate::{error, output::ErrorDetail};

pub enum Format {
    Utf8,
    ECI,
}

/// Decoded barcode options from a CBOR map.
///
/// Parsed directly from [`ciborium::Value`] without an intermediate serde
/// struct, giving full control over field coercion and normalization.
pub struct DecodedOptions<'a> {
    pub generic: GenericOptions<'a>,
    pub display: DisplayOptions,
    pub format: Format,
}

/// Parse a CBOR byte slice into encoding and display options.
///
/// The CBOR input is a flat map with kebab-case keys from typst. Fields that
/// only affect rendering (colors, HRT styling) are silently skipped — they
/// are handled on the typst side.
pub fn decode_options(value: &ciborium::Value) -> Result<DecodedOptions<'_>, ErrorDetail> {
    let entries = value
        .as_map()
        .ok_or(error!("options must be a dictionary"))?;

    let mut symbology: Option<Symbology> = None;
    let mut generic = GenericOptions::default();
    let mut display = DisplayOptions::default();
    let mut format = Format::Utf8;

    for (key, val) in entries {
        let key_str = key.as_text().ok_or(error!("option key must be a string"))?;
        let normalized = kebab_to_snake(key_str);

        match normalized.as_str() {
            "symbology" => {
                symbology = Some(
                    val.deserialized()
                        .map_err(|e| error!("invalid symbology: {e}"))?,
                );
            }

            // --- GenericOptions fields ---
            "height" => {
                generic.height = as_f32(val, "height")?;
            }
            "whitespace_width" => {
                generic.whitespace_width = as_u32(val, "whitespace-width")?;
            }
            "whitespace_height" => {
                generic.whitespace_height = as_u32(val, "whitespace-height")?;
            }
            "border_width" => {
                generic.border_width = as_u32(val, "border-width")?;
            }
            "text_gap" => {
                generic.text_gap = as_f32(val, "text-gap")?;
            }
            "guard_descent" => {
                generic.guard_descent = as_f32(val, "guard-descent")?;
            }
            "output_options" => {
                generic.output_options = val
                    .deserialized()
                    .map_err(|e| error!("invalid `output-options`: {e}"))?;
            }
            "input_mode" => {
                generic.input_mode = Some(
                    val.deserialized()
                        .map_err(|e| error!("invalid `input-mode`: {e}"))?,
                );
            }
            "dot_size" => {
                generic.dot_size = Some(as_f32(val, "dot-size")?);
            }
            "primary" => {
                let s = val.as_text().ok_or(error!("`primary` must be a `str`"))?;
                let segment = Segment::new(s.as_bytes(), ECI::default());
                generic.primary_message = Some(segment);
            }
            "option_1" => {
                generic.option_1 = Some(as_i32(val, "option-1")?);
            }
            "option_2" => {
                generic.option_2 = Some(as_i32(val, "option-2")?);
            }
            "option_3" => {
                generic.option_3 = Some(as_i32(val, "option-3")?);
            }

            // --- DisplayOptions fields (layout-affecting only) ---
            "show_hrt" => {
                display.show_hrt = val.as_bool().ok_or(error!("`show-hrt` must be a `bool`"))?;
            }

            "format" => {
                format = match val.as_text() {
                    Some("utf8") => Format::Utf8,
                    Some("eci") => Format::ECI,
                    Some(other) => return Err(error!("unknown format: {other}")),
                    None => return Err(error!("`format` must be a string")),
                };
            }

            // Fields handled on the typst side (colors, scale, rotation, hrt styling, etc.)
            _ => {}
        }
    }

    let symbology = symbology.ok_or(error!("missing required field: `symbology`"))?;
    generic.symbology = symbology;

    Ok(DecodedOptions {
        generic,
        display,
        format,
    })
}

/// Normalize a CBOR field name for matching.
///
/// Typst sends kebab-case keys (`fg-color`, `output-options`).
fn kebab_to_snake(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
}

fn as_f32(val: &Value, field: &str) -> Result<f32, ErrorDetail> {
    match val {
        Value::Float(f) => Ok(*f as f32),
        Value::Integer(i) => {
            let n: i128 = (*i).into();
            let n = n as i64;
            Ok(n as f32)
        }
        _ => Err(error!("`{field}` must be an `int`")),
    }
}

fn as_u32(val: &Value, field: &str) -> Result<u32, ErrorDetail> {
    match val {
        Value::Integer(i) => {
            let n: i128 = (*i).into();
            let n = n as i64;
            Ok(n as u32)
        }
        Value::Float(f) => Ok(*f as u32),
        _ => Err(error!("`{field}` must be an `int`")),
    }
}

fn as_i32(val: &Value, field: &str) -> Result<i32, ErrorDetail> {
    match val {
        Value::Integer(i) => {
            let n: i128 = (*i).into();
            let n = n as i64;
            Ok(n as i32)
        }
        Value::Float(f) => Ok(*f as i32),
        _ => Err(error!("`{field}` must be an `int`")),
    }
}
