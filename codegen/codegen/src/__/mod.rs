#![allow(unused, non_camel_case_types)]
pub enum __ {}
pub enum predicate {}
pub enum identifier {}
pub enum etc {}

/// Manually override the translation of a particular JSON Schema.
///
/// ### Syntax
/// - `predicate`: a [path], [schema], [all], [any], [not]
pub fn overrides(p: predicate) -> __ {
    unreachable!()
}

pub fn rename(p: predicate, new_name: identifier) -> __ {
    unreachable!()
}

pub mod predicates {
    use super::{__, predicate, etc};

    /// All the predicates must be true.
    pub fn all(p1: predicate, p2: predicate, etc: etc) -> predicate {
        unreachable!()
    }

    /// At least one of the predicates must be true.
    pub fn any(p1: predicate, p2: predicate, etc: etc) -> predicate {
        unreachable!()
    }

    /// This predicate must be false.
    pub fn not(p: predicate) -> predicate {
        unreachable!()
    }

    pub struct path;

    pub struct schema;
}
