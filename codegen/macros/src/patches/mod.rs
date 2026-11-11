// TODO: Add postfix `?` for entire schema, too.
// TODO: Only the four macro verbs are allowed at the top level for an attribute macro.

#![expect(unused)] // We eventually will move a lot of this into a common crate for codegen and IDE support.
pub mod ide;
pub mod utils;

use proc_macro2::Span;
use syn::{Token, parse::Parse, punctuated::Punctuated};

pub struct Patches(pub Vec<Item>);

impl Parse for Patches {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut items = Vec::new();
        while !input.is_empty() {
            items.push(input.parse()?);
        }

        Ok(Self(items))
    }
}

pub enum Item {
    Schema(ItemSchema),
    Operation(ItemOperation),
}

impl Parse for Item {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse(input)?;

        if input.peek(kw::schema) {
            let mut item: ItemSchema = input.parse()?;
            item.attrs = attrs;
            return Ok(Self::Schema(item));
        }

        if input.peek(kw::operation) {
            let mut item: ItemOperation = input.parse()?;
            item.attrs = attrs;
            return Ok(Self::Operation(item));
        }

        Err(syn::Error::new(
            input.span(),
            "Expected either `use`, `schema`, or `operation`",
        ))
    }
}

pub mod kw {
    // Actual Keywords
    syn::custom_keyword!(schema);
    syn::custom_keyword!(at);
    syn::custom_keyword!(operation);

    // Macro Verbs
    syn::custom_keyword!(prepend);
    syn::custom_keyword!(replace);
    syn::custom_keyword!(append);
    syn::custom_keyword!(delete);
}

pub enum RawField {
    Identifier(syn::Ident),
    LiStr(syn::LitStr),
}

pub struct SelectorAt {
    pub at: kw::at,
    pub selector: Sel,
}

// TODO: Warning against using reserved keywords (at) within selectors.

pub enum Sel {
    /// An identifier: `example`.
    Ident(Ident),
    /// An absolute path with possible path parameters,
    /// in square brackets: `[/order/{id}]`.
    Path(SelPath),
    /// Access of a named property using an identifier (`sel.field`)
    /// or using raw property string (`sel."weird-field"`).
    Property(SelProperty),
    /// TODO: rename
    MethodCall(SelMethod),
}

impl Parse for Sel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut base = if input.peek(syn::token::Bracket) {
            Self::Path(input.parse()?)
        } else {
            Self::Ident(input.parse()?)
        };

        while input.peek(Token![.]) {
            let dot_token = input.parse()?;
            let ident: Ident = input.parse()?;

            if input.peek(syn::token::Paren) {
                let inner;
                let paren_token = syn::parenthesized!(inner in input);
                let args = Punctuated::parse_terminated(&inner)?;

                base = Self::MethodCall(SelMethod {
                    sel: Box::new(base),
                    dot_token,
                    method: ident,
                    paren_token,
                    args,
                });

                continue;
            }

            base = Self::Property(SelProperty {
                base: Box::new(base),
                dot_token,
                property: ident,
            });
        }

        Ok(base)
    }
}

pub struct SelPath {
    pub bracket: syn::token::Bracket,
    pub path: syn::LitStr,
}

impl Parse for SelPath {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inner;
        Ok(Self {
            bracket: syn::bracketed!(inner in input),
            path: inner.parse()?,
        })
    }
}

pub struct SelProperty {
    pub base: Box<Sel>,
    pub dot_token: Token![.],
    pub property: Ident,
}
pub struct SelMethod {
    pub sel: Box<Sel>,
    pub dot_token: Token![.],
    pub method: Ident,
    pub paren_token: syn::token::Paren,
    pub args: Punctuated<syn::Expr, Token![,]>,
}

pub enum Ident {
    Ident(syn::Ident),
    LitStr(syn::LitStr),
}

impl Parse for Ident {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitStr) {
            Ok(Self::LitStr(input.parse()?))
        } else {
            Ok(Self::Ident(input.parse()?))
        }
    }
}

impl Ident {
    pub fn span(&self) -> Span {
        match self {
            Ident::Ident(ident) => ident.span(),
            Ident::LitStr(lit_str) => lit_str.span(),
        }
    }
}

pub enum Selector {
    Identifier(syn::Ident),
    At(kw::at, Sel),
}
impl Selector {
    fn get_ident(&self) -> Option<&syn::Ident> {
        match self {
            Selector::Identifier(ident) | Selector::At(_, Sel::Ident(Ident::Ident(ident))) => {
                Some(ident)
            }
            _ => None,
        }
    }
}

impl Parse for Selector {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::at) {
            Ok(Self::At(input.parse()?, input.parse()?))
        } else {
            Ok(Self::Identifier(input.parse()?))
        }
    }
}

pub struct Renaming {
    pub as_token: Token![as],
    pub alias: Alias,
}

impl Renaming {
    pub fn get_ident(&self) -> Option<&syn::Ident> {
        match &self.alias {
            Alias::Ident(ident) => Some(ident),
            Alias::Debug(_) => None,
        }
    }
}

pub enum Alias {
    Ident(syn::Ident),
    Debug(Token![?]),
}

