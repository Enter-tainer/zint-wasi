//! Tests for the `fm_printf` that `patch/filemem.c` puts in front of zint's
//! `vsnprintf` based one.
//!
//! Every case is written twice into an in-memory file, once by the fast path
//! and once by zint's original, and the two have to come out byte for byte the
//! same. A format the fast path does not take is checked the same way, which is
//! what shows it was handed to the original rather than misread.

use std::ffi::{c_char, c_int, CStr, CString};

use zint_wasm_sys::{zint_symbol, ZBarcode_Create, ZBarcode_Delete, BARCODE_MEMORY_FILE};

extern "C" {
    /// Writes one `fm_printf` into the symbol's in-memory file: through the
    /// fast path when `fast` is non-zero, else through zint's original.
    fn zint_wasi_fm_printf_to_memory(
        symbol: *mut zint_symbol,
        fast: c_int,
        fmt: *const c_char,
        ...
    ) -> c_int;
}

/// One argument to a format, as C's default promotions pass it.
#[derive(Clone, Copy)]
enum Arg<'a> {
    Int(c_int),
    Str(&'a CStr),
}

/// What one path writes for `fmt` and `args`.
fn render(fast: bool, fmt: &str, args: &[Arg]) -> Vec<u8> {
    let fmt = CString::new(fmt).expect("formats hold no NUL");
    let fast = c_int::from(fast);

    unsafe {
        // Safety: the symbol comes from zint and is deleted below; every format
        // passed takes exactly the arguments given with it.
        let symbol = ZBarcode_Create();
        assert!(!symbol.is_null(), "zint could not allocate a symbol");
        (*symbol).output_options = BARCODE_MEMORY_FILE as c_int;

        let f = zint_wasi_fm_printf_to_memory;
        let p = fmt.as_ptr();
        let ok = match *args {
            [] => f(symbol, fast, p),
            [Arg::Int(a)] => f(symbol, fast, p, a),
            [Arg::Str(a)] => f(symbol, fast, p, a.as_ptr()),
            [Arg::Int(a), Arg::Int(b)] => f(symbol, fast, p, a, b),
            [Arg::Int(a), Arg::Str(b)] => f(symbol, fast, p, a, b.as_ptr()),
            _ => panic!("no case passes these arguments"),
        };
        assert_eq!(ok, 1, "writing {fmt:?} failed");

        let written =
            std::slice::from_raw_parts((*symbol).memfile, (*symbol).memfile_size as usize).to_vec();
        ZBarcode_Delete(symbol);
        written
    }
}

/// Both paths agree, and write `expected`.
fn assert_writes(fmt: &str, args: &[Arg], expected: &str) {
    let original = render(false, fmt, args);
    assert_eq!(
        String::from_utf8_lossy(&original),
        expected,
        "zint's own fm_printf wrote something else for {fmt:?}; the expectation is wrong"
    );
    assert_eq!(
        String::from_utf8_lossy(&render(true, fmt, args)),
        expected,
        "the fast path disagrees with zint for {fmt:?}"
    );
}

#[test]
fn a_format_without_conversions_is_copied() {
    assert_writes("", &[], "");
    assert_writes(
        " <desc>Zint Generated Symbol</desc>\n",
        &[],
        " <desc>Zint Generated Symbol</desc>\n",
    );
}

#[test]
fn integers_are_written_in_decimal() {
    for (value, text) in [
        (c_int::MIN, "-2147483648"),
        (-1, "-1"),
        (0, "0"),
        (1, "1"),
        (216, "216"),
        (c_int::MAX, "2147483647"),
    ] {
        assert_writes("%d", &[Arg::Int(value)], text);
    }
}

#[test]
fn strings_are_copied() {
    assert_writes("%s", &[Arg::Str(c"")], "");
    assert_writes("#%s", &[Arg::Str(c"1F4788")], "#1F4788");
    // Already escaped by zint, so passed through as it is.
    assert_writes(
        "   %s\n",
        &[Arg::Str(c"A&amp;B &lt;1&gt; Grüße")],
        "   A&amp;B &lt;1&gt; Grüße\n",
    );
}

/// Zint's memory file grows in 32 KiB pages; a string longer than one has to
/// make it grow rather than be cut off.
#[test]
fn a_string_longer_than_a_page_of_the_memory_file_is_written_whole() {
    let long = CString::new("x".repeat(100_000)).unwrap();
    let written = render(true, "<%s>", &[Arg::Str(&long)]);

    assert_eq!(written.len(), 100_002);
    assert_eq!(written, render(false, "<%s>", &[Arg::Str(&long)]));
}

/// The formats `svg.c` actually writes with, filled the way it fills them.
#[test]
fn the_formats_zint_writes_svg_with_come_out_the_same() {
    assert_writes(
        "<svg width=\"%d\" height=\"%d\" version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\n",
        &[Arg::Int(224), Arg::Int(117)],
        "<svg width=\"224\" height=\"117\" version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\n",
    );
    assert_writes(
        " <g id=\"barcode\" fill=\"#%s\">\n",
        &[Arg::Str(c"000000")],
        " <g id=\"barcode\" fill=\"#000000\">\n",
    );
    assert_writes(
        "  <rect x=\"0\" y=\"0\" width=\"%d\" height=\"%s\"",
        &[Arg::Int(72), Arg::Str(c"24")],
        "  <rect x=\"0\" y=\"0\" width=\"72\" height=\"24\"",
    );
    assert_writes(
        " transform=\"rotate(%d",
        &[Arg::Int(270)],
        " transform=\"rotate(270",
    );
    assert_writes(
        " font-family=\"%s, monospace\"",
        &[Arg::Str(c"OCRB")],
        " font-family=\"OCRB, monospace\"",
    );
}

/// Anything other than a bare `%d` or `%s` is left to zint's own `fm_printf`.
/// Read as a bare conversion, each of these would write something else.
#[test]
fn other_conversions_are_left_to_zint() {
    assert_writes("%5d|", &[Arg::Int(42)], "   42|");
    assert_writes("%-3d|", &[Arg::Int(7)], "7  |");
    assert_writes("%02X", &[Arg::Int(0x0A)], "0A");
    assert_writes("%x", &[Arg::Int(255)], "ff");
    assert_writes("%.3s", &[Arg::Str(c"abcdef")], "abc");
    assert_writes("100%%", &[], "100%");
    assert_writes("%d%%", &[Arg::Int(5)], "5%");
    // One conversion it does not take sends the whole format to zint, the
    // ones it would take included.
    assert_writes("%d %02d", &[Arg::Int(3), Arg::Int(4)], "3 04");
}
