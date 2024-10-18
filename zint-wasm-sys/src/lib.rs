#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(all(test, any(unix, target_os = "windows")))]
mod tests {
    use super::*;
    use libc::strlen;
    use std::ffi::{CStr, CString};

    // Tested with Zint v2.12.0
    #[test]
    fn hello_code_128() {
        let encoded_text = CString::new("A12345B").expect("CString::new failed");
        let fg_color = CString::new("001100").expect("CString::new failed");
        let bg_color = CString::new("a0b93d").expect("CString::new failed");

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
            libc::free(svg_cstr as *mut libc::c_void);
            svg_str
        };
        assert_eq!(err_code, 0);
        // Free memory
        unsafe { ZBarcode_Delete(symbol) }
    }
}
