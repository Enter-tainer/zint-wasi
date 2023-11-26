#import "./lib.typ": *
#show: doc => columns(2, doc)
#let barcode-height = 5em
#let left-right(..args) = grid(columns: (1fr, auto), gutter: 1%, ..args)
= EAN
== EAN-13
#left-right[
  ```typ
  #ean("6975004310001")
  ```
][
  #align(right)[#ean("6975004310001", height: barcode-height)]
]

== EAN-8

#left-right[
  ```typ
  #ean("12345564")
  ```
][
  #align(right)[#ean("12345564", height: barcode-height)]
]

== EAN-5

#left-right[
  ```typ
  #ean("12345")
  ```
][
  #align(right)[#ean("12345", height: barcode-height)]
]

== EAN-2

#left-right[
  ```typ
  #ean("12")
  ```
][
  #align(right)[#ean("12", height: barcode-height)]
]

= Code 128

#left-right[
  ```typ
  #code128("1234567890")
  ```
][
  #align(right)[#code128("1234567890", height: barcode-height)]
]

= Code 39

#left-right[
  ```typ
  #code39("ABCD")
  ```
][
  #align(right)[#code39("ABCD", height: barcode-height)]
]

= UPCA

#left-right[
  ```typ
  #upca("123456789012")
  ```
][
  #align(right)[#upca("123456789012", height: barcode-height)]
]

= Data Matrix

#left-right[
  ```typ
  #data-matrix("1234567890")
  ```
][
  #align(right)[#data-matrix("1234567890", height: barcode-height)]
]

= QR Code

#left-right[
  ```typ
  #qrcode("1234567890")
  ```
][
  #align(right)[#qrcode("1234567890", height: barcode-height)]
]

= Channel Code

#left-right[
  ```typ
  #channel("1234567")
  ```
][
  #align(right)[#channel("1234567", height: barcode-height)]
]

= MSI Plessey

#left-right[
  ```typ
  #msi-plessey("1234567")
  ```
][
  #align(right)[#msi-plessey("1234567", width: 7em)]
]

// = Micro PDF417

// #left-right[
//   ```typ
//   #micro-pdf417("1234567890")
//   ```
// ][
//   #align(right)[#micro-pdf417("1234567890", height: barcode-height)]
// ]

= Aztec Code

#left-right[
  ```typ
  #aztec("1234567890")
  ```
][
  #align(right)[#aztec("1234567890", height: barcode-height)]
]

= Code 16k

#left-right[
  ```typ
  #code16k("1234567890")
  ```
][
  #align(right)[#code16k("1234567890", width: 7em)]
]

= MaxiCode

#left-right[
  ```typ
  #maxicode("1234567890")
  ```
][
  #align(right)[#maxicode("1234567890", height: barcode-height)]
]

= Planet Code

#left-right[
  ```typ
  #planet("1234567890")
  ```
][
  #align(right)[#planet("1234567890", width: 9em)]
]

= Others

== C25Standard

#left-right[
  ```typ
  #barcode("123", "C25Standard")
  ```
][
  #align(right)[#barcode("123", "C25Standard", height: barcode-height)]
]

== UPCE

#left-right[
  ```typ
  #barcode("1234567", "UPCEChk")
  ```
][
  #align(right)[#barcode("1234567", "UPCEChk", width: barcode-height)]
]

== MicroQR

#left-right[
  ```typ
  #barcode("1234567890", "MicroQR")
  ```
][
  #align(right)[#barcode("1234567890", "MicroQR", width: 3em)]
]

== Aztec Runes

#left-right[
  ```typ
  #barcode("1234567890", "AzRune")
  ```
][
  #align(right)[#barcode("122", "AzRune", height: 3em)]
]

== Australia Post

#left-right[
  ```typ
  #barcode("1234567890", "AusPost")
  ```
][
  #align(right)[#barcode("12345678", "AusPost", width: 9em)]
]

== DotCode

#left-right[
  ```typ
  #barcode("1234567890", "DotCode")
  ```
][
  #align(right)[#barcode("1234567890", "DotCode", width: 3em)]
]

== CodeOne

#left-right[
  ```typ
  #barcode("1234567890", "CodeOne")
  ```
][
  #align(right)[#barcode("1234567890", "CodeOne", height: 3em)]
]

== Grid Matrix

#left-right[
  ```typ
  #barcode("1234567890", "GridMatrix")
  ```
][
  #align(right)[#barcode("1234567890", "GridMatrix", width: 2em)]
]

== Han Xin Code

#left-right[
  ```typ
  #barcode("1234567890", "HanXin")
  ```
][
  #align(right)[#barcode("1234567890", "HanXin", width: 3em)]
]

== Code128B

#left-right[
  ```typ
  #barcode("1234567890", "Code128B")
  ```
][
  #align(right)[#barcode("1234567890", "Code128B", height: 3em)]
]
