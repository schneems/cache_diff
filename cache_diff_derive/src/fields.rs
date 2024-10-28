use proc_macro2::TokenStream;
use quote::quote;
use syn::Data::Struct;
use syn::Fields::Named;
use syn::{DataStruct, Expr, ExprLit, FieldsNamed, Lit, MetaNameValue};
use syn::{DeriveInput, Field};

fn extract_attribute_from_field<'a>(f: &'a Field, name: &'a str) -> Option<&'a syn::Attribute> {
    f.attrs.iter().find(|&attr| attr.path().is_ident(name))
}

fn match_expr_as_lit_str(expr: &Expr) -> Option<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        Some(lit_str.value())
    } else {
        None
    }
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
    let comparisons = fields.iter().map(|f| {
        let field_name = &f.ident;
        let rename = extract_attribute_from_field(f, "cache_diff")
            .map(|a| &a.meta)
            .and_then(|m| match m {
                syn::Meta::Path(_) => panic!("Not supported, TODO better message"),
                syn::Meta::List(meta_list) => {
                    if let Ok(name_value) = meta_list.parse_args::<MetaNameValue>() {
                        if name_value.path.is_ident("rename") {
                            if let Some(value) = match_expr_as_lit_str(&name_value.value) {
                                Some(value)
                            } else {
                                panic!("Expected a string literal")
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                syn::Meta::NameValue(_) => panic!("Not supported, TODO better message"),
            });

        let mut display = rename.unwrap_or_else(|| field_name.as_ref().unwrap().to_string());
        display = display.replace("_", " ");

        // TODO: Wrap with bullet_stream::style::value
        // TODO: Rename attribute `cache_diff(rename = "Ruby version" )`
        // TODO: Ignore attribute `cache_diff(ignore)`
        // TODO: Handle attributes that don't directly `impl Display``
        //       like PathBuf. We could special case the most common
        //       or do something like thiserr but the DSL would be odd
        //       Maybe something like:
        //
        //         `cache_diff(display = PathBuff::display)`
        quote! {
            if self.#field_name != old.#field_name {
                differences.push(#display.to_string())
            }
        }
    });

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
