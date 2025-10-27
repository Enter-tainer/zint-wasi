use syn::{parse::Parse, punctuated::Punctuated, *};

pub enum SymbologyOptionsDeclaration {
    Binding {
        binding: ExprPath,
        alias_name: Option<BindingAlias>,
    },
    Inline {
        name_override: Option<Ident>,
        lifetime: Option<Lifetime>,
        brace_token: token::Brace,
        entries: Punctuated<SymbologyEntryOption, Token![,]>,
    },
}

impl Parse for SymbologyOptionsDeclaration {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let has_name_override = input.peek2(token::Brace);
        if input.peek(token::Brace) || input.peek(Token![<]) || has_name_override {
            let name_override = if has_name_override {
                Some(input.parse()?)
            } else {
                None
            };

            let lifetime = if input.peek(Token![<]) {
                let _: Token![<] = input.parse()?;
                let lifetime = input.parse()?;
                let _: Token![>] = input.parse()?;
                Some(lifetime)
            } else {
                None
            };

            let content;
            let brace_token = braced!(content in input);
            let entries = content.parse_terminated(SymbologyEntryOption::parse, Token![,])?;

            return Ok(Self::Inline {
                name_override,
                lifetime,
                brace_token,
                entries,
            });
        }

        let path: ExprPath = input.parse().map_err(|err| {
            Error::new(
                err.span(),
                "expected either an existing options struct or anonymous inline declaration",
            )
        })?;

        let alias_name = if input.peek(Token![as]) {
            let _: Token![as] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self::Binding {
            binding: path,
            alias_name,
        })
    }
}

pub enum BindingAlias {
    Name(Ident, Option<Lifetime>),
    None(Token![_]),
}

impl Parse for BindingAlias {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        Ok(if input.peek(Token![_]) {
            BindingAlias::None(input.parse()?)
        } else {
            let name = input.parse()?;

            let lifetime = if input.peek(Token![<]) {
                let _: Token![<] = input.parse()?;
                let lifetime = input.parse()?;
                let _: Token![>] = input.parse()?;
                Some(lifetime)
            } else {
                None
            };

            BindingAlias::Name(name, lifetime)
        })
    }
}

pub struct SymbologyEntryOption {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub ty: Type,
    pub default: Option<(Token![=], Expr)>,
}

impl Parse for SymbologyEntryOption {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty = input.parse()?;

        let default = if input.peek(Token![=]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        Ok(Self {
            attrs,
            name,
            ty,
            default,
        })
    }
}

pub struct ApplyOptionClosure {
    pub result_name: Option<Ident>,
    pub options_name: Option<Ident>,
    pub body: Box<Expr>,
}

impl Parse for ApplyOptionClosure {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let _: Token![|] = input.parse()?;

        let result_name = if input.peek(Token![_]) {
            let _: Token![_] = input.parse()?;
            None
        } else {
            Some(input.parse()?)
        };
        let _: Token![,] = input.parse()?;
        let options_name = if input.peek(Token![_]) {
            let _: Token![_] = input.parse()?;
            None
        } else {
            Some(input.parse()?)
        };
        let _: Token![|] = input.parse()?;
        let body = input.parse()?;

        Ok(Self {
            result_name,
            options_name,
            body: Box::new(body),
        })
    }
}
