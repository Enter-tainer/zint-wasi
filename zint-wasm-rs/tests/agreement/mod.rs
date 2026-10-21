//! Checks the shapes `Symbol::encode_vector` returns against the SVG zint
//! writes from the same shapes.
//!
//! Zint's SVG writer walks the very lists `encode_vector` copies out, and the
//! SVGs it produces are already pinned by the golden files. So rather than pin
//! the vectors a second time, each one is compared with the SVG drawn for the
//! same case: every rectangle, hexagon, circle and text has to be there, in
//! order, where the SVG puts it. A field read from the wrong place, a list cut
//! short or a colour mapped to the wrong ink shows up as a disagreement.
//!
//! The SVG rounds coordinates to two decimals, the radius and outline of a
//! circle to three and font sizes to one, so values are compared to within
//! that rounding.

use zint_wasm_rs::vector::{Align, Circle, Hexagon, Ink, Rectangle, Text, Vector};

/// Fails with the first shape the two disagree on.
pub fn vector_draws_what_the_svg_draws(vector: &Vector, svg: &str) -> Result<(), String> {
    let drawn = parse(svg)?;

    let (width, height) = drawn.size;
    if width != vector.width.ceil() || height != vector.height.ceil() {
        return Err(format!(
            "the SVG is {width}x{height}, the vector {}x{}",
            vector.width, vector.height
        ));
    }

    compare(
        "rectangle",
        &vector.rectangles,
        &drawn.rectangles,
        rectangle,
    )?;
    compare("hexagon", &vector.hexagons, &drawn.hexagons, hexagon)?;
    compare("circle", &vector.circles, &drawn.circles, circle)?;
    compare("text", &vector.texts, &drawn.texts, text)?;
    Ok(())
}

/// Compares two lists shape by shape, then their lengths, so that a missing
/// shape is reported as the first one that differs where possible.
fn compare<V, S>(
    kind: &str,
    vector: &[V],
    svg: &[S],
    same: impl Fn(&V, &S) -> Result<(), String>,
) -> Result<(), String> {
    for (index, (expected, drawn)) in vector.iter().zip(svg).enumerate() {
        same(expected, drawn).map_err(|error| format!("{kind} {index}: {error}"))?;
    }
    if vector.len() != svg.len() {
        return Err(format!(
            "the vector has {} of type {kind}, the SVG {}",
            vector.len(),
            svg.len()
        ));
    }
    Ok(())
}

/// Whether `value` prints as `printed` when rounded to `decimals` places.
fn close(value: f32, printed: f32, decimals: i32) -> bool {
    (value - printed).abs() <= 0.5 * 10f32.powi(-decimals) + 1e-3
}

fn expect_close(name: &str, value: f32, printed: f32, decimals: i32) -> Result<(), String> {
    if close(value, printed, decimals) {
        Ok(())
    } else {
        Err(format!("{name} is {value}, the SVG says {printed}"))
    }
}

/// The fill zint's SVG writer gives a rectangle of each ink. The foreground is
/// inherited from the enclosing group, so it carries none of its own.
fn fill_of(ink: Ink) -> Option<&'static str> {
    match ink {
        Ink::Foreground => None,
        Ink::Cyan => Some("00ffff"),
        Ink::Blue => Some("0000ff"),
        Ink::Magenta => Some("ff00ff"),
        Ink::Red => Some("ff0000"),
        Ink::Yellow => Some("ffff00"),
        Ink::Green => Some("00ff00"),
        Ink::Black => Some("000000"),
        Ink::White => Some("ffffff"),
    }
}

fn rectangle(vector: &Rectangle, drawn: &DrawnRectangle) -> Result<(), String> {
    expect_close("x", vector.x, drawn.x, 2)?;
    expect_close("y", vector.y, drawn.y, 2)?;
    expect_close("width", vector.width, drawn.width, 2)?;
    expect_close("height", vector.height, drawn.height, 2)?;
    let expected = fill_of(vector.ink);
    if expected != drawn.fill.as_deref() {
        return Err(format!(
            "{:?} would be filled with {expected:?}, the SVG uses {:?}",
            vector.ink, drawn.fill
        ));
    }
    Ok(())
}

/// The six corners zint's SVG writer draws for a hexagon, in its order.
fn corners(hexagon: &Hexagon) -> [(f32, f32); 6] {
    let (x, y) = (hexagon.x, hexagon.y);
    let radius = 0.5 * hexagon.diameter;
    let half = 0.25 * hexagon.diameter;
    let half_sqrt3 = 0.433_012_7 * hexagon.diameter;
    if hexagon.rotation == 0 || hexagon.rotation == 180 {
        [
            (x, y + radius),
            (x + half_sqrt3, y + half),
            (x + half_sqrt3, y - half),
            (x, y - radius),
            (x - half_sqrt3, y - half),
            (x - half_sqrt3, y + half),
        ]
    } else {
        [
            (x - radius, y),
            (x - half, y + half_sqrt3),
            (x + half, y + half_sqrt3),
            (x + radius, y),
            (x + half, y - half_sqrt3),
            (x - half, y - half_sqrt3),
        ]
    }
}

