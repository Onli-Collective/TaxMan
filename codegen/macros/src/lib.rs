mod matching;

use quote::quote;

#[proc_macro_attribute]
pub fn overrides(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = proc_macro2::TokenStream::from(attrs);
    let item = proc_macro2::TokenStream::from(item);

    let ident = match syn::parse2::<syn::Ident>(attr) {
        Ok(ident) => quote! {#ident},
        Err(_) => quote! { self },
    };

    let hook = quote! {
        #[cfg(rust_analyzer)]
        #[allow(unexpected_cfgs)]
        #[doc(hidden)]
        const _: () = {
            #[allow(unused_imports)]
            use taxman::__::predicates::{#ident};
        };
    };

    quote! {
        #item
        #hook
    }
    .into()
}
