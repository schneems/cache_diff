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
        let field_name = &f.ident;
        let mut display = field_name.as_ref().unwrap().to_string();
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
