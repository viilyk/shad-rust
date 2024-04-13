use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};

#[proc_macro_derive(Scan)]
pub fn derive_scan(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as DeriveInput);

    let struct_ = match input.data {
        Data::Struct(struct_) => struct_,
        _ => panic!("Expected Data::Struct"),
    };

    let DeriveInput { ident, .. } = input;

    let syn::Fields::Named(fields) = struct_.fields else {
            let impl_ = quote! {
                impl gc::Scan for #ident {
                    fn get_ref_addr(&self, c: &mut dyn FnMut(usize)) {}
                }
            };
            return TokenStream::from(impl_);
    };

    let fields = fields.named.iter().map(|field| { 
        let field_ident = field.ident.as_ref().unwrap();
        quote!(self.#field_ident)
    }).collect::<Vec<_>>();

    let impl_ = quote! {
        impl gc::Scan for #ident {
            fn get_ref_addr(&self, c: &mut dyn FnMut(usize)) {
                #(#fields.get_ref_addr(c));*
            }
        }
    };

    impl_.into()
}
