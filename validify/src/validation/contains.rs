use crate::traits::Contains;

/// Validates whether the value contains the needle
///
/// See [Contains] for more details.
pub fn validate_contains<T: Contains<C>, C>(haystack: T, needle: &C, not: bool) -> bool {
    if not {
        !haystack.has_element(needle)
    } else {
        haystack.has_element(needle)
    }
}

#[cfg(test)]
mod contains_tests {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_validate_contains_string() {
        assert!(validate_contains("hey", &"e", false));
    }

    #[test]
    fn test_validate_contains_string_can_fail() {
        assert!(!validate_contains("hey", &"o", false));
    }

    #[test]
    fn test_validate_contains_hashmap_key() {
        let mut map = HashMap::new();
        map.insert("hey", 1);
        assert!(validate_contains(map, &"hey", false));
    }

    #[test]
    fn test_validate_contains_hashmap_key_can_fail() {
        let mut map = HashMap::new();
        map.insert("hey", 1);
        assert!(!validate_contains(map, &"bob", false));
    }

    #[test]
    fn test_validate_contains_cow() {
        let test: Cow<'static, str> = "hey".into();
        assert!(validate_contains(test, &"e", false));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(validate_contains(test, &"e", false));
    }

    #[test]
    fn test_validate_contains_cow_can_fail() {
        let test: Cow<'static, str> = "hey".into();
        assert!(!validate_contains(test, &"o", false));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(!validate_contains(test, &"o", false));
    }

    #[test]
    fn test_validate_does_not_contain_string() {
        assert!(!validate_contains("hey", &"e", true));
    }

    #[test]
    fn test_validate_does_not_contain_string_can_fail() {
        assert!(validate_contains("hey", &"o", true));
    }

    #[test]
    fn test_validate_does_not_contain_hashmap_key() {
        let mut map = HashMap::new();
        map.insert("hey", 1);
        assert!(!validate_contains(map, &"hey", true));
    }

    #[test]
    fn test_validate_does_not_contain_hashmap_key_can_fail() {
        let mut map = HashMap::new();
        map.insert("hey", 1);
        assert!(validate_contains(map, &"bob", true));
    }

    #[test]
    fn test_validate_does_not_contain_cow() {
        let test: Cow<'static, str> = "hey".into();
        assert!(!validate_contains(test, &"e", true));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(!validate_contains(test, &"e", true));
    }

    #[test]
    fn test_validate_does_not_contain_cow_can_fail() {
        let test: Cow<'static, str> = "hey".into();
        assert!(validate_contains(test, &"o", true));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(validate_contains(test, &"o", true));
    }
}

#[cfg(test)]
mod in_notin_tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct A {
        a: usize,
    }

    #[test]
    fn _in() {
        const STRUCTS: [A; 3] = [A { a: 1 }, A { a: 2 }, A { a: 3 }];

        let a = A { a: 2 };
        assert!(validate_contains(STRUCTS, &a, false));

        let a = A { a: 4 };
        assert!(!validate_contains(STRUCTS, &a, false));
    }

    #[test]
    fn not_in() {
        const STRUCTS: [A; 3] = [A { a: 1 }, A { a: 2 }, A { a: 3 }];

        let a = A { a: 4 };
        assert!(validate_contains(STRUCTS, &a, true));

        let a = A { a: 2 };
        assert!(!validate_contains(STRUCTS, &a, true));
    }
}
