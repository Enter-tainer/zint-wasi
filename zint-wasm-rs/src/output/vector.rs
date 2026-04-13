use crate::Color;
use crate::{
    error::Error,
    options::rotation::Rotation,
    options::DisplayOptions,
    output::{PlotKind, PlotResult},
};
use serde::Serialize;
use std::{ffi::CStr, marker::PhantomData};
use zint_sys::*;

/// Generic vector output that can be used as a plot output when raw primitives
/// are necessary.
#[derive(Serialize)]
pub struct VectorPlot {
    pub width: f32,
    pub height: f32,

    pub geometry: VectorGeometry,

    #[cfg(feature = "display")]
    pub background: Color,
}

/// List of shapes the code is composed of.
#[derive(Serialize)]
pub struct VectorGeometry {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rectangles: Vec<Rect>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hexagons: Vec<Hexagon>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strings: Vec<Text>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub circles: Vec<Circle>,
}

impl<'a> PlotResult<'a> for VectorPlot {
    const KIND: PlotKind = PlotKind::Vector;

    fn from_symbol(
        symbol: &'a zint_sys::zint_symbol,
        options: &DisplayOptions,
    ) -> Result<Self, Error> {
        let vector_data = unsafe { symbol.vector.as_ref().ok_or(Error::MissingVectorData)? };

        Ok(Self {
            width: vector_data.width,
            height: vector_data.height,

            geometry: VectorGeometry {
                rectangles: LinkedListIter::new(vector_data.rectangles)
                    .map(|data| Rect::from_zint_data(data, options))
                    .collect(),
                hexagons: LinkedListIter::new(vector_data.hexagons)
                    .map(|data| Hexagon::from_zint_data(data, options))
                    .collect(),
                strings: LinkedListIter::new(vector_data.strings)
                    .map(|data| Text::from_zint_data(data, options))
                    .collect(),
                circles: LinkedListIter::new(vector_data.circles)
                    .map(|data| Circle::from_zint_data(data, options))
                    .collect(),
            },

            #[cfg(feature = "display")]
            background: options.background,
        })
    }
}

trait LinkedListNode {
    fn next_item(&self) -> *mut Self;
}
macro_rules! impl_next_ptr_field {
    ($($T:tt :: $next:ident),* $(,)?) => {
        $(
            impl LinkedListNode for $T {
                fn next_item(&self) -> *mut Self {
                    self.$next
                }
            }
        )*
    };
}
impl_next_ptr_field![
    zint_vector_rect::next,
    zint_vector_circle::next,
    zint_vector_hexagon::next,
    zint_vector_string::next,
];
struct LinkedListIter<'s, T: LinkedListNode> {
    head: *mut T,
    _phantom: PhantomData<&'s [T]>,
}
impl<'s, T: LinkedListNode> LinkedListIter<'s, T> {
    pub fn new(head: *mut T) -> Self {
        Self {
            head,
            _phantom: PhantomData,
        }
    }
}
impl<'s, T: LinkedListNode + 's> Iterator for LinkedListIter<'s, T> {
    type Item = &'s T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = unsafe { self.head.as_ref()? };
        self.head = current.next_item();
        Some(current)
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub width: f32,
    pub color: Color,
}

impl Rect {
    fn from_zint_data(value: &zint_vector_rect, options: &DisplayOptions) -> Self {
        Self {
            x: value.x,
            y: value.y,
            height: value.height,
            width: value.width,
            #[cfg(feature = "display")]
            color: to_color(value.colour, options.foreground),
            #[cfg(not(feature = "display"))]
            color: value.colour as i8,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub diameter: f32,
    pub width: f32,
    pub color: Color,
}

impl Circle {
    fn from_zint_data(value: &zint_vector_circle, options: &DisplayOptions) -> Self {
        Self {
            x: value.x,
            y: value.y,
            diameter: value.diameter,
            width: value.width,
            #[cfg(feature = "display")]
            color: to_color(value.colour, options.foreground),
            #[cfg(not(feature = "display"))]
            color: value.colour as i8,
        }
    }
}

/// Color is always foreground.
#[derive(Debug, Copy, Clone, Serialize)]
pub struct Hexagon {
    pub x: f32,
    pub y: f32,
    pub diameter: f32,
    pub rotation: Rotation,
    pub color: Color,
}

impl Hexagon {
    fn from_zint_data(value: &zint_vector_hexagon, options: &DisplayOptions) -> Self {
        Self {
            x: value.x,
            y: value.y,
            diameter: value.diameter,
            rotation: unsafe { std::mem::transmute::<i32, Rotation>(value.rotation) },
            #[cfg(feature = "display")]
            color: options.foreground,
            #[cfg(not(feature = "display"))]
            color: -1, // always foreground
        }
    }
}

/// Horizontal alignment of the text
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
#[repr(i32)]
pub enum HorizontalAlign {
    Center = 0,
    Left = 1,
    Right = 2,
}

/// Color is always foreground.
#[derive(Debug, Clone, Serialize)]
#[repr(C)]
pub struct Text {
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
    pub width: f32,
    pub length: i32,
    pub rotation: Rotation,
    pub horizontal_align: HorizontalAlign,
    pub text: String,
    pub color: Color,
}

impl Text {
    fn from_zint_data(value: &zint_vector_string, options: &DisplayOptions) -> Self {
        let text = unsafe {
            // TODO: In code, vector_add_string seems to always be called with utf-8 or symbol->text.
            CStr::from_ptr(value.text as *const i8)
                .to_str()
                .expect("not a utf-8 string")
        };

        Self {
            x: value.x,
            y: value.y,
            font_size: value.fsize,
            width: value.width,
            length: value.length,
            rotation: unsafe { std::mem::transmute::<i32, Rotation>(value.rotation) },
            horizontal_align: unsafe { std::mem::transmute::<i32, HorizontalAlign>(value.halign) },
            text: text.to_string(),
            #[cfg(feature = "display")]
            color: options.foreground,
            #[cfg(not(feature = "display"))]
            color: -1, // always foreground
        }
    }
}

/// Plotted vectors don't store actual color values, but instead an integer that
/// represents what the color should be in the target format.
///
/// This function remaps those to [`Color`]s.
#[cfg(feature = "display")]
fn to_color(mapping: i32, foreground: Color) -> Color {
    match mapping {
        -1 => foreground,
        1 => Color::new(0, 0xFF, 0xFF, 0xFF), // Cyan
        2 => Color::new(0, 0, 0xFF, 0xFF),    // Blue
        3 => Color::new(0xFF, 0, 0xFF, 0xFF), // Magenta
        4 => Color::new(0xFF, 0, 0, 0xFF),    // Red
        5 => Color::new(0xFF, 0xFF, 0, 0xFF), // Yellow
        6 => Color::new(0, 0xFF, 0, 0xFF),    // Green
        7 => Color::BLACK,                    // Black
        8 => Color::WHITE,                    // White
        _ => Color::BLACK,
    }
}
