use cache_diff::CacheDiff;

#[derive(CacheDiff)]
struct Hello {}
fn main() {
    let _ = Hello {};
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_diff<T: cache_diff::CacheDiff>(_in: &T) {}

    #[test]
    fn test_cache_diff() {
        #[derive(CacheDiff)]
        struct Person {
            _name: String,
        }
        let richard = Person {
            _name: "richard".to_string(),
        };
        is_diff(&richard);
        let diff = richard.diff(&Person {
            _name: "rich".to_string(),
        });

        assert_eq!(diff.len(), 1);
    }
}
