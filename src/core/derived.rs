//! Holds derived attributes

#[macro_export]
/// Creates a hashmap from vector key => value pairs
macro_rules! hashmap {
    ($($key: expr => $val: expr), *) =>{{
    let mut map = std::collections::HashMap::new();
    $(map.insert($key, $val);)*
        map
}}}
