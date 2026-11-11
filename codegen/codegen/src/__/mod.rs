//! This module provides IDE autocomplete/documentation for [taxman_macros]
//! using a similar technique to
//! [`bon`](https://github.com/elastio/bon/blob/c6b012790f041663e4d6d0e5a8a1bc811b23dd08/bon-macros/src/util/ide.rs).
#![allow(unused, non_camel_case_types)]

/// Used for internal uses only...
#[doc(hidden)]
pub enum __ {}

/// ### `at` selector
/// Lets you locate the [Schema](openapi::Schema) or [Operation](openapi::Operation) you wish to patch.
///
/// ### Syntax
/// `at <selector>` where `<selector>` is either:
/// #### Path Item paths
/// A [PathItem](openapi::PathItem) url (`["/order/{id}"]`) &mdash; refers to PathItem with the key of the provided path in [OpenAPI::paths](openapi::OpenApi::paths).
/// #### Schema and Operation IDs
/// - an ident (`ExampleId`) &mdash; refers to an id within the appropriate namespace, meaning:
///   - for [`schema`]: a valid key for schema within [Components::schemas](openapi::Components::schemas), i.e. `schema Example { ... }` refers to `components.schemas.Example` (JSON path).
///   - for [`operation`]: a matching [Operation::operation_id](openapi::Operation::operation_id) across all operations for this API, i.e. `operation Example` refers to the operation with `operation_id == "Example"`.
/// #### Property, Key/Index accesses
/// - *Property* access: use dot-syntax to get property of the correct type, i.e. `["/order/{id}"].get`, `Example.not`.
/// - *Key/Index* accesses: use `(key)` syntax to get the value at that key or index, i.e. `["/order/{id}"].get.responses(200)`, `Example.anyOf(1)`.
#[doc(hidden)]
pub fn at(_: __) -> __ {
    unreachable!()
}

/// ### `schema` patches
/// Modifies the type generation for a given schema.
///
/// ### Examples
/// ```ignore
/// taxman::patches! {
///     schema at ["/order/{id}"].get.responses(200) as OrderOk {
///         "awkward-name" as order_id, // Rename this field.
///         "very-complex_name" as ?,   // Check what this field is auto-renamed as.
///         "another-field": String,    // Ascribe the type of a field.
///         mystery_type: ?,            // Check what type is auto-ascribed for this field.
///     }
/// }
/// ```
///
/// ### Signature
/// ```ignore
/// schema <selector> [as <renamedIdent>] { /* ... */ }
/// ```
///
/// #### Selectors
/// You may either directly use:
/// - an ident by itself &mdash; `schema Example`, or `schema "Example"`.
/// - [`at`] &mdash; `schema at Example`, `schema at "Example"`, `schema at ["/order/{id}"]`.
///
/// #### Renaming the type
/// You may change the generated Rust type name for this schema by appending `as NewName` to the end of the `schema` signature, i.e. `schema "bad-name" as NewName`.
///
/// Alternatively, to see what this type is auto-renamed to, you may use `as ?`.
///
/// ### Fields
///
/// `<fieldName> [as <newName>] [: <newType>]`
///
/// Including a field will override the type generator for that field and use your provided name, and/or ascribed type.
///
/// Similarly to most idents, the field name can other be a Rust ident `ident`, or in quotes for non-Rust compliant names (`non-compliant-field`).
///
/// #### Renaming a field
///
/// Use the `as new_name` syntax. **NOTE**: this **MUST** be a valid Rust identifier (or `?`). It is recommended that you follow the
/// [Rust API Naming Guidelines](https://rust-lang.github.io/api-guidelines/naming.html) (`snake_case` for fields).
///
/// Example: `oldField as old_field`.
///
/// You may use a [`?`](question) in place of an identifier to preview what the field will be auto-renamed to.
///
/// #### Re-ascribing a type
///
/// If you use `field: Type` syntax, the type for that field will be set to your provided type (`Type` in the example).
///
/// You may use [`_`](OriginalType) as a placeholder for the original auto-ascribed type for a field, for example:
/// - `field: _?` does the same thing as `field: ?`
/// - `field: Option<_>` wraps the auto-ascribed type in an [Option].
///
/// ### Macros
///
/// You are not allowed to use any other attribute macros that are not listed below.
///
/// You may use the [delete](macros::delete), [prepend](macros::prepend), [replace](macros::replace), [append](macros::append)
/// helper macros on the schema type itself, or for a particular field.
///
/// See [macros] for more information.
///
#[doc(hidden)]
pub fn schema(_: __) -> __ {
    unreachable!()
}

// TODO: Add docs.
#[doc(hidden)]
pub fn operation(_: __) -> __ {
    unreachable!()
}

