/// Given another cache object, returns a list of differences between the two
///
/// If no differences, return an empty list
pub trait CacheDiff {
    fn diff(&self, old: &Self) -> Vec<String>;
}
pub use cache_diff_derive::CacheDiff;
