#![feature(local_key_cell_methods)]

mod object;
mod pool;
mod tool;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    tool::test(_attr, input)
}

#[proc_macro_derive(Object)]
pub fn object(input: TokenStream) -> TokenStream {
    tool::object(input)
}
