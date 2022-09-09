use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};
use syn::{parse_quote, ItemStruct, Stmt};

pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let vis = &input.vis;

    let result = quote! {
        #[test]
        #(#attrs)*
        #vis fn #name() #ret {
            futures::executor::block_on(async { #body })
        }
    };

    TokenStream::from(result)
}

pub fn object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = input.ident;

    let fields = input.fields.into_iter();
    let mut stmts: Vec<Stmt> = vec![];

    for field in fields {
        let name = field.ident;
        stmts.push(parse_quote!(
            self.#name.reuse();
        ))
    }

    let result = quote!({
        impl Object for #struct_name {
            fn reuse(self: &mut Self) {
                #(#stmts)*
            }
        }
    });

    TokenStream::from(result)
}

mod test {
    use syn::ItemImpl;

    use super::*;

    #[test]
    fn object_test() {
        // thread 'tool::test::object_test' panicked at 'procedural macro API is used outside of a procedural macro', library/proc_macro/src/bridge/client.rs:346:17

        let input = TokenStream::from(quote!(
            struct S {
                a: Vec<u8>,
                b: Vec<u8>,
            }
        ));

        let expect = TokenStream::from(quote!(
            impl Object for S {fn reuse(self: &mut Self) {a.reuse();b.reuse();}}
        ));
        let expect: ItemImpl = syn::parse(expect).unwrap();

        let result = object(input);
        let result: ItemImpl = syn::parse(result).unwrap();

        assert_eq!(expect, result);
    }
}
