use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated};

extern crate proc_macro;

/// This macro specifies a system use it to create a sytem with either built in components or define your
/// own coponents using the Component derive Macro. You also need to tell tarator you want to actually use
/// the System using the InitSystems macro.
///
/// ## Example:
/// ```rust
/// #[derive(Component)]
/// pub struct Item(String);
///
/// #[System(Update)]
/// pub fn move_items(items: Vec<(Transform, Item)>) {
///     for (transform, item) in items {
///         let before = transform.pos;
///         transform.pos.x += 1;
///         let after = transform.pos;
///         println!("moved {item} from {before} to {after}");
///     }
/// }
///
/// #[InitSystems]
/// pub fn init() -> Systems {
///     Systems::new()
///         .add(move_items)
/// }
///
/// ```
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

    let arg: syn::FnArg = parse_quote!(world: &mut ::tar_ecs::prelude::World);

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

        // let bundle_type = match *pat.ty {
        //     syn::Type::
        // };
        // let is_vec = true;

        let name = match *pat.pat {
            syn::Pat::Ident(i) => i.ident,
            _ => panic!("queries should be of type: 'Vec<(Component1, Component2)>'"),
        };

        let is_mut: bool;

        let bundle_type = match *pat.ty {
            syn::Type::Path(p) => {
                let last = p.path.segments.into_iter().last().unwrap();

                match last.ident.to_string().as_str() {
                    "Query" => is_mut = false,
                    "QueryMut" => is_mut = true,
                    _ => panic!("only Query and QueryMut are supported as function parameters"),
                }

                match last.arguments {
                    syn::PathArguments::AngleBracketed(a) => a.args,
                    _ => panic!(
                        "queries should have a structure like: 'Query<(Component1, Component2)>'"
                    ),
                }
            }
            _ => panic!("queries should have a structure like: 'Query<(Component1, Component2)>'"),
        };

        let new_stmt: syn::Stmt = if is_mut {
            parse_quote!(let #name = world.component_query_mut::<#bundle_type>();)
        } else {
            parse_quote!(let #name = world.component_query::<#bundle_type>();)
        };
        new_stmts.push(new_stmt);
    }

    // add the requested bundles/components
    let mut tmp = func.block.stmts;
    func.block.stmts = new_stmts;
    func.block.stmts.append(&mut tmp);

    println!("{name}");
    quote!(#func).into()
}

/// This macro allows you to specify what systems you want to use and when you want them.
///
/// ## Example:
/// ```rust
/// #[derive(Component)]
/// pub struct Item(String);
///
/// #[System(Update)]
/// pub fn move_items(items: Vec<(Transform, Item)>) {
///     for (transform, item) in items {
///         let before = transform.pos;
///         transform.pos.x += 1;
///         let after = transform.pos;
///         println!("moved {item} from {before} to {after}");
///     }
/// }
///
/// #[InitSystems]
/// pub fn init() -> Systems {
///     Systems::new()
///         .add(move_items)
/// }
///
/// ```
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
    func.sig.ident = parse_quote!(init_systems);
    func.sig.output = parse_quote!(-> ::scr_types::Systems);

    // add no_mangle attribute to preserve function name after compilation
    func.attrs.push(parse_quote!(#[no_mangle]));

    quote!(#func).into()
}
