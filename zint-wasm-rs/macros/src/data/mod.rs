use crate::util::*;
use syn::*;
use syn::{parse::Parse, token::PathSep};
use syn::{punctuated::Punctuated, spanned::Spanned};

pub mod options;
pub use options::*;

pub struct SymbologyDeclaration {
    pub attrs: Vec<Attribute>,
    pub vis_pub: Token![pub],
    pub enum_token: Token![enum],
    pub ident: Ident,
    // No generics
    pub brace_token: token::Brace,
    pub variants: Punctuated<SymbologyVariant, Token![,]>,
}

impl Parse for SymbologyDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let vis_pub = input.parse()?;
        let enum_token = input.parse()?;
        let ident = input.parse()?;

        let content;
        let brace_token = braced!(content in input);
        let variants = content.parse_terminated(SymbologyVariant::parse, Token![,])?;

        Ok(Self {
            attrs,
            vis_pub,
            enum_token,
            ident,
            brace_token,
            variants,
        })
    }
}

pub struct SymbologyVariant {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub eq_token: Token![=],
    pub brace_token: token::Brace,
    pub entries: Punctuated<SymbologyEntry, Token![,]>,
}

impl Parse for SymbologyVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let symbol_name = input.parse()?;
        let eq_token = input.parse()?;

        let content;
        let brace_token = braced!(content in input);
        let entries = content.parse_terminated(SymbologyEntry::parse, Token![,])?;

        let result = Self {
            attrs,
            name: symbol_name,
            eq_token,
            brace_token,
            entries,
        };
        result.validate()?;
        Ok(result)
    }
}

impl SymbologyVariant {
    pub fn get_entry(&self, name: &'static str) -> Option<&SymbologyEntry> {
        self.entries.iter().find(|it| it.name() == name)
    }
    pub fn require_entry(&self, name: &'static str) -> Result<&SymbologyEntry> {
        self.get_entry(name).ok_or(Error::new(
            self.name.span(),
            format!("symbology is missing `{name}` entry"),
        ))
    }

    pub fn validate(&self) -> Result<()> {
        let _ = self.require_entry(SymbologyEntry::RAW_NAME)?;

        let options = match self.get_entry(SymbologyEntry::OPTIONS_NAME) {
            None => None,
            Some(SymbologyEntry::Options { name, value, .. }) => Some((name, value)),
            _ => unreachable!(),
        };
        let apply_option = self.get_entry(SymbologyEntry::APPLY_OPTION_NAME);

        let extra_apply = |name: &Ident| {
            Err(Error::new(
                name.span(),
                format!(
                    "`{1}` callback can only be specified for inline `{0}`",
                    SymbologyEntry::OPTIONS_NAME,
                    SymbologyEntry::APPLY_OPTION_NAME,
                ),
            ))
        };
        let missing_apply = |name: &Ident| {
            Err(Error::new(
                name.span(),
                format!(
                    "inline `{}` declaration requires `{}` callback to be specified",
                    SymbologyEntry::OPTIONS_NAME,
                    SymbologyEntry::APPLY_OPTION_NAME,
                ),
            ))
        };
        match options {
            None | Some((_, SymbologyOptionsDeclaration::Binding { .. }))
                if apply_option.is_some() =>
            {
                let apply_option = apply_option.unwrap();
                return extra_apply(apply_option.name());
            }
            Some((options_name, SymbologyOptionsDeclaration::Inline { .. }))
                if apply_option.is_none() =>
            {
                return missing_apply(options_name);
            }
            _ => {}
        }

        Ok(())
    }

    pub fn discriminant(&self) -> Expr {
        Expr::Cast(ExprCast {
            attrs: vec![],
            expr: Box::new(Expr::Path(self.raw().expect("missing raw entry").clone())),
            as_token: token::As::default(),
            ty: Box::new(bare_type("i32")),
        })
    }

    pub fn raw(&self) -> Option<&ExprPath> {
        let raw = self.get_entry(SymbologyEntry::RAW_NAME)?;
        match raw {
            SymbologyEntry::Raw { value, .. } => Some(value),
            _ => unreachable!(),
        }
    }

    pub fn options(&self) -> Option<&SymbologyOptionsDeclaration> {
        let raw = self.get_entry(SymbologyEntry::OPTIONS_NAME)?;
        match raw {
            SymbologyEntry::Options { value, .. } => Some(value),
            _ => unreachable!(),
        }
    }

    pub fn apply_fn(&self) -> Option<&ApplyOptionClosure> {
        let raw = self.get_entry(SymbologyEntry::APPLY_OPTION_NAME)?;
        match raw {
            SymbologyEntry::ApplyOption { closure, .. } => Some(closure),
            _ => unreachable!(),
        }
    }

