use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn create_cache_diff(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse2(item).unwrap();
    let name = ast.ident;

    quote! {
        #[allow(unused_extern_crates, clippy::useless_attribute)]
        extern crate cache_diff as _cache_diff;
        impl _cache_diff::CacheDiff for #name {
            fn diff(&self, old: &Self) -> Vec<String> {
                Vec::new()
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
