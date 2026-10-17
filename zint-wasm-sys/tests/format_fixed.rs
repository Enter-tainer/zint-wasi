//! Tests for the coordinate formatter that `patch/filemem.c` puts in front of
//! zint's `sprintf` based one.
//!
//! The SVG zint writes has to stay byte for byte what `sprintf("%.*f")` and
//! zint's trimming of trailing zeros made of it, so every test here compares
//! against that: Rust's `{:.*}` formats the exact value of the float and rounds
//! a tie to even, which is what C's `printf` does too.

use std::ffi::{c_char, c_int, CStr};

// Linked for `zint_wasi_format_fixed`, which lives in the static zint library
// the crate builds and has no binding of its own.
use zint_wasm_sys as _;

/// Room for the longest number the fast path writes, as `patch/filemem.c`
/// defines it.
const BUFFER_SIZE: usize = 32;

extern "C" {
    fn zint_wasi_format_fixed(buf: *mut c_char, dp: c_int, arg: f32) -> c_int;
}

/// The fast path's text for `value`, or `None` where it leaves the value to
/// `sprintf`.
fn fast(dp: i32, value: f32) -> Option<String> {
    let mut buffer = [0 as c_char; BUFFER_SIZE];
    let length = unsafe {
        // Safety: the buffer has the size the C side writes at most.
        zint_wasi_format_fixed(buffer.as_mut_ptr(), dp, value)
    };
    if length == 0 {
        return None;
    }

    let text = unsafe {
        // Safety: a non-zero return means a terminated string was written.
        CStr::from_ptr(buffer.as_ptr())
    };
    let text = text.to_str().expect("only digits, a sign and a point");
    assert_eq!(
        text.len(),
        length as usize,
        "the returned length disagrees with the string for {value} at {dp} places"
    );
    Some(text.to_owned())
}

/// What zint writes for `value`: `sprintf("%.*f")`, then the trailing zeros
/// of the fraction and a point left bare by that dropped.
///
/// Input:  `1.5`, `2` places, which `sprintf` makes `"1.50"`
/// Output: `"1.5"`
fn zint(dp: i32, value: f32) -> String {
    let printed = format!("{:.*}", dp as usize, f64::from(value));
    if printed.contains('.') {
        printed
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    } else {
        printed
    }
}

fn assert_formats_like_zint(dp: i32, value: f32) {
    assert_eq!(
        fast(dp, value).as_deref(),
        Some(zint(dp, value).as_str()),
        "{value:e} ({:#010x}) at {dp} places",
        value.to_bits()
    );
}

/// The decimal places zint asks for, plus the ends of what the fast path
/// takes, where `0` has no fraction at all.
const PLACES: [i32; 4] = [0, 2, 4, 9];

#[test]
fn whole_numbers_drop_the_fraction() {
    // The common case: zint lays modules out on whole units.
    assert_eq!(fast(2, 0.0).as_deref(), Some("0"));
    assert_eq!(fast(2, 1.0).as_deref(), Some("1"));
    assert_eq!(fast(2, 100.0).as_deref(), Some("100"));
    assert_eq!(fast(0, 20.0).as_deref(), Some("20"));
}

#[test]
fn trailing_zeros_of_the_fraction_are_dropped() {
    assert_eq!(fast(2, 53.5).as_deref(), Some("53.5"));
    assert_eq!(fast(2, 53.25).as_deref(), Some("53.25"));
    assert_eq!(fast(4, 0.0625).as_deref(), Some("0.0625"));
    // The zeros between the point and the first digit stay.
    assert_eq!(fast(4, 1.0 / 128.0).as_deref(), Some("0.0078"));
}

#[test]
fn values_round_to_the_nearest_place() {
    // 53.34 is not exact in binary; it is held as 53.340000152587890625.
    assert_eq!(fast(2, 53.34).as_deref(), Some("53.34"));
    assert_eq!(fast(2, 2.0 / 3.0).as_deref(), Some("0.67"));
    assert_eq!(fast(2, 0.996).as_deref(), Some("1"));
    assert_eq!(fast(0, 0.75).as_deref(), Some("1"));
}

