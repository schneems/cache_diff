use proc_macro2::TokenStream;
use quote::quote;
use syn::Data::Struct;
use syn::DeriveInput;
use syn::Fields::Named;
use syn::{DataStruct, FieldsNamed};

pub fn create_cache_diff(item: TokenStream) -> TokenStream {
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
        let name = &f.ident;
        quote! {
            if self.#name != old.#name {
                differences.push(format!("#name"))
            }
        }
    });

    quote! {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate cache_diff as _cache_diff;
        impl _cache_diff::CacheDiff for #name {
            fn diff(&self, old: &Self) -> Vec<String> {
                let mut differences = Vec::new();
                #(#comparisons)*
                differences
            }
        }
    }
}

// #[allow(unused_extern_crates, clippy::useless_attribute)]
// extern crate cache_diff::CacheDiff as _cache_diff;

// impl _cache_diff for #name {
//     fn diff(&self, old: &Self) -> Vec<String> {
//         Vec::new()
//     }
// }
