extern crate proc_macro;

use tar_types::script::{FileContent, Frequency, System};

use proc_macro::{TokenStream, TokenTree};

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn System(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut fin = String::from("use libc::c_void;");

    fin.push_str("use tar_ecs::world::World;");
    fin.push_str("#[no_mangle]fn ");

    let mut item = item.into_iter();

    let frequency = attr.into_iter().next().unwrap().to_string();

    let frequency = if frequency == "Update" {
        Frequency::Update
    } else if frequency == "Startup" {
        Frequency::Startup
    } else if frequency == "FixedUpdate" {
        Frequency::FixedUpdate
    } else {
        panic!("{frequency} is not a valid frequency");
    };

    if item.next().unwrap().to_string() != "fn" {
        panic!("no additional function clarification required");
    }
    let name = item.next().unwrap().to_string();

    let system = FileContent::System(System {
        name: name.clone(),
        frequency,
    });

    let system_string = ron::to_string(&system).unwrap();

    std::fs::write(format!("../.scr/{name}.scr"), system_string).unwrap();

    fin.push_str(&name);
    fin.push_str("(world: *mut c_void) {");
    fin.push_str("let world: &mut World = unsafe{ std::mem::transmute(world) };");

    parse_args(item.next().unwrap(), &mut fin);
    fin.push_str(&(item.next().unwrap().to_string()));

    fin.push_str("}");
    fin.parse().unwrap()
}

fn parse_args(args: TokenTree, fin: &mut String) {
    if let TokenTree::Group(g) = args {
        let mut s = g.stream().into_iter();
        let mut done = false;
        while !done {
            let t = s.next();
            if let Some(TokenTree::Ident(p)) = t {
                fin.push_str("let mut ");
                fin.push_str(&p.to_string());
                fin.push_str(" = world.component_query_mut::<");
                s.next();
                match s.next().unwrap() {
                    TokenTree::Ident(i) => {
                        let name = i.to_string();
                        if name == "Iter" {
                            s.next();
                            fin.push_str(&(s.next().unwrap().to_string()));
                            fin.push_str(">();");
                            s.next();
                            s.next();
                        } else {
                            fin.push_str(&(name));
                            fin.push_str(">().next().unwrap();");
                            s.next();
                        }
                    }
                    TokenTree::Group(g) => {
                        fin.push_str(&(g.to_string()));
                        fin.push_str(">().next().unwrap();");
                        s.next();
                    }
                    _ => panic!("something went very wrong"),
                }
            } else {
                done = true;
            }
        }
    }
}
