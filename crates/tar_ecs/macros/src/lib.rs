extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    token::Comma,
    DeriveInput, Ident, LitInt, Result,
};

struct ForeachTuple {
    macro_ident: Ident,
    start: usize,
    end: usize,
    idents: Vec<Ident>,
}

impl Parse for ForeachTuple {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let start = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let end = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let mut idents = vec![input.parse::<Ident>()?];
        while input.parse::<Comma>().is_ok() {
            idents.push(input.parse::<Ident>()?);
        }

        Ok(Self {
            macro_ident,
            start,
            end,
            idents,
        })
    }
}

/// Implements a given macro for each tuple (number of fields provided)
#[proc_macro]
pub fn foreach_tuple(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ForeachTuple);
    let len = input.end - input.start;
    let mut ident_tuples = Vec::with_capacity(len);
    for i in input.start..=input.end {
        let idents = input
            .idents
            .iter()
            .map(|ident| format_ident!("{}{}", ident, i));
        if input.idents.len() < 2 {
            ident_tuples.push(quote! {
                #(#idents)*
            });
        } else {
            ident_tuples.push(quote! {
                (#(#idents),*)
            });
        }
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let ident_tuples = &ident_tuples[..i];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Sized + Send + Sync + 'static });

    let name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    quote! {
        unsafe impl #impl_generics Component for #name #type_generics #where_clause {}
    }
    .into()
}

#[proc_macro_derive(Callback)]
pub fn derive_callback(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let (impl_generics, type_generics, where_clause) = &ast.generics.split_for_impl();

    quote! {
        unsafe impl #impl_generics InnerCallback for #name #type_generics #where_clause {}

        impl #impl_generics Callback<Empty> for #name #type_generics #where_clause {
            fn callback(&mut self, _: &mut Empty) {}
        }
    }
    .into()
}

struct Identifier {
    ident: Ident,
    int: Ident,
}

impl Parse for Identifier {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let int = input.parse::<Ident>()?;
        Ok(Self { ident, int })
    }
}

#[proc_macro]
pub fn identifier(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as Identifier);
    let name = ast.ident;
    let int = ast.int;

    quote! {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct #name(#int);

        impl #name {
            pub const EMPTY: Self = Self::new(0);
            pub const INVALID: Self = Self::new(#int::MAX);

            #[inline]
            pub const fn new(index: #int) -> Self {
                Self(index)
            }

            #[inline]
            pub const fn id(self) -> #int {
                self.0
            }
        }

        impl crate::store::sparse::SparseSetIndex for #name {
            #[inline]
            fn from_usize(value: usize) -> Self {
                Self::new(value as #int)
            }

            #[inline]
            fn as_usize(&self) -> usize {
                self.0 as usize
            }
        }

    }
    .into()
}
