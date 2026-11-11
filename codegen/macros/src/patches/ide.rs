//! IDE Autocomplete helpers for the `patches!` macro.

use proc_macro2::{Span, TokenStream, TokenTree, extra::DelimSpan};
use quote::{ToTokens, quote};
use syn::{spanned::Spanned, token};

use crate::patches::{
    Alias, Attribute, Item, ItemOperation, ItemSchema, Renaming, Selector, Type, Verb,
    utils::{Either, Partition, Visitor, path_last_ident, use_path_last_ident},
};

/// Create an inaccessible anonymous `const` shell which is hidden from any docs
/// to allow for IDE highlighting using the "Span" trick.
fn shell(tokens: impl IntoIterator<Item = TokenTree>) -> TokenStream {
    let tokens = tokens.into_iter();
    quote! {
        #[cfg(rust_analyzer)]
        #[allow(unexpected_cfgs)]
        #[doc(hidden)]
        const _: () = {
            #(#tokens)*
        };
    }
}

fn exposed_shell(tokens: impl IntoIterator<Item = TokenTree>) -> TokenStream {
    let tokens = tokens.into_iter();
    quote! {
        #[doc(hidden)]
        const _: () = {
            #(#tokens)*
        };
    }
}

/// Generates functions which generate a corresponding `use ... as _` statement.
///
/// Access these functions through `using::*` module.
macro_rules! using {
    ($($(#[$($attr: tt)*])* $f: ident => $p: path);*$(;)?) => {
        mod using {
            $(
                $(#[$($attr)*])*
                pub fn $f() -> syn::ItemUse {
                    syn::parse2(quote::quote!(
                        use $p as _;
                    ))
                    .unwrap()
                }
            )*
        }
    };
}

using! {
    question => taxman::__::question;
    at => taxman::__::at;
    schema => taxman::__::schema;
    operation => taxman::__::operation;

    // Verb Macros
    delete => taxman::__::macros::delete;
    prepend => taxman::__::macros::prepend;
    replace => taxman::__::macros::replace;
    append => taxman::__::macros::append;
}

/// Check if there are any `?` debug marks.
/// If so, span-trick into linking to the `use ...::question`
pub fn question(items: &mut [Item]) -> TokenStream {
    #[derive(Default)]
    struct Questions(Vec<Span>);

    impl Visitor<Item> for Questions {
        fn visit(&mut self, value: &mut Item) {
            match value {
                Item::Schema(ItemSchema {
                    renaming, fields, ..
                }) => {
                    if let Some(renaming) = renaming {
                        self.visit(renaming);
                    }

                    fields.iter_mut().for_each(|field| {
                        if let Some(renaming) = &mut field.renaming {
                            self.visit(renaming);
                        }

                        if let Some((_, ty)) = &mut field.ty {
                            self.visit(ty);
                        }
                    });
                }
                Item::Operation(ItemOperation {
                    renaming: Some(renaming),
                    ..
                }) => {
                    self.visit(renaming);
                }
                _ => (),
            }
        }
    }

    impl Visitor<Renaming> for Questions {
        fn visit(&mut self, value: &mut Renaming) {
            if let Alias::Debug(question) = value.alias {
                self.0.push(question.span)
            }
        }
    }

    impl Visitor<Type> for Questions {
        fn visit(&mut self, value: &mut Type) {
            match value {
                Type::Debug(question) => self.0.push(question.span),
                Type::Type {
                    debug: Some(question),
                    ..
                } => self.0.push(question.span),
                _ => (),
            }
        }
    }

    let mut visitor = Questions::default();

    for item in items {
        visitor.visit(item);
    }

    let use_ = using::question();
    let uses = visitor.0.into_iter().flat_map(|span| {
        let mut use_ = use_.clone();
        let ident = use_path_last_ident(&mut use_.tree)
            .expect("our path always has a rename, which will return Some(_)");

        ident.set_span(span);
        use_.into_token_stream()
    });

    shell(uses)
}

fn link(instances: Vec<Span>, path: fn() -> syn::ItemUse) -> impl Iterator<Item = TokenTree> {
    let path = path();
    instances.into_iter().flat_map(move |span| {
        let mut path = path.clone();
        use_path_last_ident(&mut path.tree)
            .expect("our path always has a rename, which will return Some(_)")
            .set_span(span);

        path.into_token_stream()
    })
}

/// Map spans of keywords into `use`s to allow for docs to be shown by rust-analyzer.
pub fn keywords(items: &mut [Item]) -> TokenStream {
    #[derive(Default)]
    struct Keywords {
        at: Vec<Span>,
        schema: Vec<Span>,
        operation: Vec<Span>,
    }

    impl Visitor<Item> for Keywords {
        fn visit(&mut self, value: &mut Item) {
            match value {
                Item::Schema(ItemSchema {
                    schema_token,
                    selector,
                    ..
                }) => {
                    self.schema.push(schema_token.span);
                    self.visit(selector);
                }
                Item::Operation(ItemOperation {
                    operation_token,
                    selector,
                    ..
                }) => {
                    self.operation.push(operation_token.span);
                    self.visit(selector);
                }
            }
        }
    }

    impl Visitor<Selector> for Keywords {
        fn visit(&mut self, value: &mut Selector) {
            if let Selector::At(at, _) = value {
                self.at.push(at.span)
            }
        }
    }

    let mut keywords = Keywords::default();

    for item in items {
        keywords.visit(item);
    }

    let Keywords {
        at,
        schema,
        operation,
    } = keywords;

    shell(
        link(at, using::at)
            .chain(link(schema, using::schema))
            .chain(link(operation, using::operation)),
    )
}

pub fn verbs(items: &mut [Item]) -> TokenStream {
    #[derive(Default)]
    struct Verbs {
        delete: Vec<Span>,
        prepend: Vec<Span>,
        replace: Vec<Span>,
        append: Vec<Span>,
    }

    impl Verbs {
        pub fn add(&mut self, verb: Verb) {
            match verb {
                Verb::Delete(delete) => self.delete.push(delete.span),
                Verb::Prepend(prepend) => self.prepend.push(prepend.span),
                Verb::Replace(replace) => self.replace.push(replace.span),
                Verb::Append(append) => self.append.push(append.span),
            }
        }
    }

    impl Visitor<Item> for Verbs {
        fn visit(&mut self, value: &mut Item) {
            match value {
                Item::Schema(ItemSchema { attrs, fields, .. }) => {
                    fields
                        .iter()
                        .flat_map(|field| field.attrs.iter())
                        .chain(attrs.iter())
                        .for_each(|Attribute { verb, .. }| self.add(*verb));
                }
                Item::Operation(ItemOperation { attrs, .. }) => {
                    attrs
                        .iter()
                        .for_each(|Attribute { verb, .. }| self.add(*verb));
                }
            }
        }
    }

    let mut verbs = Verbs::default();

    for item in items {
        verbs.visit(item);
    }

    let Verbs {
        delete,
        prepend,
        replace,
        append,
    } = verbs;

    shell(
        link(delete, using::delete)
            .chain(link(prepend, using::prepend))
            .chain(link(replace, using::replace))
            .chain(link(append, using::append)),
    )
}

fn to_rust_attr(attr: &Attribute) -> Box<dyn Iterator<Item = syn::Attribute> + '_> {
    if matches!(attr.verb, Verb::Delete(_)) {
        return Box::new(std::iter::empty());
    }

    Box::new(attr.attrs.iter().cloned().map(|meta| syn::Attribute {
        pound_token: attr.pound_token,
        style: syn::AttrStyle::Outer,
        bracket_token: Default::default(),
        meta,
    }))
}

const DEFAULT_TYPE_NAME: &str = "_Anonymous";
fn default_type_name() -> syn::Ident {
    syn::Ident::new(DEFAULT_TYPE_NAME, Span::call_site())
}

fn inferred_type_name(span: Span) -> syn::Type {
    let mut ty: syn::Type = syn::parse2(quote!(taxman::__::OriginalType)).unwrap();

    {
        let syn::Type::Path(syn::TypePath { path, .. }) = &mut ty else {
            unimplemented!("type will always a path")
        };
        let ident = path_last_ident(path).expect("path will always have last ident");
        ident.set_span(span);
    }

    ty
}

fn replace_wildcard(ty: &mut syn::Type, mut with: impl FnMut(&mut syn::Type, Span) + Copy) {
    match ty {
        ty @ syn::Type::Infer(syn::TypeInfer { .. }) => with(ty, ty.span()),

        syn::Type::Array(syn::TypeArray { elem, .. })
        | syn::Type::Group(syn::TypeGroup { elem, .. })
        | syn::Type::Paren(syn::TypeParen { elem, .. })
        | syn::Type::Reference(syn::TypeReference { elem, .. })
        | syn::Type::Slice(syn::TypeSlice { elem, .. })
        | syn::Type::Ptr(syn::TypePtr { elem, .. }) => replace_wildcard(elem, with),
        syn::Type::Tuple(syn::TypeTuple { elems, .. }) => elems
            .iter_mut()
            .for_each(|elem| replace_wildcard(elem, with)),
        syn::Type::Path(syn::TypePath { path, .. }) => path
            .segments
            .iter_mut()
            .map(|seg| &mut seg.arguments)
            .filter_map(|args| match args {
                syn::PathArguments::AngleBracketed(angled) => Some(angled),
                _ => None,
            })
            .flat_map(|a| {
                a.args.iter_mut().filter_map(|arg| match arg {
                    syn::GenericArgument::Type(t) => Some(t),
                    _ => None,
                })
            })
            .for_each(|elem| replace_wildcard(elem, with)),
        _ => (),
    }
}

fn derive_serialize_deserialize(
    attrs: impl IntoIterator<Item = syn::Attribute>,
    braces_span: DelimSpan,
) -> impl Iterator<Item = syn::Attribute> {
    let attrs = attrs.into_iter();

    attrs.chain([
        syn::Attribute {
            pound_token: token::Pound {
                spans: [Span::mixed_site()],
            },
            style: syn::AttrStyle::Outer,
            bracket_token: token::Bracket { span: braces_span },
            meta: syn::Meta::List(syn::MetaList {
                path: syn::Ident::new("derive", Span::call_site()).into(),
                delimiter: syn::MacroDelimiter::Paren(syn::token::Paren { span: braces_span }),
                tokens: quote!(
                    taxman::__::reexports::serde::Serialize,
                    taxman::__::reexports::serde::Deserialize
                ),
            }),
        },
        syn::Attribute {
            pound_token: Default::default(),
            style: syn::AttrStyle::Outer,
            bracket_token: Default::default(),
            meta: syn::Meta::List(syn::MetaList {
                path: syn::Ident::new("serde", Span::call_site()).into(),
                delimiter: syn::MacroDelimiter::Paren(syn::token::Paren { span: braces_span }),
                tokens: quote!(crate = "taxman::__::reexports::serde"),
            }),
        },
        syn::Attribute {
            pound_token: Default::default(),
            style: syn::AttrStyle::Outer,
            bracket_token: Default::default(),
            meta: syn::Meta::List(syn::MetaList {
                path: syn::Ident::new("allow", Span::call_site()).into(),
                delimiter: syn::MacroDelimiter::Paren(Default::default()),
                tokens: quote!(dead_code),
            }),
        },
    ])
}

/// Map spans of field names, types to an auto-generated struct to allow for IDE hover
/// information on `schema` fields.
pub fn schema(items: &mut [Item]) -> impl Iterator<Item = TokenTree> {
    items
        .iter()
        .flat_map(|item| -> Box<dyn Iterator<Item = _>> {
            let schema = match item {
                Item::Schema(schema) => schema,
                _ => return Box::new(std::iter::empty()),
            };

            // TODO: Eventually check and reconcile this to work like codegen.
            let attrs = schema.attrs.iter().flat_map(to_rust_attr);
            let ident = schema
                .renaming
                .as_ref()
                .and_then(Renaming::get_ident)
                .or_else(|| schema.selector.get_ident())
                .cloned()
                .unwrap_or_else(default_type_name);

            let (named, auto_named): (Vec<_>, Vec<_>) =
                schema
                    .fields
                    .iter()
                    .partition_with(|f| match (&f.ident, &f.renaming) {
                        (
                            _,
                            Some(Renaming {
                                alias: Alias::Ident(ident),
                                ..
                            }),
                        ) => Either::Left((ident, f)),
                        (original, _) => Either::Right((original, f)),
                    });

            // TODO: ensure that we actually rename according to codegen rules.

            Box::new(
                exposed_shell(
                    syn::ItemStruct {
                        attrs: derive_serialize_deserialize(attrs, schema.brace_token.span)
                            .collect(),
                        vis: syn::Visibility::Public(token::Pub {
                            span: Span::call_site(),
                        }),
                        struct_token: token::Struct {
                            span: Span::call_site(),
                        },
                        ident,
                        generics: syn::Generics::default(),
                        fields: syn::Fields::Named(syn::FieldsNamed {
                            brace_token: schema.brace_token,
                            named: named
                                .into_iter()
                                .map(|(ident, field)| (ident.clone(), field))
                                .chain(auto_named.into_iter().enumerate().map(|(i, (_, field))| {
                                    (
                                        syn::Ident::new(&format!("field_{i}"), Span::mixed_site()),
                                        field,
                                    )
                                }))
                                .map(|(ident, f)| {
                                    let (colon, ty) = f.ty.clone().unwrap_or_else(|| {
                                        (
                                            Default::default(),
                                            Type::Type {
                                                ty: Box::new(syn::Type::Infer(syn::TypeInfer {
                                                    attrs: Vec::new(),
                                                    underscore_token: Default::default(),
                                                })),
                                                debug: None,
                                            },
                                        )
                                    });

                                    syn::Field {
                                        attrs: f.attrs.iter().flat_map(to_rust_attr).collect(),
                                        vis: syn::Visibility::Public(Default::default()),
                                        modifiers: syn::FieldModifiers::default(),
                                        ident: Some(ident.clone()),
                                        colon_token: Some(colon),
                                        ty: match ty {
                                            Type::Debug(question) => {
                                                inferred_type_name(question.span())
                                            }
                                            Type::Type { mut ty, .. } => {
                                                replace_wildcard(&mut ty, |ty, span| {
                                                    *ty = inferred_type_name(span)
                                                });
                                                *ty
                                            }
                                        },
                                        default: None,
                                    }
                                })
                                .collect(),
                        }),
                        semi_token: None,
                    }
                    .into_token_stream(),
                )
                .into_iter(),
            )
        })
}
