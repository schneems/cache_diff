use cache_diff::CacheDiff;

#[derive(CacheDiff)]
struct Hello {}
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_diff() {
        #[derive(CacheDiff)]
        struct Person {
            name: String,
        }

        let diff = Person {
            name: "richard".to_string(),
        }
        .diff(&Person {
            name: "rich".to_string(),
        });

        assert_eq!(diff.len(), 1);
    }
}
