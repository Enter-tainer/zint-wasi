use std::{ffi::CString, ptr::slice_from_raw_parts};

use zint_wasm_sys::*;

pub fn main() {
    let encoded_text = CString::new("A12345B").expect("CString::new failed");
    let fg_color = CString::new("000000").expect("CString::new failed");
    let bg_color = CString::new("FFFFFF").expect("CString::new failed");
    let filename = CString::new("res.svg").expect("CString::new failed");
    // Barcode configs
    let symbol = unsafe { ZBarcode_Create().as_mut().unwrap() };
    symbol.symbology = BARCODE_CODE128 as i32;
    symbol.output_options |=
        BARCODE_QUIET_ZONES as i32 | BARCODE_BIND as i32 | BARCODE_MEMORY_FILE as i32;
    symbol.height = 50.0;
    symbol.show_hrt = 1;
    symbol.border_width = 5;
    symbol.scale = 1.0;
    symbol.whitespace_width = 10;

    // Generate the barcode
    unsafe {
        symbol
            .fgcolor
            .copy_from(fg_color.as_ptr(), fg_color.as_bytes().len());
        symbol
            .bgcolor
            .copy_from(bg_color.as_ptr(), bg_color.as_bytes().len());
        symbol
            .outfile
            .as_mut_ptr()
            .copy_from(filename.as_ptr(), filename.as_bytes().len());

        let err_code: i32 =
            ZBarcode_Encode_and_Print(symbol, encoded_text.as_ptr() as *const u8, 0, 0);
        assert_eq!(err_code, 0);
    }
    let res = unsafe {
        println!("memfile_size: {}", symbol.memfile_size);
        let memfile = &*slice_from_raw_parts(symbol.memfile, symbol.memfile_size as usize);
        let svg_str = String::from_utf8_lossy(memfile);
        svg_str
    };
    println!("{}", res);
    // Free memory
    unsafe { ZBarcode_Delete(symbol) }
}
