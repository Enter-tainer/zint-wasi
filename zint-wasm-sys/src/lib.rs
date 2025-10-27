#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "internals")]
pub mod internal {
    use super::*;

    extern "C" {
        #[cfg(feature = "ps")]
        pub fn zint_ps_plot(symbol: *mut zint_symbol, rotate_angle: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
        #[cfg(feature = "emf")]
        pub fn zint_emf_plot(symbol: *mut zint_symbol, rotate_angle: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
        #[cfg(feature = "svg")]
        pub fn zint_svg_plot(symbol: *mut zint_symbol, rotate_angle: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
    }
}
