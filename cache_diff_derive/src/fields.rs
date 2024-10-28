use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::Data::Struct;
use syn::Fields::Named;
use syn::{Attribute, DataStruct, FieldsNamed, Ident, PathArguments, Token};
use syn::{DeriveInput, Field};

#[derive(Debug, PartialEq, Eq, Default)]
struct CacheAttributes {
    rename: Option<String>,
    display: Option<syn::Path>,
    ignore: Option<()>,
}

impl CacheAttributes {
    // Parse all attributes inside of `#[cache_diff(...)]` and return a single CacheAttributes value
    fn parse_all(input: &Attribute) -> syn::Result<Self> {
        let mut attribute = CacheAttributes::default();

        match &input.meta {
            syn::Meta::List(meta_list) => {
                for attr in meta_list
                    .parse_args_with(Punctuated::<CacheAttributes, Token![,]>::parse_terminated)?
                {
                    if let Some(value) = attr.rename {
                        attribute.rename = Some(value);
                    }
                    if let Some(display) = attr.display {
                        attribute.display = Some(display);
                    }
                    if let Some(ignore) = attr.ignore {
                        attribute.ignore = Some(ignore);
                    }
                }
                Ok(attribute)
            }
            _ => Err(syn::Error::new(
                input.pound_token.span,
                "Expected a list of attributes",
            )),
        }
    }
}

impl syn::parse::Parse for CacheAttributes {
    // Parse a single attribute inside of a `#[cache_diff(...)]` attribute
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();
        let mut attribute = CacheAttributes::default();
        match name.to_string().as_ref() {
            "rename" => {
                input.parse::<syn::Token![=]>()?;
                let value = input.parse::<syn::LitStr>()?;
                attribute.rename = Some(value.value());
            }
            "display" => {
                input.parse::<syn::Token![=]>()?;
                attribute.display = Some(input.parse()?);
            }
            "ignore" => {
                attribute.ignore = Some(());
            }
            _ => {
                return Err(syn::Error::new(
                    name.span(),
                    format!("Unknown attribute: {}", name_str),
                ))
            }
        }
        Ok(attribute)
    }
}

fn extract_attribute_from_field<'a>(f: &'a Field, name: &'a str) -> Option<&'a syn::Attribute> {
    f.attrs.iter().find(|&attr| attr.path().is_ident(name))
}

fn is_pathbuf(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "PathBuf" && segment.arguments == PathArguments::None;
        }
    }
    false
}

pub fn create_cache_diff(item: TokenStream) -> syn::Result<TokenStream> {
    let ast: DeriveInput = syn::parse2(item).unwrap();
    let name = ast.ident;
    let fields = match ast.data {
        Struct(DataStruct {
            fields: Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => unimplemented!("Only implemented for structs"),
    };
    let mut comparisons = Vec::new();
    for f in fields.iter() {
        let field_name = &f.ident;

        let attributes = extract_attribute_from_field(f, "cache_diff")
            .map(CacheAttributes::parse_all)
            .unwrap_or_else(|| Ok(CacheAttributes::default()))?;

        let mut name = attributes
            .rename
            .unwrap_or_else(|| field_name.as_ref().unwrap().to_string());
        name = name.replace("_", " ");

        let display = attributes.display.unwrap_or_else(|| {
            if is_pathbuf(&f.ty) {
                syn::parse_str("std::path::Path::display")
                    .expect("PathBuf::display parses as a syn::Path")
            } else {
                syn::parse_str("std::convert::identity")
                    .expect("std::convert::identity parses as a syn::Path")
            }
        });

        if attributes.ignore.is_none() {
            comparisons.push(quote! {
                if self.#field_name != old.#field_name {
                    differences.push(
                        format!("{name} (`{old}` to `{now}`)",
                            name = #name,
                            old = #display(&old.#field_name),
                            now = #display(&self.#field_name)
                        )
                    );
                }
            })
        }
    }

    // TODO: Wrap with bullet_stream::style::value
    Ok(quote! {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate cache_diff as _cache_diff;
        impl _cache_diff::CacheDiff for #name {
            fn diff(&self, old: &Self) -> Vec<String> {
                let mut differences = Vec::new();
                #(#comparisons)*
                differences
            }
        }
    })
}

// #[allow(unused_extern_crates, clippy::useless_attribute)]
// extern crate cache_diff::CacheDiff as _cache_diff;

// impl _cache_diff for #name {
//     fn diff(&self, old: &Self) -> Vec<String> {
//         Vec::new()
//     }
// }
