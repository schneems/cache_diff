use crate::attributes::CacheAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Data::Struct;
use syn::Fields::Named;
use syn::{DataStruct, DeriveInput, Field, FieldsNamed, PathArguments};

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

        let name = attributes
            .rename
            .unwrap_or_else(|| field_name.as_ref().unwrap().to_string().replace("_", " "));

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
                        format!("{name} ({old} to {now})",
                            name = #name,
                            old = self.fmt_value(&#display(&old.#field_name)),
                            now = self.fmt_value(&#display(&self.#field_name))
                        )
                    );
                }
            })
        }
    }

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
