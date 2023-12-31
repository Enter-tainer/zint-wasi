use std::ffi::{CStr, CString};

use libc::strlen;
use zint_wasm_sys::*;

pub fn main() {
    let encoded_text = CString::new("A12345B").expect("CString::new failed");
    let fg_color = CString::new("000000").expect("CString::new failed");
    let bg_color = CString::new("FFFFFF").expect("CString::new failed");

    // Barcode configs
    let symbol = unsafe { ZBarcode_Create().as_mut().unwrap() };
    symbol.symbology = BARCODE_CODE128 as i32;
    symbol.output_options |= BARCODE_QUIET_ZONES as i32 | BARCODE_BIND as i32;
    symbol.height = 50.0;
    symbol.show_hrt = 1;
    symbol.border_width = 5;
    symbol.scale = 1.0;
    symbol.whitespace_width = 10;

    // Generate the barcode
    unsafe {
        symbol
            .fgcolor
            .copy_from(fg_color.as_ptr(), strlen(fg_color.as_ptr()));
        symbol
            .bgcolor
            .copy_from(bg_color.as_ptr(), strlen(bg_color.as_ptr()));

        ZBarcode_Encode_and_Buffer_Vector(symbol, encoded_text.as_ptr() as *const u8, 0, 0);
    }
    let mut err_code: i32 = 0;
    let res = unsafe {
        let err_code_ptr = &mut err_code as *mut i32;
        let svg_cstr = svg_plot_string(symbol, err_code_ptr);
        let svg_str = CStr::from_ptr(svg_cstr).to_string_lossy().into_owned();
        free_svg_plot_string(svg_cstr);
        svg_str
    };
    assert_eq!(err_code, 0);
    println!("{}", res);
    // Free memory
    unsafe { ZBarcode_Delete(symbol) }
}
