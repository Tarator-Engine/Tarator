extern crate proc_macro;

use std::sync::Mutex;
use proc_macro::TokenStream;
use quote::quote;
use syn;

static mut COMPONENTCOUNTER: Mutex<usize> = Mutex::new(0);

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_component_impl(&ast)
}

fn derive_component_impl(ast: &syn::DeriveInput) -> TokenStream {
    let t_ident = &ast.ident;
    let id = unsafe {
        let mut counter = COMPONENTCOUNTER.lock().unwrap();
        let id: usize = *counter;
        *counter += 1;
        id
    };
    let gen = quote! {
        impl Component for #t_ident {
            #[inline]
            fn id() -> ComponentId {
                #id
            }
        }
    };    
    gen.into()
}

