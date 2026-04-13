// Shared utilities for manual and generated docs.

#let _rainbow = gradient.linear(angle: 7deg,
  (rgb("#7cd5ff"), 0%), (rgb("#a6fbca"), 33%),
  (rgb("#fff37c"), 66%), (rgb("#ffa49d"), 100%),
)

#let _type-colors = (
  "content": rgb("#a6ebe6"),
  "str": rgb("#d1ffe2"), "string": rgb("#d1ffe2"),
  "none": rgb("#ffcbc4"), "auto": rgb("#ffcbc4"),
  "bool": rgb("#ffedc1"), "boolean": rgb("#ffedc1"),
  "int": rgb("#e7d9ff"), "integer": rgb("#e7d9ff"),
  "float": rgb("#e7d9ff"), "length": rgb("#e7d9ff"),
  "array": rgb("#eff0f3"), "dictionary": rgb("#eff0f3"),
  "color": _rainbow, "gradient": _rainbow,
  "function": rgb("#f9dfff"),
  "stroke": rgb("#eff0f3"),
)

#let _type-docs = (
  "content": "https://typst.app/docs/reference/foundations/content/",
  "str": "https://typst.app/docs/reference/foundations/str/",
  "string": "https://typst.app/docs/reference/foundations/str/",
  "none": "https://typst.app/docs/reference/foundations/none/",
  "auto": "https://typst.app/docs/reference/foundations/auto/",
  "bool": "https://typst.app/docs/reference/foundations/bool/",
  "boolean": "https://typst.app/docs/reference/foundations/bool/",
  "int": "https://typst.app/docs/reference/foundations/int/",
  "integer": "https://typst.app/docs/reference/foundations/int/",
  "float": "https://typst.app/docs/reference/foundations/float/",
  "length": "https://typst.app/docs/reference/foundations/length/",
  "array": "https://typst.app/docs/reference/foundations/array/",
  "dictionary": "https://typst.app/docs/reference/foundations/dictionary/",
  "color": "https://typst.app/docs/reference/visualize/color/",
  "gradient": "https://typst.app/docs/reference/visualize/gradient/",
  "stroke": "https://typst.app/docs/reference/visualize/stroke/",
  "function": "https://typst.app/docs/reference/foundations/function/",
)

/// Render a colored type badge, linked to typst docs when available.
#let typst-type(..types, docs: (:)) = {
  let all-docs = _type-docs + docs
  for (i, ty) in types.pos().enumerate() {
    if i > 0 {
      h(2pt)
      text(size: 9pt)[or]
      h(2pt)
    }
    let clr = _type-colors.at(ty, default: rgb("#eff0f3"))
    let badge = box(outset: 2pt, fill: clr, radius: 2pt, raw(ty))
    let dest = all-docs.at(ty, default: none)
    h(2pt)
    if dest != none { link(dest, badge) } else { badge }
    h(2pt)
  }
}

/// Render a colored value literal.
#let typst-val(v) = {
  let fg = black
  if v.starts-with("\"") { fg = _type-colors.at("str").darken(50%).saturate(70%) }
  else if v == "true" or v == "false" { fg = _type-colors.at("bool").darken(50%).saturate(70%) }
  else if v == "none" { fg = _type-colors.at("none").darken(50%).saturate(70%) }
  else { fg = _type-colors.at("float").darken(50%).saturate(70%) }
  text(fill: fg, raw(v))
}

#let l(dest, body) = underline(link(dest, body))

#let ref-table(columns: (auto, 100pt, 1fr, auto), head-rows: 1, key-column: true, ..cells) = {
  table(
    columns: columns,
    align: (center + horizon, center + horizon, left + horizon, center + horizon),
    stroke: gray.lighten(60%),
    fill: (col, row) => if row < head-rows {
      blue.lighten(80% - 10% * (head-rows - 1 - row))
    } else if col == 0 and key-column {
      blue.lighten(90%)
    },
    ..cells,
  )
}
