use crate::data::*;
use crate::util::path_from_str;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::*;

pub fn gen_symbology_enum(declaration: &SymbologyDeclaration) -> Result<ItemEnum> {
    let variants = declaration.variants.iter().map(|it| Variant {
        attrs: it.attrs.clone(),
        ident: it.name.clone(),
        fields: syn::Fields::Unit,
        discriminant: Some((it.eq_token, it.discriminant())),
    });

    Ok(ItemEnum {
        attrs: declaration.attrs.clone(),
        vis: syn::Visibility::Public(declaration.vis_pub),
        enum_token: declaration.enum_token,
        ident: declaration.ident.clone(),
        generics: Generics::default(),
        brace_token: declaration.brace_token,
        variants: Punctuated::from_iter(variants),
    })
}

pub fn gen_symbol_option_structs(declaration: &SymbologyDeclaration) -> Result<TokenStream> {
    let mut result = TokenStream::new();

    let default_fn_call_expr = Expr::Call(ExprCall {
        attrs: vec![],
        func: Box::new(Expr::Path(ExprPath {
            attrs: vec![],
            qself: None,
            path: path_from_str("Default::default"),
        })),
        paren_token: Default::default(),
        args: Punctuated::new(),
    });

    let build_option_struct = |options_struct: Ident,
                               lifetime: &Option<Lifetime>,
                               attributes: &[Attribute],
                               fields: &Punctuated<SymbologyEntryOption, token::Comma>,
                               apply_fn: &ApplyOptionClosure|
     -> TokenStream {
        let mut field_list: Vec<Field> = Vec::with_capacity(fields.len());
        let mut initializer_list: Vec<FieldValue> = Vec::with_capacity(fields.len());

        for field in fields {
            field_list.push(Field {
                attrs: field.attrs.clone(),
                vis: Visibility::Public(Default::default()),
                mutability: FieldMutability::None,
                ident: Some(field.name.clone()),
                colon_token: Some(Default::default()),
                ty: field.ty.clone(),
            });

            let default_value = match &field.default {
                Some((_, default)) => default,
                None => &default_fn_call_expr,
            };

            initializer_list.push(FieldValue {
                attrs: vec![],
                member: Member::Named(field.name.clone()),
                colon_token: Some(Default::default()),
                expr: default_value.clone(),
            })
        }

        let result_name = match &apply_fn.result_name {
            Some(ident) => Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: ident.clone(),
                subpat: None,
            }),
            None => Pat::Wild(PatWild {
                attrs: vec![],
                underscore_token: Default::default(),
            }),
        };
        let options_name = match &apply_fn.options_name {
            Some(ident) => Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: ident.clone(),
                subpat: None,
            }),
            None => Pat::Wild(PatWild {
                attrs: vec![],
                underscore_token: Default::default(),
            }),
        };
        let handler_impl = apply_fn.body.as_ref();

        let struct_generics = lifetime.as_ref().map(|it| Generics {
            lt_token: Default::default(),
            params: Punctuated::from_iter([GenericParam::Lifetime(LifetimeParam::new(it.clone()))]),
            gt_token: Default::default(),
            where_clause: None,
        });

        let handler_generics = match lifetime {
            Some(_) => struct_generics.clone().unwrap(),
            None => Generics {
                lt_token: Default::default(),
                params: Punctuated::from_iter([GenericParam::Lifetime(LifetimeParam::new(
                    Lifetime::new("'o", Span::call_site()),
                ))]),
                gt_token: Default::default(),
                where_clause: None,
            },
        };

        quote! {
            #[derive(Debug, Clone)]
            #(#attributes)*
            pub struct #options_struct #struct_generics {
                #(#field_list),*
            }

            impl #struct_generics Default for #options_struct #struct_generics {
                fn default() -> Self {
                    Self {
                        #(#initializer_list),*
                    }
                }
            }

            impl #handler_generics ConfigureSymbolOptions #handler_generics for #options_struct #struct_generics {
                fn configure(&self, symbol: &mut GenericOptions<'o>) -> Result<(), SymbolOptionError> {
                    #[inline(always)]
                    fn handler #handler_generics (#result_name: &mut GenericOptions #handler_generics, #options_name: &#options_struct #struct_generics) -> Result<(), SymbolOptionError> {
                        #handler_impl
                    }
                    handler(symbol, self)
                }
            }
        }
    };

    for it in &declaration.variants {
        match it.get_entry(SymbologyEntry::OPTIONS_NAME) {
            // only generate structs for options that are declared inline
            Some(SymbologyEntry::Options {
                attrs,
                value:
                    SymbologyOptionsDeclaration::Inline {
                        name_override,
                        lifetime,
                        entries,
                        ..
                    },
                ..
            }) => {
                let apply_fn = it.apply_fn().expect(
                    "gen_symbol_option_structs: validation didn't handle apply_options requirement",
                );

                let name = match name_override {
                    Some(it) => it.clone(),
                    None => {
                        let name = format!("{}Options", it.name);
                        Ident::new(&name, Span::call_site())
                    }
                };
                result.extend(build_option_struct(
                    name, lifetime, attrs, entries, apply_fn,
                ));
            }
            // bindings should simply produce an alias
            Some(SymbologyEntry::Options {
                attrs,
                value:
                    SymbologyOptionsDeclaration::Binding {
                        binding,
                        alias_name,
                    },
                ..
            }) => {
                let (alias, lifetime) = match alias_name {
                    Some(BindingAlias::Name(it, lifetime)) => (it.clone(), lifetime.clone()),
                    Some(BindingAlias::None(_)) => {
                        // alias explicitly erased, skip declaring it
                        continue;
                    }
                    None => {
                        let alias = format!("{}Options", it.name);
                        (Ident::new(&alias, Span::call_site()), None)
                    }
                };
                if alias == binding.to_token_stream().to_string() {
                    // Struct was independently declared with the same name we'd
                    // use in alias.
                    continue;
                }

                let alias = Path {
                    leading_colon: None,
                    segments: Punctuated::from_iter([PathSegment {
                        ident: alias,
                        arguments: match lifetime {
                            None => PathArguments::None,
                            Some(it) => {
                                PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Default::default(),
                                    args: Punctuated::from_iter([GenericArgument::Lifetime(it)]),
                                    gt_token: Default::default(),
                                })
                            }
                        },
                    }]),
                };

                result.extend(quote! {
                    #(#attrs)*
                    pub type #alias = #binding;
                });
            }
            _ => {}
        }
    }

    Ok(result)
}

