//! The shapes zint lays a symbol out as, before any output format is involved.
//!
//! Zint's SVG, EPS and EMF writers all draw from the same `zint_vector`: a
//! list each of rectangles, hexagons, circles and strings, in a coordinate
//! system where one X-dimension is two units at scale 1. This module copies
//! that list out of zint's linked lists into plain values that can be handed
//! on, so that a caller can draw the symbol with its own primitives and its
//! own colours instead of parsing an SVG.
//!
//! The values say *what* to draw, not in which colour: bars, dots and text are
//! foreground, and only Ultracode names colours of its own. The caller decides
//! what foreground means.

use serde::Serialize;
use zint_wasm_sys::{
    zint_vector, zint_vector_circle, zint_vector_hexagon, zint_vector_rect, zint_vector_string,
};

use crate::error::Error;

/// Everything zint drew for one symbol.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Vector {
    /// Width of the whole image, quiet zones and text included.
    pub width: f32,
    /// Height of the whole image, quiet zones and text included.
    pub height: f32,
    pub rectangles: Vec<Rectangle>,
    pub hexagons: Vec<Hexagon>,
    pub circles: Vec<Circle>,
    pub texts: Vec<Text>,
}

/// A bar, a module, or a run of modules.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Rectangle {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub ink: Ink,
}

/// What a rectangle is filled with.
///
/// Every symbology but Ultracode draws in the foreground alone. Ultracode
/// encodes data in colour, so its modules name the colour the standard gives
/// them, and a caller has to draw them in that colour for the symbol to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ink {
    Foreground,
    Cyan,
    Blue,
    Magenta,
    Red,
    Yellow,
    Green,
    Black,
    White,
}

impl Ink {
    /// Reads zint's `colour` field.
    ///
    /// Input:  `-1`, or `3`
    /// Output: `Ink::Foreground`, or `Ink::Magenta`
    fn from_zint(colour: i32) -> Result<Self, Error> {
        Ok(match colour {
            -1 => Self::Foreground,
            1 => Self::Cyan,
            2 => Self::Blue,
            3 => Self::Magenta,
            4 => Self::Red,
            5 => Self::Yellow,
            6 => Self::Green,
            7 => Self::Black,
            8 => Self::White,
            // A colour zint added after this was written would otherwise be
            // drawn as something it is not, and the symbol would not scan.
            _ => return Err(Error::UnknownVectorColour(colour)),
        })
    }
}

/// A MaxiCode module, which is a hexagon with flat sides left and right at a
/// rotation of 0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Hexagon {
    /// Centre.
    pub x: f32,
    /// Centre.
    pub y: f32,
    /// The short diameter, between two opposite sides.
    pub diameter: f32,
    /// 0, 90, 180 or 270 degrees. At 0 and 180 an apex points up; at 90 and
    /// 270 an apex points left.
    pub rotation: i32,
}

/// A dot in dotty mode, or a ring of the MaxiCode bullseye.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Circle {
    /// Centre.
    pub x: f32,
    /// Centre.
    pub y: f32,
    /// Diameter of the circle, not counting the width of its outline.
    pub diameter: f32,
    /// Width of the outline for a ring, and 0 for a filled disc.
    pub stroke: f32,
    /// Whether the circle is drawn in the background colour rather than the
    /// foreground. Zint no longer does this, but its format still allows it.
    pub background: bool,
}

/// A run of human readable text.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Text {
    /// Horizontal anchor, which `align` says how to read.
    pub x: f32,
    /// The baseline.
    pub y: f32,
    /// Font size, in the same units as everything else.
    pub size: f32,
    /// Zint's estimate of the rendered width, which it used for the layout.
    pub width: f32,
    /// 0, 90, 180 or 270 degrees, about the anchor.
    pub rotation: i32,
    pub align: Align,
    pub text: String,
    /// Whether zint's own writers set this text in bold, which they do when
    /// `BOLD_TEXT` is asked for, except on EAN and UPC symbols.
    pub bold: bool,
}

/// How a text's `x` relates to the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Align {
    /// `x` is the middle of the text.
    Center,
    /// `x` is where the text starts.
    Left,
    /// `x` is where the text ends.
    Right,
}

impl Align {
    /// Reads zint's `halign` field.
    ///
    /// Input:  `0`, or `2`
    /// Output: `Align::Center`, or `Align::Right`
    fn from_zint(halign: i32) -> Result<Self, Error> {
        match halign {
            0 => Ok(Self::Center),
            1 => Ok(Self::Left),
            2 => Ok(Self::Right),
            _ => Err(Error::UnknownVectorAlignment(halign)),
        }
    }
}

