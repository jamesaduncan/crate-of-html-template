use proc_macro::TokenStream;

#[proc_macro_derive(Renderable, attributes(renderable))]
pub fn derive_renderable(_input: TokenStream) -> TokenStream {
    // TODO: Implement in Phase 4
    TokenStream::new()
}