fn hexagon(vector: &Hexagon, drawn: &DrawnHexagon) -> Result<(), String> {
    for (corner, ((x, y), (drawn_x, drawn_y))) in corners(vector).iter().zip(drawn).enumerate() {
        expect_close(&format!("corner {corner} x"), *x, *drawn_x, 2)?;
        expect_close(&format!("corner {corner} y"), *y, *drawn_y, 2)?;
    }
    Ok(())
}

fn circle(vector: &Circle, drawn: &DrawnCircle) -> Result<(), String> {
    expect_close("x", vector.x, drawn.x, 2)?;
    expect_close("y", vector.y, drawn.y, 2)?;
    let ring = vector.stroke != 0.0;
    expect_close(
        "radius",
        vector.diameter / 2.0,
        drawn.radius,
        if ring { 3 } else { 2 },
    )?;
    match drawn.stroke {
        Some(stroke) => expect_close("stroke", vector.stroke, stroke, 3),
        None if !ring => Ok(()),
        None => Err(format!(
            "a ring of width {} is drawn as a disc",
            vector.stroke
        )),
    }
}

fn text(vector: &Text, drawn: &DrawnText) -> Result<(), String> {
    expect_close("x", vector.x, drawn.x, 2)?;
    expect_close("y", vector.y, drawn.y, 2)?;
    expect_close("size", vector.size, drawn.size, 1)?;
    let anchor = match vector.align {
        Align::Center => "middle",
        Align::Left => "start",
        Align::Right => "end",
    };
    if anchor != drawn.anchor {
        return Err(format!(
            "{:?} is anchored {}, the SVG anchors it {}",
            vector.align, anchor, drawn.anchor
        ));
    }
    if vector.rotation != drawn.rotation {
        return Err(format!(
            "rotated {}, the SVG rotates it {}",
            vector.rotation, drawn.rotation
        ));
    }
    if vector.bold != drawn.bold {
        return Err(format!(
            "bold is {}, the SVG says {}",
            vector.bold, drawn.bold
        ));
    }
    if vector.text != drawn.text {
        return Err(format!("{:?}, the SVG says {:?}", vector.text, drawn.text));
    }
    Ok(())
}

/// What one of zint's SVGs draws, read back into numbers.
#[derive(Debug, Default)]
struct Drawn {
    size: (f32, f32),
    rectangles: Vec<DrawnRectangle>,
    hexagons: Vec<DrawnHexagon>,
    circles: Vec<DrawnCircle>,
    texts: Vec<DrawnText>,
}

#[derive(Debug)]
struct DrawnRectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    /// The path's own fill, lower case and without the `#`.
    fill: Option<String>,
}

type DrawnHexagon = Vec<(f32, f32)>;

#[derive(Debug)]
struct DrawnCircle {
    x: f32,
    y: f32,
    radius: f32,
    stroke: Option<f32>,
}

#[derive(Debug)]
struct DrawnText {
    x: f32,
    y: f32,
    size: f32,
    anchor: String,
    rotation: i32,
    bold: bool,
    text: String,
}

/// Reads an SVG the way zint writes it: one element to a line, a text's
/// content on the line after its start tag.
fn parse(svg: &str) -> Result<Drawn, String> {
    let mut drawn = Drawn::default();
    let mut lines = svg.lines();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.starts_with("<svg ") {
            drawn.size = (number(line, "width")?, number(line, "height")?);
        } else if line.starts_with("<path ") {
            let path = attribute(line, "d")?;
            let fill = attribute(line, "fill")
                .ok()
                .map(|it| it.trim_start_matches('#').to_ascii_lowercase());
            // Hexagons are drawn with lines between their corners, and
            // rectangles with horizontal and vertical steps only.
            if path.contains('L') {
                for shape in shapes(path) {
                    drawn.hexagons.push(parse_hexagon(shape)?);
                }
            } else {
                for shape in shapes(path) {
                    drawn.rectangles.push(parse_rectangle(shape, fill.clone())?);
                }
            }
        } else if line.starts_with("<circle ") {
            drawn.circles.push(DrawnCircle {
                x: number(line, "cx")?,
                y: number(line, "cy")?,
                radius: number(line, "r")?,
                stroke: number(line, "stroke-width").ok(),
            });
        } else if line.starts_with("<text ") {
            // Zint indents the content by three spaces and adds nothing else,
            // so whatever follows them is the text, spaces included.
            let content = lines
                .next()
                .and_then(|it| it.strip_prefix("   "))
                .ok_or_else(|| format!("a text with no content: {line}"))?;
            drawn.texts.push(DrawnText {
                x: number(line, "x")?,
                y: number(line, "y")?,
                size: number(line, "font-size")?,
                anchor: attribute(line, "text-anchor")?.to_string(),
                rotation: match attribute(line, "transform") {
                    Ok(transform) => rotation(transform)?,
                    Err(_) => 0,
                },
                bold: attribute(line, "font-weight") == Ok("bold"),
                text: unescape(content),
            });
        }
    }
    Ok(drawn)
}