    pub fn aliases(&self) -> Option<&[LitStr]> {
        let raw = self.get_entry(SymbologyEntry::ALIAS_NAME)?;
        match raw {
            SymbologyEntry::Aliases { aliases, .. } => Some(aliases),
            _ => unreachable!(),
        }
    }

    pub fn options_struct_name(&self) -> Option<TypePath> {
        Some(match self.options()? {
            crate::data::SymbologyOptionsDeclaration::Binding {
                binding,
                alias_name: Some(alias_name),
            } => match alias_name {
                BindingAlias::Name(ident, lifetime) => TypePath {
                    qself: None,
                    path: match lifetime {
                        Some(lifetime) => syn::Path {
                            leading_colon: None,
                            segments: Punctuated::from_iter([PathSegment {
                                ident: ident.clone(),
                                arguments: syn::PathArguments::AngleBracketed(
                                    AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: Default::default(),
                                        args: Punctuated::from_iter([GenericArgument::Lifetime(
                                            lifetime.clone(),
                                        )]),
                                        gt_token: Default::default(),
                                    },
                                ),
                            }]),
                        },
                        None => path_from_ident(ident.clone()),
                    },
                },
                BindingAlias::None(_) => TypePath {
                    qself: binding.qself.clone(),
                    path: binding.path.clone(),
                },
            },
            crate::data::SymbologyOptionsDeclaration::Inline {
                name_override,
                lifetime,
                ..
            } => TypePath {
                qself: None,
                path: {
                    let mut path = match name_override {
                        Some(ident) => path_from_ident(ident.clone()),
                        None => path_from_str(format!("{}Options", self.name)),
                    };
                    if let Some(lifetime) = lifetime {
                        let last_segment = path
                            .segments
                            .last_mut()
                            .expect("inline options struct name can't be empty");
                        last_segment.arguments =
                            PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: Default::default(),
                                args: Punctuated::from_iter([GenericArgument::Lifetime(
                                    lifetime.clone(),
                                )]),
                                gt_token: Default::default(),
                            })
                    }
                    path
                },
            },
            _ => TypePath {
                qself: None,
                path: path_from_str(format!("{}Options", self.name)),
            },
        })
    }
}

pub enum SymbologyEntry {
    Raw {
        name: Ident,
        value: ExprPath,
    },
    Aliases {
        name: Ident,
        aliases: Vec<LitStr>,
    },
    Options {
        attrs: Vec<Attribute>,
        name: Ident,
        value: SymbologyOptionsDeclaration,
    },
    ApplyOption {
        name: Ident,
        closure: ApplyOptionClosure,
    },
}
impl SymbologyEntry {
    pub const RAW_NAME: &'static str = "raw";
    pub const OPTIONS_NAME: &'static str = "options";
    pub const ALIAS_NAME: &'static str = "alias";
    pub const APPLY_OPTION_NAME: &'static str = "apply_options";

    pub fn name(&self) -> &Ident {
        match self {
            SymbologyEntry::Raw { name, .. }
            | SymbologyEntry::Options { name, .. }
            | SymbologyEntry::Aliases { name, .. }
            | SymbologyEntry::ApplyOption { name, .. } => name,
        }
    }
}

impl Parse for SymbologyEntry {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let name: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;

        let name_str = name.to_string();
        Ok(match name_str.as_str() {
            SymbologyEntry::RAW_NAME if attrs.is_empty() => Self::Raw {
                name,
                value: input.parse()?,
            },
            SymbologyEntry::OPTIONS_NAME => Self::Options {
                attrs,
                name,
                value: input.parse()?,
            },
            SymbologyEntry::ALIAS_NAME => {
                let aliases = if input.peek(token::Bracket) {
                    let content;
                    let _ = bracketed!(content in input);
                    content.parse_terminated(Lit::parse, Token![,])?
                } else {
                    let single: Lit = input.parse()?;
                    Punctuated::from_iter([single])
                };

                let aliases = aliases.into_iter().map(|it| match it {
                    Lit::Str(lit_str) => Ok(lit_str),
                    other => Err(Error::new(other.span(), "only string literals allowed")),
                });
                let aliases = flatten_errors(aliases)?;

                SymbologyEntry::Aliases { name, aliases }
            }
            SymbologyEntry::APPLY_OPTION_NAME if attrs.is_empty() => Self::ApplyOption {
                name,
                closure: input.parse()?,
            },
            _ if !attrs.is_empty() => {
                let span = attrs.first().unwrap().span();
                return Err(Error::new(
                    span,
                    format!(
                        "rustdoc is only allowed on `{}` entry",
                        SymbologyEntry::OPTIONS_NAME
                    ),
                ));
            }
            unknown => {
                return Err(Error::new(
                    name.span(),
                    format!("`{unknown}` is not a supported symbology entry"),
                ));
            }
        })
    }
}
