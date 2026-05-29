//! Procedural macros that expand into compile-time heavy code paths.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Duplicates function body into nested const-eval style blocks (compile pressure).
#[proc_macro_attribute]
pub fn stress_expand(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let block = &input.block;

    let const_name = syn::Ident::new(&format!("{name}_const_pressure"), name.span());

    let expanded = quote! {
        #input

        #[doc(hidden)]
        #[allow(dead_code)]
        fn #const_name() -> usize {
            let out: usize = (|| #block)();
            out
        }
    };

    expanded.into()
}

/// Generates a deeply nested generic wrapper type at compile time.
#[proc_macro]
pub fn generic_chain(input: TokenStream) -> TokenStream {
    let depth: usize = input.to_string().parse().unwrap_or(8);
    let depth = depth.clamp(1, 64);

    let mut wrappers = Vec::new();
    let mut inner = quote! { () };

    for i in 0..depth {
        let ident = syn::Ident::new(&format!("Wrap{i}"), proc_macro2::Span::call_site());
        wrappers.push(quote! {
            struct #ident<T>(T);
            impl<T> #ident<T> {
                fn id(self) -> T { self.0 }
            }
        });
        inner = quote! { #ident<#inner> };
    }

    let output = quote! {
        #(#wrappers)*
        pub type Chain = #inner;
    };

    output.into()
}
