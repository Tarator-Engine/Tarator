extern crate proc_macro;

use syn::{__private::quote::quote, parse_quote};
use tar_types::script::{FileContent, Frequency, System};

use proc_macro::TokenStream;

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn System(attr: TokenStream, item: TokenStream) -> TokenStream {
    //TODO!: this works but it sucks
    let frequency = attr
        .into_iter()
        .next()
        .expect("You have to define a frequency")
        .to_string();

    let frequency = if frequency == "Update" {
        Frequency::Update
    } else if frequency == "Startup" {
        Frequency::Startup
    } else if frequency == "FixedUpdate" {
        Frequency::FixedUpdate
    } else {
        panic!("{frequency} is not a valid frequency");
    };

    let ast = syn::parse_macro_input!(item as syn::Item);

    let mut func = if let syn::Item::Fn(func) = ast {
        func
    } else {
        panic!("System macro can only be applied to functions")
    };

    let name = func.sig.ident.to_string();

    // add no_mangle attribute to preserve function name after compilation
    func.attrs.push(parse_quote!(#[no_mangle]));

    let ins = &mut func.sig.inputs;

    let mut new_stmts = vec![];

    let null_ptr: syn::Type = parse_quote!(*mut ::libc::c_void);

    let mut inputs = vec![];

    for input in ins {
        let pat = match input {
            syn::FnArg::Receiver(_) => panic!("self not supported in function signature"),
            syn::FnArg::Typed(t) => t,
        };

        inputs.push(quote!(#pat).to_string());

        let name = if let syn::Pat::Ident(i) = *pat.pat.clone() {
            i.ident
        } else {
            unreachable!()
        };

        let new_stmt: syn::Stmt = parse_quote!(let #pat = unsafe {::std::mem::transmute(#name)};);
        new_stmts.push(new_stmt);
        *pat.ty = null_ptr.clone();
        println!("input");
    }

    // add the conversions to usable types
    let mut tmp = func.block.stmts;
    func.block.stmts = new_stmts;
    func.block.stmts.append(&mut tmp);

    println!("{name}");

    let system = FileContent::System(System {
        name: name.clone(),
        inputs,
        frequency,
    });

    let system_string = ron::to_string(&system).unwrap();

    std::fs::write(format!("../.scr/{name}.scr"), system_string).unwrap();

    quote!(#func).into()
}
