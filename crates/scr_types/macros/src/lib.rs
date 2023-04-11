use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated};

extern crate proc_macro;

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn System(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);

    let mut func = if let syn::Item::Fn(func) = ast {
        func
    } else {
        panic!("System macro can only be applied to functions")
    };

    let name = func.sig.ident.to_string();

    let inputs = func.sig.inputs;

    let arg: syn::FnArg = parse_quote!(world: ::tar_ecs::world::World);

    func.sig.inputs = Punctuated::new();
    func.sig.inputs.push(arg);

    let mut new_stmts = vec![];

    for input in inputs {
        let pat = match input {
            syn::FnArg::Receiver(_) => panic!("self not supported in function signature"),
            syn::FnArg::Typed(t) => t,
        };

        // inputs.push(quote!(#pat).to_string());

        // let name = if let syn::Pat::Ident(i) = *pat.pat.clone() {
        //     i.ident
        // } else {
        //     unreachable!()
        // };

        let new_stmt: syn::Stmt = parse_quote!(let #pat = world.component_query_mut(););
        new_stmts.push(new_stmt);
    }

    // add the requested bundles/components
    let mut tmp = func.block.stmts;
    func.block.stmts = new_stmts;
    func.block.stmts.append(&mut tmp);

    println!("{name}");
    quote!(#func).into()
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn InitSystems(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(item as syn::Item);

    let mut func = if let syn::Item::Fn(func) = ast {
        func
    } else {
        panic!("InitSystem macro can only be applied to functions")
    };

    // set the name of the function to the constant "init_system" and make shure it always returns
    // "Systems"
    func.sig.ident = syn::Ident::new("init_system", proc_macro2::Span::call_site());
    func.sig.output = parse_quote!(::scr_types::Systems);

    // add no_mangle attribute to preserve function name after compilation
    func.attrs.push(parse_quote!(#[no_mangle]));

    quote!(#func).into()
}