impl Vector {
    /// Copies what `vector` points at. `bold` is whether its texts are bold,
    /// which the vector does not record.
    ///
    /// # Safety
    ///
    /// `vector` must be the vector zint built for a symbol that is still alive,
    /// so that every list it heads is intact.
    pub(crate) unsafe fn from_zint(vector: &zint_vector, bold: bool) -> Result<Self, Error> {
        Ok(Self {
            width: vector.width,
            height: vector.height,
            rectangles: walk(vector.rectangles, |it: &zint_vector_rect| it.next)
                .map(|it| {
                    Ok(Rectangle {
                        x: it.x,
                        y: it.y,
                        width: it.width,
                        height: it.height,
                        ink: Ink::from_zint(it.colour)?,
                    })
                })
                .collect::<Result<_, Error>>()?,
            hexagons: walk(vector.hexagons, |it: &zint_vector_hexagon| it.next)
                .map(|it| Hexagon {
                    x: it.x,
                    y: it.y,
                    diameter: it.diameter,
                    rotation: it.rotation,
                })
                .collect(),
            circles: walk(vector.circles, |it: &zint_vector_circle| it.next)
                .map(|it| Circle {
                    x: it.x,
                    y: it.y,
                    diameter: it.diameter,
                    stroke: it.width,
                    background: it.colour != 0,
                })
                .collect(),
            texts: walk(vector.strings, |it: &zint_vector_string| it.next)
                .map(|it| {
                    Ok(Text {
                        x: it.x,
                        y: it.y,
                        size: it.fsize,
                        width: it.width,
                        rotation: it.rotation,
                        align: Align::from_zint(it.halign)?,
                        text: text_of(it),
                        bold,
                    })
                })
                .collect::<Result<_, Error>>()?,
        })
    }
}

/// The text of a string, read by the length zint recorded for it rather than
/// up to a terminator, so that a missing one cannot run the read past it.
fn text_of(string: &zint_vector_string) -> String {
    if string.text.is_null() || string.length <= 0 {
        return String::new();
    }
    let bytes = unsafe {
        // Safety: zint allocates `length` bytes of text, plus a terminator,
        // for every string it adds to the vector.
        std::slice::from_raw_parts(string.text, string.length as usize)
    };
    // Zint writes UTF-8; a mangled character still beats losing the text.
    String::from_utf8_lossy(bytes).into_owned()
}

/// Iterates one of zint's singly linked lists, starting at `head` and
/// following `next` until it reaches null.
///
/// # Safety
///
/// Every node reachable from `head` must be valid for as long as the iterator
/// and the references it yields are used.
unsafe fn walk<'a, T: 'a>(
    head: *mut T,
    next: impl Fn(&T) -> *mut T + 'a,
) -> impl Iterator<Item = &'a T> + 'a {
    let mut current = head;
    std::iter::from_fn(move || {
        let node = unsafe {
            // Safety: guaranteed by the caller of `walk`.
            current.as_ref()?
        };
        current = next(node);
        Some(node)
    })
}

#[cfg(test)]
mod tests {
    use super::{Align, Ink};
    use crate::error::Error;

    /// Ultracode reads by colour, so each number zint uses has to land on the
    /// colour its header assigns: "1-8 for Cyan, Blue, Magenta, Red, Yellow,
    /// Green, Black, White".
    #[test]
    fn every_colour_zint_names_is_the_ink_it_means() {
        let inks: Vec<Ink> = [-1, 1, 2, 3, 4, 5, 6, 7, 8]
            .into_iter()
            .map(|colour| Ink::from_zint(colour).expect("zint defines it"))
            .collect();

        assert_eq!(
            inks,
            [
                Ink::Foreground,
                Ink::Cyan,
                Ink::Blue,
                Ink::Magenta,
                Ink::Red,
                Ink::Yellow,
                Ink::Green,
                Ink::Black,
                Ink::White,
            ]
        );
    }

    /// 0 is the gap between -1 and the first colour, and 9 is one past the
    /// last, so between them they cover a colour zint might add at either end.
    #[test]
    fn a_colour_zint_does_not_define_is_refused() {
        for colour in [i32::MIN, -2, 0, 9, i32::MAX] {
            let error = Ink::from_zint(colour).expect_err("not a colour zint defines");
            assert!(
                matches!(error, Error::UnknownVectorColour(it) if it == colour),
                "unexpected error: {error:?}"
            );
        }
    }

    #[test]
    fn every_alignment_zint_names_is_read() {
        assert_eq!(Align::from_zint(0).ok(), Some(Align::Center));
        assert_eq!(Align::from_zint(1).ok(), Some(Align::Left));
        assert_eq!(Align::from_zint(2).ok(), Some(Align::Right));
    }

    #[test]
    fn an_alignment_zint_does_not_define_is_refused() {
        for halign in [i32::MIN, -1, 3, i32::MAX] {
            let error = Align::from_zint(halign).expect_err("not an alignment zint defines");
            assert!(
                matches!(error, Error::UnknownVectorAlignment(it) if it == halign),
                "unexpected error: {error:?}"
            );
        }
    }
}
