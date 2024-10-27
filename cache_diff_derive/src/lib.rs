use fields::create_cache_diff;
use proc_macro::TokenStream;

mod fields;

#[proc_macro_derive(CacheDiff, attributes(cache_diff))]
pub fn cache_diff(item: TokenStream) -> TokenStream {
    create_cache_diff(item.into()).into()
}
