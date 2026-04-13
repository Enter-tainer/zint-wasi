extern crate proc_macro;

use crate::{
    data::SymbologyDeclaration,
    generate::{gen_symbol_option_structs, gen_symbol_options_enum, gen_symbology_enum},
};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

mod data;
mod generate;
#[cfg(feature = "_gen_symbology_summary")]
mod summary;
mod util;

#[proc_macro]
pub fn symbol_data(input: TokenStream) -> TokenStream {
    let declaration = parse_macro_input!(input as SymbologyDeclaration);

    let result = gen_symbology_enum(&declaration);
    let mut result = match result {
        Ok(it) => it.into_token_stream(),
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let structs = match gen_symbol_option_structs(&declaration) {
        Ok(it) => it,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let config_enum = match gen_symbol_options_enum(&declaration) {
        Ok(it) => it,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    result.extend([structs, config_enum]);

    #[cfg(feature = "_gen_symbology_summary")]
    result.extend([summary::gen_symbology_summary(&declaration)]);

    result.into()
}
