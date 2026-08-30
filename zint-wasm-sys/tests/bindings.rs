//! Smoke tests over the generated bindings.
//!
//! Nothing in this crate is checked by the compiler alone: `bindgen` rewrites
//! the bindings from the vendored `zint.h` on every build, and the C library is
//! compiled from the submodule next to it. These tests are what notices when
//! either of the two moves.

use std::ffi::CStr;

use zint_wasm_sys::*;

/// The libzint release the submodule is pinned to.
///
/// `ZBarcode_Version` packs the version as
/// `major * 10000 + minor * 100 + release * 10 + build`, so 21309 is 2.13.0.9.
/// Change this together with the submodule, and expect the golden files in
/// `zint-wasm-rs` to move with it.
const PINNED_VERSION: i32 = 21309;

#[test]
fn the_vendored_library_is_the_release_the_submodule_pins() {
    let version = unsafe {
        // Safety: takes no arguments and only reads compile time constants.
        ZBarcode_Version()
    };

    assert_eq!(
        version, PINNED_VERSION,
        "the vendored libzint changed; regenerate the golden files and update this version"
    );
}

/// Points a symbol at an SVG "file" in memory.
///
/// The extension of `outfile` is what selects the output format, even when the
/// symbol renders into memory rather than onto disk: without it zint reaches
/// for the raster backend, which this build patches out.
///
/// # Safety
///
/// `symbol` must point at a symbol zint created.
unsafe fn render_svg_into_memory(symbol: *mut zint_symbol) {
    let outfile = c"out.svg";
    (*symbol).output_options |= BARCODE_MEMORY_FILE as i32;
    std::ptr::copy_nonoverlapping(
        outfile.as_ptr(),
        (*symbol).outfile.as_mut_ptr(),
        outfile.to_bytes_with_nul().len(),
    );
}

/// The whole path the plugin depends on, in one test: allocate a symbol, encode
/// into the in-memory file, read the SVG back, free it.
#[test]
fn a_symbol_can_be_created_encoded_and_deleted() {
    let symbol = unsafe {
        // Safety: allocates and initialises a symbol, or returns null.
        ZBarcode_Create()
    };
    assert!(!symbol.is_null(), "zint could not allocate a symbol");

    let data = c"A12345B";
    let result = unsafe {
        // Safety: `symbol` is the symbol allocated above and `data` is a
        // NUL terminated string, which is what a length of 0 asks for.
        (*symbol).symbology = BARCODE_CODE128 as i32;
        render_svg_into_memory(symbol);
        ZBarcode_Encode_and_Print(symbol, data.as_ptr() as *const u8, 0, 0)
    };
    assert_eq!(result, 0, "Code 128 encodes alphanumeric data");

    let svg = unsafe {
        // Safety: a successful encode with BARCODE_MEMORY_FILE set leaves the
        // output in `memfile`, of length `memfile_size`.
        std::slice::from_raw_parts((*symbol).memfile, (*symbol).memfile_size as usize)
    };
    let svg = String::from_utf8_lossy(svg);
    assert!(svg.contains("<svg "), "the memory file holds an SVG");

    unsafe {
        // Safety: `symbol` was allocated by zint and is not used afterwards.
        ZBarcode_Delete(symbol)
    };
}

/// The wrapper copies strings into these buffers and decides from their size
/// whether a value fits, so the sizes are part of the contract.
#[test]
fn the_fixed_size_fields_are_the_sizes_the_wrapper_relies_on() {
    let symbol = unsafe {
        // Safety: see above.
        ZBarcode_Create()
    };
    assert!(!symbol.is_null(), "zint could not allocate a symbol");
    let fields = unsafe {
        // Safety: `symbol` is not null and zint initialised it.
        &*symbol
    };

    assert_eq!(
        fields.fgcolour.len(),
        16,
        "eight hex digits, or a decimal CMYK string, and a terminator"
    );
    assert_eq!(fields.bgcolour.len(), 16);
    assert_eq!(fields.outfile.len(), 256);
    assert_eq!(
        fields.primary.len(),
        128,
        "a composite symbol's linear component"
    );
    assert_eq!(fields.errtxt.len(), 100);

    unsafe {
        // Safety: see above.
        ZBarcode_Delete(symbol)
    };
}

/// Zint explains every failure in `errtxt`, in far more detail than the return
/// code carries.
#[test]
fn a_failed_encode_leaves_an_explanation_behind() {
    let symbol = unsafe {
        // Safety: see above.
        ZBarcode_Create()
    };
    assert!(!symbol.is_null(), "zint could not allocate a symbol");

    // An EAN-13 whose check digit does not match the twelve digits before it.
    let data = c"6975004310002";
    let result = unsafe {
        // Safety: see above.
        (*symbol).symbology = BARCODE_EANX_CHK as i32;
        render_svg_into_memory(symbol);
        ZBarcode_Encode_and_Print(symbol, data.as_ptr() as *const u8, 0, 0)
    };
    assert_eq!(result, ZINT_ERROR_INVALID_CHECK as i32);

    let explanation = unsafe {
        // Safety: zint always leaves `errtxt` NUL terminated.
        CStr::from_ptr((*symbol).errtxt.as_ptr())
    };
    assert!(
        !explanation.to_bytes().is_empty(),
        "zint has more to say about the failure than the return code does"
    );

    unsafe {
        // Safety: see above.
        ZBarcode_Delete(symbol)
    };
}

/// The capability flags are how a caller asks what a symbology supports, so
/// they have to answer differently for symbologies that differ.
#[test]
fn capability_flags_describe_the_symbology_they_are_asked_about() {
    let asked = ZINT_CAP_HRT | ZINT_CAP_ECI | ZINT_CAP_DOTTY;

    let qr = unsafe {
        // Safety: any `int` is accepted; zint range checks the symbology.
        ZBarcode_Cap(BARCODE_QRCODE as i32, asked)
    };
    assert_eq!(
        qr,
        ZINT_CAP_ECI | ZINT_CAP_DOTTY,
        "a QR code carries no human readable text but understands ECI and dots"
    );

    let code128 = unsafe {
        // Safety: see above.
        ZBarcode_Cap(BARCODE_CODE128 as i32, asked)
    };
    assert_eq!(
        code128, ZINT_CAP_HRT,
        "Code 128 prints its text but is neither an ECI nor a dotty symbology"
    );

    let unknown = unsafe {
        // Safety: see above.
        ZBarcode_Cap(0, asked)
    };
    assert_eq!(unknown, 0, "an unknown symbology has no capabilities");
}
