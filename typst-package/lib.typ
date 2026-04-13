// Core barcode function
#import "core.typ": barcode

// Generated shortcut functions
#import "wrappers.typ": *

/// Returns #typst-type("int") option value for given Data Matrix _width_ and _height_.
///
/// Zint allows square and rectangular values to be enforced with `DM_SQUARE` and `DM_DMRE` Option 3 values.
///
/// - width (int): Data Matrix width
/// - height (int): Data Matrix height
/// -> int
#let dm-size(height, width) = {
  // Copied from DM size table
  if height == 10 and width == 10 { return int(1) }
  if height == 12 and width == 12 { return int(2) }
  if height == 14 and width == 14 { return int(3) }
  if height == 16 and width == 16 { return int(4) }
  if height == 18 and width == 18 { return int(5) }
  if height == 20 and width == 20 { return int(6) }
  if height == 22 and width == 22 { return int(7) }
  if height == 24 and width == 24 { return int(8) }
  if height == 26 and width == 26 { return int(9) }
  if height == 32 and width == 32 { return int(10) }
  if height == 36 and width == 36 { return int(11) }
  if height == 40 and width == 40 { return int(12) }
  if height == 44 and width == 44 { return int(13) }
  if height == 48 and width == 48 { return int(14) }
  if height == 52 and width == 52 { return int(15) }
  if height == 64 and width == 64 { return int(16) }
  if height == 72 and width == 72 { return int(17) }
  if height == 80 and width == 80 { return int(18) }
  if height == 88 and width == 88 { return int(19) }
  if height == 96 and width == 96 { return int(20) }
  if height == 104 and width == 104 { return int(21) }
  if height == 120 and width == 120 { return int(22) }
  if height == 132 and width == 132 { return int(23) }
  if height == 144 and width == 144 { return int(24) }
  if height == 8 and width == 18 { return int(25) }
  if height == 8 and width == 32 { return int(26) }
  if height == 12 and width == 26 { return int(28) }
  if height == 12 and width == 36 { return int(28) }
  if height == 16 and width == 36 { return int(29) }
  if height == 16 and width == 48 { return int(30) }
  // DMRE table
  if height == 8 and width == 48 { return int(31) }
  if height == 8 and width == 64 { return int(32) }
  if height == 8 and width == 80 { return int(33) }
  if height == 8 and width == 96 { return int(34) }
  if height == 8 and width == 120 { return int(35) }
  if height == 8 and width == 144 { return int(36) }
  if height == 12 and width == 64 { return int(37) }
  if height == 12 and width == 88 { return int(38) }
  if height == 16 and width == 64 { return int(39) }
  if height == 20 and width == 36 { return int(40) }
  if height == 20 and width == 44 { return int(41) }
  if height == 20 and width == 64 { return int(42) }
  if height == 22 and width == 48 { return int(43) }
  if height == 24 and width == 48 { return int(44) }
  if height == 24 and width == 64 { return int(45) }
  if height == 26 and width == 40 { return int(46) }
  if height == 26 and width == 48 { return int(47) }
  if height == 26 and width == 64 { return int(48) }
  panic("Data Matrix with dimensions " + str(width) + "x" + str(height) + " not supported")
}

#let barcode-primary(primary, data, type, options: (:), ..args) = barcode(
  data,
  type,
  options: (primary: primary, ..options),
  ..args,
)

#let barcode-composite(
  primary, data, mode, type, options: (:), ..args,
) = barcode-primary(
  primary, data, type,
  options: (option_1: int(mode), ..options),
  ..args,
)
