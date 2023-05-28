use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_quote, punctuated::Punctuated, DeriveInput};

extern crate proc_macro;

/// This macro specifies a system use it to create a sytem with either built in components or define your
/// own coponents using the Component derive Macro. You also need to tell tarator you want to actually use
/// the System using the InitSystems macro.
///
/// ## Example:
/// ```ignore
/// #[derive(Component)]
/// pub struct Item(String);
///
/// #[System(Update)]
/// pub fn move_items(items: Vec<(Transform, Item)>, state: GameState<()>) {
///     for (transform, item) in items {
///         let before = transform.pos;
///         transform.pos.x += 1.0 * state.dt;
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

    let mut state_arg = None;

    for input in inputs {
        let pat = match input {
            syn::FnArg::Receiver(_) => panic!("self not supported in function signature"),
            syn::FnArg::Typed(t) => t,
        };

        let name = match *pat.pat {
            syn::Pat::Ident(i) => i.ident,
            _ => panic!("queries should be of type: 'Vec<(Component1, Component2)>'"),
        };

        let mut is_mut = false;
        let mut is_state = false;

        let bundle_type = match *pat.ty {
            syn::Type::Path(p) => {
                let last = p.path.segments.into_iter().last().unwrap();

                match last.ident.to_string().as_str() {
                    "Query" => is_mut = false,
                    "QueryMut" => is_mut = true,
                    "GameState" => is_state = true,
                    _ => panic!("only Query and QueryMut are supported as function parameters"),
                }
                if is_state {
                    None
                } else {
                    Some(match last.arguments {
                        syn::PathArguments::AngleBracketed(a) => a.args,
                        _ => panic!(
                        "queries should have a structure like: 'Query<(Component1, Component2)>'"
                    ),
                    })
                }
            }
            _ => panic!("queries should have a structure like: 'Query<(Component1, Component2)>'"),
        };
        if is_state {
            let arg: syn::FnArg = parse_quote!(#name: &::scr_types::game_state::GameState);
            if state_arg.is_some() {
                panic!("only one instance of game_state is allowed")
            }
            state_arg = Some(arg);
        } else {
            let new_stmt: syn::Stmt = if let Some(bundle_type) = bundle_type {
                if is_mut {
                    parse_quote!(let #name = world.get_component_query_mut::<#bundle_type>();)
                } else {
                    parse_quote!(let #name = world.get_component_query::<#bundle_type>();)
                }
            } else {
                panic!("queries should have a structure like: 'Query<(Component1, Component2)>'");
            };
            new_stmts.push(new_stmt);
        }
    }

    func.sig.inputs.push(if let Some(arg) = state_arg {
        arg
    } else {
        parse_quote!(_state: &::scr_types::game_state::GameState)
    });

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
/// ```ignore
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

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let mut ast = syn::parse_macro_input!(input as DeriveInput);
    ast.attrs.push(parse_quote!( #[derive(tar_ecs::component::Component)] ));
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, type_generics, where_clause) = &generics.split_for_impl();
    let serde_name = format!("\"{name}\"");

    quote!(
        unsafe impl #impl_generics tar_ecs::component::Component for #name #type_generics #where_clause {}

        impl #impl_generics SerdeComponent for #name #type_generics #where_clause {
            const NAME: &'static str = #serde_name;
        }
    ).into()
}