/// A value exactly halfway between two outputs rounds to the even one, as C's
/// `printf` does. Ties only exist for a fraction of `2^-(dp + 1)`, so `0.125`
/// at two places, `0.03125` at four.
#[test]
fn a_tie_rounds_to_the_even_neighbour() {
    assert_eq!(fast(2, 0.125).as_deref(), Some("0.12"));
    assert_eq!(fast(2, 0.375).as_deref(), Some("0.38"));
    assert_eq!(fast(2, 0.625).as_deref(), Some("0.62"));
    assert_eq!(fast(2, 1.875).as_deref(), Some("1.88"));
    assert_eq!(fast(4, 0.03125).as_deref(), Some("0.0312"));
    assert_eq!(fast(4, 0.09375).as_deref(), Some("0.0938"));
    assert_eq!(fast(0, 0.5).as_deref(), Some("0"));
    assert_eq!(fast(0, 1.5).as_deref(), Some("2"));
    assert_eq!(fast(0, 2.5).as_deref(), Some("2"));
    // 1/1024 is 0.0009765625, so the ninth place is a tie too.
    assert_eq!(fast(9, 1.0 / 1024.0).as_deref(), Some("0.000976562"));
    assert_eq!(fast(2, -0.125).as_deref(), Some("-0.12"));
}

/// `sprintf` keeps the sign of a negative value that rounds to zero, and so
/// writes `-0`. Zint's output has always carried that, so it stays.
#[test]
fn a_negative_value_keeps_its_sign_when_it_rounds_to_zero() {
    assert_eq!(fast(2, -1.5).as_deref(), Some("-1.5"));
    assert_eq!(fast(2, -0.001).as_deref(), Some("-0"));
    assert_eq!(fast(2, -0.0).as_deref(), Some("-0"));
}

/// Values the fast path cannot format exactly are left to `sprintf`, which
/// writes them the way zint always has.
#[test]
fn what_the_fast_path_does_not_cover_is_left_to_sprintf() {
    assert_eq!(fast(2, f32::NAN), None);
    assert_eq!(fast(2, f32::INFINITY), None);
    assert_eq!(fast(2, f32::NEG_INFINITY), None);
    assert_eq!(fast(2, f32::MAX), None);
    assert_eq!(fast(2, 1e14), None);
    assert_eq!(fast(-1, 1.0), None);
    assert_eq!(fast(10, 1.0), None);
    assert_eq!(fast(i32::MIN, 1.0), None);
    assert_eq!(fast(i32::MAX, 1.0), None);

    // Just inside the limit is still formatted.
    assert_formats_like_zint(2, 9e12);
    assert_formats_like_zint(9, 999_999.9);
}

/// Every multiple of 1/64 over the size range of real barcodes, which is
/// where all the ties are and where every coordinate zint draws lies.
#[test]
fn every_fine_fraction_over_the_drawing_range_matches_zint() {
    for step in -(2048 * 64)..=(2048 * 64) {
        let value = step as f32 / 64.0;
        for dp in PLACES {
            assert_formats_like_zint(dp, value);
        }
    }
}

/// A spread over every exponent a float has, for the values that are not a
/// fine fraction. Each one is either formatted as zint would, or left to
/// `sprintf`.
#[test]
fn a_spread_of_all_floats_matches_zint_or_is_left_to_sprintf() {
    // A prime stride, so the low bits of the significand vary too.
    for bits in (0..=u32::MAX).step_by(65_521) {
        let value = f32::from_bits(bits);
        for dp in PLACES {
            match fast(dp, value) {
                Some(text) => assert_eq!(
                    text,
                    zint(dp, value),
                    "{value:e} ({bits:#010x}) at {dp} places"
                ),
                None => assert!(
                    !value.is_finite() || f64::from(value).abs() * 10f64.powi(dp) >= 1e15,
                    "{value:e} ({bits:#010x}) at {dp} places was left to sprintf"
                ),
            }
        }
    }
}
