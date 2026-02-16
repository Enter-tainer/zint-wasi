use crate::{
    error::Error,
    options::{rotation::Rotation, DisplayOptions},
    output::{PlotKind, PlotResult},
};
use std::{ffi::CStr, marker::PhantomData};
use zint_sys::*;

/// Semantic color mapping for vector output elements.
///
/// Instead of embedding resolved RGBA values, each element carries a semantic
/// tag so the rendering side (e.g. Typst) can fill in the actual colors at
/// render time, enabling support for rich color spaces (CMYK, Oklab, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorMapping {
    Foreground,
    Background,
    Palette(i32),
}

/// Generic vector output that can be used as a plot output when raw primitives
/// are necessary.
#[derive(serde::Serialize)]
pub struct VectorPlot {
    pub width: f32,
    pub height: f32,

    pub geometry: VectorGeometry,
}

/// List of shapes the code is composed of.
#[derive(serde::Serialize)]
pub struct VectorGeometry {
    pub rectangles: Vec<Rect>,
    pub hexagons: Vec<Hexagon>,
    pub strings: Vec<Text>,
    pub circles: Vec<Circle>,
}

impl<'a> PlotResult<'a> for VectorPlot {
    const KIND: PlotKind = PlotKind::Vector;

    fn from_symbol(
        symbol: &'a mut zint_sys::zint_symbol,
        options: &crate::options::DisplayOptions,
    ) -> Result<Self, crate::error::Error> {
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

#[derive(Debug, Copy, Clone, serde::Serialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub width: f32,
    pub color: ColorMapping,
}

impl Rect {
    fn from_zint_data(value: &zint_vector_rect, _options: &DisplayOptions) -> Self {
        Self {
            x: value.x,
            y: value.y,
            height: value.height,
            width: value.width,
            color: to_color_mapping(value.colour),
        }
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub diameter: f32,
    pub width: f32,
    pub color: ColorMapping,
}

impl Circle {
    fn from_zint_data(value: &zint_vector_circle, _options: &DisplayOptions) -> Self {
        Self {
            x: value.x,
            y: value.y,
            diameter: value.diameter,
            width: value.width,
            color: to_color_mapping(value.colour),
        }
    }
}

/// Color is always foreground.
#[derive(Debug, Copy, Clone, serde::Serialize)]
pub struct Hexagon {
    pub x: f32,
    pub y: f32,
    pub diameter: f32,
    pub rotation: Rotation,
    pub color: ColorMapping,
}

impl Hexagon {
    fn from_zint_data(value: &zint_vector_hexagon, _options: &DisplayOptions) -> Self {
        Self {
            x: value.x,
            y: value.y,
            diameter: value.diameter,
            rotation: match value.rotation {
                0 => Rotation::Deg0,
                90 => Rotation::Deg90,
                180 => Rotation::Deg180,
                270 => Rotation::Deg270,
                other => panic!("unexpected rotation value: {other}"),
            },
            color: ColorMapping::Foreground,
        }
    }
}

/// Horizontal alignment of the text
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(into = "i32")]
#[repr(i32)]
pub enum HorizontalAlign {
    Center = 0,
    Left = 1,
    Right = 2,
}

impl From<HorizontalAlign> for i32 {
    fn from(value: HorizontalAlign) -> Self {
        value as i32
    }
}

/// Color is always foreground.
#[derive(Debug, Clone, serde::Serialize)]
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
    pub color: ColorMapping,
}

impl Text {
    fn from_zint_data(value: &zint_vector_string, _options: &DisplayOptions) -> Self {
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
            rotation: match value.rotation {
                0 => Rotation::Deg0,
                90 => Rotation::Deg90,
                180 => Rotation::Deg180,
                270 => Rotation::Deg270,
                other => panic!("unexpected rotation value: {other}"),
            },
            horizontal_align: match value.halign {
                0 => HorizontalAlign::Center,
                1 => HorizontalAlign::Left,
                2 => HorizontalAlign::Right,
                other => panic!("unexpected halign value: {other}"),
            },
            text: text.to_string(),
            color: ColorMapping::Foreground,
        }
    }
}

/// Converts zint's internal color integer mapping to a [`ColorMapping`].
///
/// Mapping -1 means foreground; positive integers are palette indices
/// (1=cyan, 2=blue, 3=magenta, 4=red, 5=yellow, 6=green, 7=black, 8=white).
fn to_color_mapping(mapping: i32) -> ColorMapping {
    match mapping {
        -1 => ColorMapping::Foreground,
        idx => ColorMapping::Palette(idx),
    }
}