/// A question mark `?` can be used to reveal the automatically generated names and types
/// for schemas.
///
/// ## Example: Renames
/// You may use `?` as a replacement for renames, which makes the codegen
/// print the name of the generated type or field name:
/// ```ignore
/// // in example/lib.rs
/// patches! {
///     schema "test-schema" as ? { // [example/lib.rs:3:30] NAME - SCHEMA "test-schema" => TestSchema
///         "original-field" as ?,  // [example/lib.rs:4:30] NAME - FIELD "test-schema"."original-field" => original_field
///     }
/// }
/// ```
/// ## Example: Types
/// You may also use `?` as a replacement for a type of a field, or postfix after a type, or schema.
/// In either case, the codegen will print out the full type.
/// ```ignore
/// // in example/lib.rs
/// patches! {
///     schema "test-schema" {
///         "original-field": ?, // [example/lib.rs:4:29] TYPE - FIELD "test-schema"."original-field": String
///         "field2": Arc<[_]>?, // [example/lib.rs:5:29] TYPE - FIELD "test-schema"."field2": Arc<[String]>
///     }
/// }
/// ```
pub mod question {}

/// Macro verbs allow you to modify the macros that are auto-generated for a particular type and its fields.
///
/// **TIPS** for documentation-writers:
/// - To add/replace rustdoc using the helper macros, use `doc = "My documentation!",`.
/// - In most cases when appending or prepending documentation, you will want to add
/// a blank `doc = "",` line at the start, or end respectively to split up the paragraphs.
///
///
/// ### Syntax
///
/// Use the verb as the path, with a parenthesized list
/// ```ignore
/// #[replace(
///     macro1(does stuff),
///     macro2 = "value",
///     // ...
/// )]
/// ```
/// With each item being the inside of a an attr macro, such as `#[macro1(does stuff)]` => `macro1(does stuff)`,
/// and same for `macro2` in the example.
///
/// This allows you to quickly add or replace multiple macros at once, with each macro being replaced/added in top-down order.
///
/// ### Macro Groups
/// Verbs work on the concept of macro "groups", which are simply all instances of a macro with the same path, for example:
///
/// ```ignore
/// #[doc = "One line of docs..."]
/// #[doc = ""]
/// #[doc = "And another!"]
/// ```
/// Acts as one macro group (`doc`), whereas
///
/// ```ignore
/// #[doc = "Number of apples in this fruit bowl."]
/// #[serde(default)]
/// #[serde(rename = "apples")]
/// ```
/// Is two macro groups (one `doc`, and the two `serde` macros).
///
/// This means that if you do:
/// ```ignore
/// #[replace(
///     doc = "Here's my summary line of my custom documentation.",
///     doc = "",
///     doc = "And here are the details: there aren't any!",
/// )]
/// schema Example {
///     /* ... */
/// }
/// ```
/// *All* `#[doc = ...]` macros are replaced with those three that are listed, in source code order.
///
#[doc(hidden)]
pub mod macros {
    use crate::__::__;

    /// Remove all instances of this group of attribute macros from the target.
    ///
    /// For [delete], you simply use the path itself, like so:
    /// ```ignore
    /// #[delete(doc)]       // Removes all generated `#[doc = ...]` attr macros from this type.
    /// schema Example {
    ///     #[delete(serde)] // Remove any #[serde(...)] attr macros on this field.
    ///     field,
    /// }
    /// ```
    pub fn delete(_: __) -> __ {
        unreachable!()
    }

    /// Prepend the following attribute macros for the target in source order before any auto-generated macros.
    ///
    /// Note that this macro appends *the **entire block as-is** in source order*.
    ///
    /// **TIP** for documentation-writers: In most cases when prepending documentation, you will want to add
    /// a blank `doc = "",` line at the end to split up the paragraphs.
    ///
    /// ### Example
    /// ```ignore
    /// #[prepend(
    ///     doc = "One, ",
    ///     doc = "Two, ",
    ///     doc = "Three.",
    ///     doc = "",
    /// )]
    /// schema Example {}
    /// ```
    ///
    /// Would add the line `One, Two, Three.` to the start of the documentation.
    pub fn prepend(_: __) -> __ {
        unreachable!()
    }

    /// Replace the existing attribute macro groups for the target with the ones listed.
    ///
    /// ### Example
    /// ```ignore
    /// schema Example {
    ///     #[replace(doc = "Now it's just me!")] // Replaces the entire documentation for this field.
    ///     field: u32,
    /// }
    /// ```
    pub fn replace(_: __) -> __ {
        unreachable!()
    }

    /// Append the following attribute macros for the target, after any auto-generated macros.
    ///
    /// **TIP** for documentation-writers: In most cases when appending documentation, you will want to add
    /// a blank `doc = "",` line at the start to split up the paragraphs.
    ///
    /// ### Example
    /// ```ignore
    /// #[append(
    ///     doc = "",
    ///     doc = "One, ",
    ///     doc = "Two, ",
    ///     doc = "Three.",
    /// )]
    /// schema Example {}
    /// ```
    ///
    /// Would add the line `One, Two, Three.` to the end of the documentation.
    pub fn append(_: __) -> __ {
        unreachable!()
    }
}

/// `_`: The auto-ascribed type for this field.
///
/// You may use `_` as a placeholder for when you wish to reuse the type inside a container, e.g. `Option<_>`, `Box<_>`.
#[doc(hidden)]
#[derive(
    Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum OriginalType {}

impl Default for OriginalType {
    fn default() -> Self {
        unimplemented!()
    }
}

#[doc(hidden)]
pub mod reexports {
    pub use serde;
}
