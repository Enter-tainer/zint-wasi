use proc_macro2::Span;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::*;

pub fn bare_type(raw: &str) -> Type {
    Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: Punctuated::from_iter(std::iter::once(PathSegment {
                ident: Ident::new(raw, Span::call_site()),
                arguments: syn::PathArguments::None,
            })),
        },
    })
}

pub fn flatten_errors<I, K>(value: I) -> std::result::Result<Vec<K>, syn::Error>
where
    I: IntoIterator<Item = std::result::Result<K, syn::Error>>,
{
    let mut results = Vec::new();
    let mut errors = None;
    let mut iter = value.into_iter();
    for item in &mut iter {
        match item {
            Ok(it) => results.push(it),
            Err(err) => {
                errors = Some(err);
                results = vec![];
                break;
            }
        }
    }
    if let Some(err) = &mut errors {
        for item in iter {
            if let Err(other) = item {
                err.combine(other);
            }
        }
    }
    if let Some(err) = errors {
        Err(err)
    } else {
        Ok(results)
    }
}

pub fn path_from_ident(ident: Ident) -> syn::Path {
    syn::Path {
        leading_colon: None,
        segments: Punctuated::from_iter([PathSegment {
            ident,
            arguments: syn::PathArguments::None,
        }]),
    }
}

pub fn path_from_str(path: impl AsRef<str>) -> syn::Path {
    let mut parts = path.as_ref().split("::");
    let first = parts.next().expect("provided path is empty");
    let mut segments = Vec::with_capacity(3);
    let leading_colon = if first.is_empty() {
        Some(Default::default())
    } else {
        segments.push(first);
        None
    };
    for current in parts {
        segments.push(current);
    }
    syn::Path {
        leading_colon,
        segments: Punctuated::from_iter(segments.iter().map(|it| PathSegment {
            ident: Ident::new(it, Span::call_site()),
            arguments: syn::PathArguments::None,
        })),
    }
}

pub fn is_path_one_of<I, S>(a: &Path, items: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let base = a.into_token_stream().to_string().replace(" ", "");
    for item in items {
        if base == item.as_ref() {
            return true;
        }
    }
    return false;
}