impl Parse for Renaming {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let as_token = input.parse()?;

        let alias = if input.peek(Token![?]) {
            Alias::Debug(input.parse()?)
        } else {
            Alias::Ident(input.parse()?)
        };

        Ok(Self { as_token, alias })
    }
}

pub struct ItemSchema {
    pub attrs: Vec<Attribute>,
    pub schema_token: kw::schema,
    pub selector: Selector,
    pub renaming: Option<Renaming>,
    pub brace_token: syn::token::Brace,
    pub fields: Punctuated<FieldOverride, Token![,]>,
    pub rest: Option<Token![..]>,
}

impl Parse for ItemSchema {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let in_braces;
        let is_trailing: bool;

        Ok(Self {
            attrs: Attribute::parse(input)?,
            schema_token: input.parse()?,
            selector: input.parse()?,
            renaming: if input.peek(Token![as]) {
                Some(input.parse()?)
            } else {
                None
            },
            brace_token: syn::braced!(in_braces in input),
            fields: {
                let fields = Punctuated::parse_terminated(&in_braces)?;
                is_trailing = fields.trailing_punct();
                fields
            },
            rest: if input.peek(Token![..]) {
                let rest = input.parse()?;
                if is_trailing {
                    Some(rest)
                } else {
                    return Err(syn::Error::new_spanned(
                        rest,
                        "Must have a trailing comma before `..`",
                    ));
                }
            } else {
                None
            },
        })
    }
}

// TODO: warn on just original name -- unused patch...

pub struct FieldOverride {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub renaming: Option<Renaming>,
    pub ty: Option<(Token![:], Type)>,
}

impl Parse for FieldOverride {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: Attribute::parse(input)?,
            ident: input.parse()?,
            renaming: if input.peek(Token![as]) {
                Some(input.parse()?)
            } else {
                None
            },
            ty: if input.peek(Token![:]) {
                Some((input.parse()?, input.parse()?))
            } else {
                None
            },
        })
    }
}

#[derive(Clone)]
pub enum Type {
    Debug(Token![?]),
    Type {
        ty: Box<syn::Type>,
        debug: Option<Token![?]>,
    },
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![?]) {
            return Ok(Self::Debug(input.parse()?));
        }

        Ok(Self::Type {
            ty: input.parse()?,
            debug: if input.peek(Token![?]) {
                Some(input.parse()?)
            } else {
                None
            },
        })
    }
}

#[non_exhaustive]
pub struct ItemOperation {
    pub attrs: Vec<Attribute>,
    pub operation_token: kw::operation,
    pub selector: Selector,
    pub renaming: Option<Renaming>,
    pub brace_token: syn::token::Brace,
}

impl Parse for ItemOperation {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _inner;
        Ok(Self {
            attrs: Attribute::parse(input)?,
            operation_token: input.parse()?,
            selector: input.parse()?,
            renaming: if input.peek(Token![as]) {
                Some(input.parse()?)
            } else {
                None
            },
            brace_token: syn::braced!(_inner in input),
        })
    }
}

#[derive(Clone, Copy)]
pub enum Verb {
    Delete(kw::delete),
    Prepend(kw::prepend),
    Replace(kw::replace),
    Append(kw::append),
}

impl PartialEq for Verb {
    fn eq(&self, other: &Self) -> bool {
        // Discount the actual span...
        matches!(
            (self, other),
            (Self::Prepend(_), Self::Prepend(_))
                | (Self::Replace(_), Self::Replace(_))
                | (Self::Append(_), Self::Append(_))
                | (Self::Delete(_), Self::Delete(_))
        )
    }
}

impl Parse for Verb {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match input {
            input if input.peek(kw::delete) => Ok(Self::Delete(input.parse()?)),
            input if input.peek(kw::prepend) => Ok(Self::Prepend(input.parse()?)),
            input if input.peek(kw::replace) => Ok(Self::Replace(input.parse()?)),
            input if input.peek(kw::append) => Ok(Self::Append(input.parse()?)),
            _ => Err(syn::Error::new(
                input.span(),
                "Expected a macro verb here: `delete`, `prepend`, `replace`, `append`.",
            )),
        }
    }
}

pub struct Attribute {
    pub pound_token: Token![#],
    pub bracket_token: syn::token::Bracket,
    pub verb: Verb,
    pub paren_token: syn::token::Paren,
    pub attrs: Punctuated<syn::Meta, Token![,]>,
}

impl Attribute {
    pub fn single_parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let outer;
        let inner;
        Ok(Self {
            pound_token: input.parse()?,
            bracket_token: syn::bracketed!(outer in input),
            verb: outer.parse()?,
            paren_token: syn::parenthesized!(inner in outer),
            attrs: Punctuated::parse_terminated(&inner)?,
        })
    }

    pub fn parse(input: syn::parse::ParseStream) -> syn::Result<Vec<Self>> {
        let mut attrs = Vec::new();
        while input.peek(Token![#]) {
            attrs.push(input.call(Self::single_parse)?);
        }
        Ok(attrs)
    }
}

// TODO: Lint to combine multiple of the same veb macro.
// TODO: Lint on empty verb macro.
