use indexmap::{IndexMap, IndexSet};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;

/// Used by the [validate_length][crate::validation::length::validate_length]
/// function to validate the size of `T`.
///
/// The trait is implemented for common container types and can be implemented
/// on anything that needs to be validated by `length`.
///
/// Note: string implementations count the characters, not the bytes.
pub trait Length {
    fn length(&self) -> usize;
}

impl<T> Length for &T
where
    T: Length,
{
    fn length(&self) -> usize {
        <T as Length>::length(*self)
    }
}

impl Length for String {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl Length for &str {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl Length for Cow<'_, str> {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl<T> Length for Vec<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> Length for &[T] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, const N: usize> Length for [T; N] {
    fn length(&self) -> usize {
        N
    }
}

impl<K, V, S> Length for HashMap<K, V, S> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T, S> Length for HashSet<T, S> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<K, V> Length for BTreeMap<K, V> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> Length for BTreeSet<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<K, V> Length for IndexMap<K, V> {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<T> Length for IndexSet<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

/// Used by the [validate_contains][crate::validation::contains::validate_contains] function.
///
/// In `contains`, the field is checked if it contains the provided value.
///
/// In `in/not_in`, the given value is checked if it contains the field.
pub trait Contains<T> {
    fn has_element(&self, needle: &T) -> bool;
}

impl<T, C> Contains<C> for &T
where
    T: Contains<C>,
{
    fn has_element(&self, needle: &C) -> bool {
        <T as Contains<C>>::has_element(self, needle)
    }
}

impl<T> Contains<T> for Vec<T>
where
    T: PartialEq,
{
    fn has_element(&self, needle: &T) -> bool {
        self.contains(needle)
    }
}

impl<T> Contains<&T> for Vec<T>
where
    T: PartialEq,
{
    fn has_element(&self, needle: &&T) -> bool {
        self.contains(*needle)
    }
}

impl Contains<&str> for Vec<String> {
    fn has_element(&self, needle: &&str) -> bool {
        self.contains(&needle.to_string())
    }
}

impl<T> Contains<T> for &[T]
where
    T: PartialEq,
{
    fn has_element(&self, needle: &T) -> bool {
        self.contains(needle)
    }
}

impl<T, const N: usize> Contains<T> for [T; N]
where
    T: PartialEq,
{
    fn has_element(&self, needle: &T) -> bool {
        self.contains(needle)
    }
}

impl<K, V> Contains<K> for HashMap<K, V>
where
    K: PartialEq + Eq + Hash,
{
    fn has_element(&self, needle: &K) -> bool {
        self.contains_key(needle)
    }
}

impl<V> Contains<&str> for HashMap<String, V> {
    fn has_element(&self, needle: &&str) -> bool {
        self.contains_key(&needle.to_string())
    }
}

impl<K, V> Contains<&K> for HashMap<K, V>
where
    K: PartialEq + Eq + Hash,
{
    fn has_element(&self, needle: &&K) -> bool {
        self.contains_key(*needle)
    }
}

impl Contains<&str> for String {
    fn has_element(&self, needle: &&str) -> bool {
        self.contains(needle)
    }
}

impl Contains<String> for String {
    fn has_element(&self, needle: &String) -> bool {
        self.contains(needle)
    }
}

impl Contains<String> for &str {
    fn has_element(&self, needle: &String) -> bool {
        self.contains(needle)
    }
}

impl Contains<&str> for &str {
    fn has_element(&self, needle: &&str) -> bool {
        self.contains(needle)
    }
}

impl Contains<&String> for &[&str] {
    fn has_element(&self, needle: &&String) -> bool {
        self.contains(&needle.as_str())
    }
}

impl<const N: usize> Contains<&String> for [&str; N] {
    fn has_element(&self, needle: &&String) -> bool {
        self.contains(&needle.as_str())
    }
}

impl Contains<&str> for Cow<'_, str> {
    fn has_element(&self, needle: &&str) -> bool {
        self.contains(needle)
    }
}
