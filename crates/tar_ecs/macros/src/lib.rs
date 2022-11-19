extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

static mut COMPONENTCOUNTER: usize = 0;

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_component_impl(&ast)
}

fn derive_component_impl(ast: &syn::DeriveInput) -> TokenStream {
    let t_ident = &ast.ident;
    let id = unsafe {
        let id = COMPONENTCOUNTER;
        COMPONENTCOUNTER += 1;
        id
    };
    let gen = quote! {
        impl Component for #t_ident {
            fn id() -> ComponentId {
                #id
            }
        }
    };    
    gen.into()
}

