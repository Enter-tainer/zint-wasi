#import "./lib.typ": *

#let barcode-height = 5em
#columns()[
= EAN
== EAN-13
#grid(columns: (1fr, 1fr), gutter: 4%)[
  ```typ
  #ean("6975004310001")
  ```
][
  #align(right)[#ean("6975004310001", height: barcode-height)]
]

== EAN-8

#grid(columns: (1fr, 1fr), gutter: 4%)[
  ```typ
  #ean("12345564")
  ```
][
  #align(right)[#ean("12345564", height: barcode-height)]
]

== EAN-5

#grid(columns: (1fr, 1fr), gutter: 4%)[
  ```typ
  #ean("12345")
  ```
][
  #align(right)[#ean("12345", height: barcode-height)]
]

== EAN-2

#grid(columns: (1fr, 1fr), gutter: 4%)[
  ```typ
  #ean("12")
  ```
][
  #align(right)[#ean("12", height: barcode-height)]
]
]
