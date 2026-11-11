mod matching;
mod patches;

use quote::quote;

#[proc_macro]
pub fn patches(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let patches::Patches(mut items) = syn::parse_macro_input!(tokens);

    let question = patches::ide::question(&mut items);
    let keywords = patches::ide::keywords(&mut items);
    let verbs = patches::ide::verbs(&mut items);
    let schemas = patches::ide::schema(&mut items);

    quote! {
        #question
        #keywords
        #verbs
        #(#schemas)*
    }
    .into()
}