/// The value of an attribute in a start tag.
///
/// Input:  `<circle cx="29" cy="28.87" r="8.215" stroke-width="1.569"/>`, `r`
/// Output: `8.215`
fn attribute<'a>(tag: &'a str, name: &str) -> Result<&'a str, String> {
    // The leading space keeps `width` from matching `stroke-width`.
    let (_, rest) = tag
        .split_once(&format!(" {name}=\""))
        .ok_or_else(|| format!("no {name} in {tag}"))?;
    let (value, _) = rest
        .split_once('"')
        .ok_or_else(|| format!("{name} is not closed in {tag}"))?;
    Ok(value)
}

fn number(tag: &str, name: &str) -> Result<f32, String> {
    let value = attribute(tag, name)?;
    value
        .parse()
        .map_err(|error| format!("{name}=\"{value}\" is not a number: {error}"))
}

fn parse_number(value: &str) -> Result<f32, String> {
    value
        .trim()
        .parse()
        .map_err(|error| format!("\"{value}\" is not a number: {error}"))
}

/// The closed shapes a path is made of.
///
/// Input:  `M0 0h4v10h-4ZM6 0h2v10h-2Z`
/// Output: `M0 0h4v10h-4`, `M6 0h2v10h-2`
fn shapes(path: &str) -> impl Iterator<Item = &str> {
    path.split('Z').filter(|shape| !shape.is_empty())
}

/// Input:  `M6 0h2v10h-2`
/// Output: x 6, y 0, width 2, height 10
fn parse_rectangle(shape: &str, fill: Option<String>) -> Result<DrawnRectangle, String> {
    let malformed = || format!("not a rectangle: {shape}");
    let rest = shape.strip_prefix('M').ok_or_else(malformed)?;
    let (x, rest) = rest.split_once(' ').ok_or_else(malformed)?;
    let (y, rest) = rest.split_once('h').ok_or_else(malformed)?;
    let (width, rest) = rest.split_once('v').ok_or_else(malformed)?;
    let (height, back) = rest.split_once("h-").ok_or_else(malformed)?;
    if back != width {
        return Err(malformed());
    }
    Ok(DrawnRectangle {
        x: parse_number(x)?,
        y: parse_number(y)?,
        width: parse_number(width)?,
        height: parse_number(height)?,
        fill,
    })
}

/// Input:  `M21 2.15L21.87 1.65L21.87 0.65L21 0.15L20.13 0.65L20.13 1.65`
/// Output: the six corners, in that order
fn parse_hexagon(shape: &str) -> Result<DrawnHexagon, String> {
    let rest = shape
        .strip_prefix('M')
        .ok_or_else(|| format!("not a hexagon: {shape}"))?;
    let corners = rest
        .split('L')
        .map(|corner| {
            let (x, y) = corner
                .split_once(' ')
                .ok_or_else(|| format!("not a corner: {corner}"))?;
            Ok((parse_number(x)?, parse_number(y)?))
        })
        .collect::<Result<Vec<_>, String>>()?;
    if corners.len() != 6 {
        return Err(format!("a hexagon with {} corners: {shape}", corners.len()));
    }
    Ok(corners)
}

/// Input:  `rotate(90,13.5,42)`
/// Output: `90`
fn rotation(transform: &str) -> Result<i32, String> {
    let degrees = transform
        .strip_prefix("rotate(")
        .and_then(|rest| rest.split_once(','))
        .map(|(degrees, _)| degrees)
        .ok_or_else(|| format!("not a rotation: {transform}"))?;
    degrees
        .parse()
        .map_err(|error| format!("\"{degrees}\" is not a rotation: {error}"))
}

/// Undoes the escaping zint's SVG writer applies to text.
///
/// Input:  `A&amp;B &lt;1&gt;`
/// Output: `A&B <1>`
fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
