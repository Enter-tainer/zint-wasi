#import "./lib.typ": *
#show: doc => columns(2, doc)
#let barcode-height = 5em
#let left-right(..args) = grid(columns: (auto, 1fr), gutter: 1%, ..args)
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
  #upca("12345678901")
  ```
][
  #align(right)[#upca("12345678901", height: barcode-height)]
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
  #align(right)[#planet("1234567890", width: 7em)]
]
