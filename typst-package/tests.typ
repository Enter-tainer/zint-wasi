// Checks the package's drawing against what `image` does with the same
// arguments, and its handling of colours.
//
// A failed assertion fails the compile, which is how `cargo xtask test-package`
// reports it. Run it on its own with
//
//   typst compile typst-package/tests.typ
//
// once the plugin has been built into `typst-package/`.

#import "lib.typ": *

// Lengths that went through floating point on the way, compared to well
// within anything a reader could see.
#let assert-close(actual, expected, message) = {
  assert(
    calc.abs((actual - expected).to-absolute() / 1pt) < 0.001,
    message: message + ": " + repr(actual) + " is not " + repr(expected),
  )
}

#let assert-size(it, width, height, message, available: auto) = context {
  let size = if available == auto { measure(it) } else { measure(it, width: available) }
  assert-close(size.width, width, message + ", width")
  assert-close(size.height, height, message + ", height")
}

// Code 128 "A12345B" is 224 by 116.28 units in Zint's layout.
#let code = code128.with("A12345B")
#let natural-width = 224 * 0.75pt
#let natural-height = 116.28 * 0.75pt

// The size the barcode had when the package embedded Zint's SVG, which Typst
// read at 96 units to the inch. Zint rounded the SVG's height up to whole
// units; the drawing keeps the exact one.
#context {
  let svg = measure(image(
    bytes("<svg width=\"224\" height=\"117\" xmlns=\"http://www.w3.org/2000/svg\"/>"),
    format: "svg",
  ))
  assert-close(measure(code()).width, svg.width, "the barcode keeps the width its SVG had")
}
#assert-size(code(), natural-width, natural-height, "with no size given")

// No size given, but less room than the natural size: the barcode shrinks to
// fit, keeping its aspect ratio, as an image would.
#assert-size(code(), 100pt, 100pt * (116.28 / 224), "with no size given in 100pt", available: 100pt)

// One side given: the other keeps the aspect ratio.
#assert-size(code(width: 4cm), 4cm, 4cm * (116.28 / 224), "with a width")
#assert-size(code(height: 1cm), 1cm * (224 / 116.28), 1cm, "with a height")
#assert-size(code(width: 50%), 100pt, 100pt * (116.28 / 224), "with half of 200pt", available: 200pt)
#assert-size(code(width: 50% + 10pt), 110pt, 110pt * (116.28 / 224), "with a relative width", available: 200pt)
#context assert-size(code(width: 2em), 2em.to-absolute(), 2em.to-absolute() * (116.28 / 224), "with an em width")

// Both sides given: the barcode takes exactly that space, whatever `fit` says
// about the drawing inside it.
#for fit in ("cover", "contain", "stretch") {
  assert-size(code(width: 3cm, height: 3cm, fit: fit), 3cm, 3cm, "fit " + fit)
}

// And inside that space, what `fit` does with a drawing twice as wide as it is
// tall, put into a square. Only "cover" overflows, so only it clips.
#let square = _fit((200pt, 100pt), (100pt, 100pt), (500pt, 500pt), "contain")
#assert.eq((square.fitted-width, square.fitted-height, square.clip), (100pt, 50pt, false))
#let square = _fit((200pt, 100pt), (100pt, 100pt), (500pt, 500pt), "cover")
#assert.eq((square.fitted-width, square.fitted-height, square.clip), (200pt, 100pt, true))
#let square = _fit((200pt, 100pt), (100pt, 100pt), (500pt, 500pt), "stretch")
#assert.eq((square.fitted-width, square.fitted-height, square.clip), (100pt, 100pt, false))

// With no size given nothing overflows, whatever `fit` is, so human readable
// text set larger than Zint laid it out for is not cut off.
#for fit in ("cover", "contain", "stretch") {
  let natural = _fit((200pt, 100pt), (auto, auto), (500pt, 500pt), fit)
  assert.eq((natural.width, natural.height, natural.clip), (200pt, 100pt, false), message: fit)
  let shrunk = _fit((200pt, 100pt), (auto, auto), (100pt, 500pt), fit)
  assert.eq((shrunk.width, shrunk.height, shrunk.clip), (100pt, 50pt, false), message: fit)
}

// A ratio of a side the region does not bound comes to nothing, as it does for
// an `image`, rather than to an infinite size no shape can be drawn at.
#assert.eq(_resolve(50%, float.inf * 1pt, "width"), 0pt)
#context assert.eq(_resolve(50% + 10pt, float.inf * 1pt, "width"), 10pt)

// The colours, given directly or through any spelling of the options they
// used to be. Every one of those spellings has to land somewhere, since the
// plugin refuses them all.
#let fg = _foreground-options
#let bg = _background-options
#assert.eq(_aliased(auto, (:), fg, "fill", black), black)
#assert.eq(_aliased(auto, (:), bg, "background", none), none)
#assert.eq(_aliased(red, (:), fg, "fill", black), red)
#assert.eq(_aliased(auto, (fg-color: "1F4788"), fg, "fill", black), rgb("#1F4788"))
#assert.eq(_aliased(auto, (fg-color: "#1F478880"), fg, "fill", black), rgb("#1F478880"))
#assert.eq(_aliased(auto, (fg-colour: blue), fg, "fill", black), blue)
#assert.eq(_aliased(auto, (stroke: "1F4788"), fg, "fill", black), rgb("#1F4788"))
#assert.eq(_aliased(auto, (bg-color: yellow), bg, "background", none), yellow)
#assert.eq(_aliased(auto, (bg-colour: yellow), bg, "background", none), yellow)
#assert.eq(_aliased(auto, (fill: "FFD700"), bg, "background", none), rgb("#FFD700"))
#assert.eq(_aliased(auto, (fg-color: none), fg, "fill", black), black)
#assert.eq(_paint(none, "background"), none)

// A CMYK colour is the point of drawing with Typst's shapes, so it has to come
// through as CMYK rather than be converted along the way.
#let black-ink = cmyk(0%, 0%, 0%, 100%)
#assert.eq(_paint(black-ink, "fill"), black-ink)
#assert.eq(_paint(black-ink, "fill").space(), cmyk)
// Gradients never compare equal, not even to themselves, so only the type is
// checked.
#assert.eq(type(_paint(gradient.linear(red, blue), "fill")), gradient)

// Every shortcut still draws, with every argument a document can give it.
#context {
  let drawn = (
    code128("A12345B", fill: black-ink, background: white, hrt: (size: 8pt)),
    qrcode("https://typst.app", options: (fg-color: "1F4788", bg-color: "FFD700")),
    maxicode("1234567890", width: 1cm),
    ultra("https://aimglobal.org/jcrv3tX", height: 1cm),
    qrcode("12345678", options: (output-options: (barcode-dotty-mode: true))),
    eanx-cc("331234567890", "[99]1234-abcd", fit: "contain", width: 2cm, height: 2cm),
  )
  for it in drawn {
    assert(measure(it).width > 0pt, message: "a barcode drew nothing")
  }
}