pub fn gen_symbol_options_enum(declaration: &SymbologyDeclaration) -> Result<TokenStream> {
    let mut variants: Vec<Variant> = Vec::with_capacity(declaration.variants.len());
    let mut match_arms: Vec<Arm> = Vec::with_capacity(declaration.variants.len());

    fn gen_symbol_option_variant(decl: &SymbologyVariant) -> Variant {
        let mut fields = Fields::Unit;

        let options = decl.options_struct_name();
        if let Some(type_path) = options {
            fields = Fields::Unnamed(FieldsUnnamed {
                paren_token: Default::default(),
                unnamed: Punctuated::from_iter([Field {
                    attrs: vec![],
                    vis: Visibility::Inherited,
                    mutability: FieldMutability::None,
                    ident: None,
                    colon_token: None,
                    ty: Type::Path(type_path),
                }]),
            });
        }

        Variant {
            attrs: vec![],
            ident: decl.name.clone(),
            fields,
            discriminant: Some((Default::default(), decl.discriminant())),
        }
    }

    fn gen_symbol_config(decl: &SymbologyVariant) -> Arm {
        let path = format!("SymbolOptions::{}", decl.name);
        let pat = Pat::TupleStruct(PatTupleStruct {
            attrs: vec![],
            qself: None,
            path: path_from_str(path),
            paren_token: Default::default(),
            elems: Punctuated::from_iter([Pat::Ident(PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None,
                ident: Ident::new("options", Span::call_site()),
                subpat: None,
            })]),
        });
        Arm {
            attrs: vec![],
            pat,
            guard: None,
            fat_arrow_token: Default::default(),
            body: Box::new(parse_quote!(options.configure(&mut result)?)),
            comma: None,
        }
    }

    let mut symbology_arms: Vec<Arm> = Vec::with_capacity(declaration.variants.len());

    for it in &declaration.variants {
        variants.push(gen_symbol_option_variant(it));
        if it.options().is_some() {
            match_arms.push(gen_symbol_config(it));
        }

        // Generate a match arm: SymbolOptions::Variant { .. } => Symbology::Variant
        let variant_name = &it.name;
        let has_fields = it.options_struct_name().is_some();
        let pat: Pat = if has_fields {
            parse_quote!(SymbolOptions::#variant_name(..))
        } else {
            parse_quote!(SymbolOptions::#variant_name)
        };
        symbology_arms.push(Arm {
            attrs: vec![],
            pat,
            guard: None,
            fat_arrow_token: Default::default(),
            body: Box::new(parse_quote!(Symbology::#variant_name)),
            comma: Some(Default::default()),
        });
    }

    Ok(quote::quote! {
        #[derive(Debug, Clone)]
        #[repr(i32)]
        pub enum SymbolOptions<'o> {
            #(#variants),*
        }

        impl<'o> TryFrom<SymbolOptions<'o>> for GenericOptions<'o> {
            type Error = SymbolOptionError;

            fn try_from(value: SymbolOptions<'o>) -> Result<Self, SymbolOptionError> {
                let symbology = match &value {
                    #(#symbology_arms)*
                };
                let mut result = GenericOptions::from_symbology(symbology);
                match value {
                    #(#match_arms),*,
                    _ => {}
                };
                Ok(result)
            }
        }
    })
}
