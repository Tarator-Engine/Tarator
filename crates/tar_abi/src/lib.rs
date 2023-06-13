//! This crate is used for loading and accessing compiled scripts (via the compiled scripts internal.rs)

use std::{fs, process::Command};

use fs_extra::dir::CopyOptions;
use libloading::{Library, Symbol};
use proc_macro2::Span;
use scr_types::{game_state::GameState, RenderEntities, Systems};
use syn::parse_quote;
use toml::{Table, Value};

type InitSystemsFunc<'lib> = Symbol<'lib, fn() -> Systems>;
type RunSystemsFunc<'lib> = Symbol<'lib, fn(&Systems, &GameState)>;
type GetRenderEntitiesFunc<'lib> = Symbol<'lib, fn() -> RenderEntities>;
type AddBasicModelFunc<'lib> = Symbol<'lib, fn(uuid::Uuid)>;
type SaveWorldFunc<'lib> = Symbol<'lib, fn()>;
type LoadWorldFunc<'lib> = Symbol<'lib, fn()>;

fn get_component_names() -> Result<Vec<(String, String)>, std::io::Error> {
    let base_table = fs::read_to_string(".scr/Info.toml")?.parse::<Table>();

    // TODO!: better error handling
    let table = match base_table {
        Ok(t) => t,
        Err(_) => return Err(std::io::Error::from_raw_os_error(0)),
    };

    // TODO!: better error handling again
    // (if the user accidentally changes the name of the components table the whole engine crashes)
    let comps_value = table["Components"].clone();

    let components_map = match comps_value {
        Value::Table(t) => t,
        _ => panic!("something in the Info.toml file is seriously wrong"),
    };

    let mut names = vec![];

    for (k, v) in components_map {
        if let Value::String(s) = v {
            names.push((k, s));
        }
    }

    Ok(names)
}

fn get_internal_file(components: Vec<(String, String)>) -> syn::File {
    println!("{components:?}");
    let mut comps: Vec<syn::Ident> = vec![];
    let mut imports: Vec<syn::Path> = vec![];
    for (name, mut path) in components {
        comps.push(syn::Ident::new(&name, Span::call_site()));
        path.push_str("::");
        path.push_str(&name);
        imports.push(syn::parse_str(&path).unwrap());
    }
    parse_quote!(
        use scr_types::{game_state::GameState, Systems, RenderEntities};
        use scr_types::prelude::*;
        use scr_types::component::{ser::SerWorld, ser::SerializeCallback, de::DeWorldBuilder};

        #(use crate::#imports;)*

        static mut WORLD: Option<World> = None;

        #[no_mangle]
        pub fn save_world() {
            unsafe {
                if let Some(world) = &mut WORLD {
                    world.component_add_callback::<SerializeCallback, Transform>();
                    world.component_add_callback::<SerializeCallback, Rendering>();
                    world.component_add_callback::<SerializeCallback, Camera>();
                    world.component_add_callback::<SerializeCallback, Info>();
                    #(world.component_add_callback::<SerializeCallback, #comps>();)*
                    let serialized =
                        serde_json::to_string(&SerWorld::new(&world, uuid::Uuid::new_v4()))
                            .unwrap();
                    println!("{serialized:?}");
                    std::fs::write("/data/world.json", serialized).unwrap();
                }
            }
        }

        #[no_mangle]
        pub fn load_world() {
            unsafe {
                let serialized = std::fs::read_to_string("/data/world.json").unwrap();
                println!("{serialized}");
                let deworld = DeWorldBuilder::new()
                    .constructor::<Transform>()
                    .constructor::<Rendering>()
                    .constructor::<Camera>()
                    .constructor::<Info>()
                    #(.constructor::<#comps>())*
                    .build(&mut serde_json::Deserializer::from_str(&serialized))
                    .unwrap();
                WORLD = Some(deworld.world);
            }
        }

        #[no_mangle]
        pub fn setup() {
            unsafe {
                WORLD = Some(World::new());
            }
        }

        #[no_mangle]
        pub fn run_systems(systems: &Systems, game_state: &GameState) {
            unsafe {
                if WORLD.is_none() {
                    WORLD = Some(World::new());
                }
            }

            for sys in &systems.systems {
                let system = sys.0;
                unsafe {
                    system(&mut WORLD.as_mut().unwrap(), game_state);
                }
            }
        }

        #[no_mangle]
        pub fn get_render_entities() -> RenderEntities {
            unsafe {
                if let Some(world) = &mut WORLD {
                    let res = world.component_collect::<(Transform, Rendering)>();
                    RenderEntities { entities: res }
                } else {
                    RenderEntities { entities: vec![] }
                }
            }

        }

        #[no_mangle]
        pub fn add_basic_model(id: uuid::Uuid) {
            unsafe {
                if let Some(world) = &mut WORLD {
                    let e = world.entity_create();
                    world.entity_set(
                        e,
                        (
                            scr_types::prelude::Transform::default(),
                            scr_types::prelude::Rendering { model_id: id },
                        ),
                    );
                }
            }
        }
    )
}

pub trait ScriptsLib {
    fn init(&self) -> Systems;
    fn run(&self, systems: &Systems, game_state: &GameState);
    fn get_render_entities(&self) -> RenderEntities;
    fn add_model(&self, id: uuid::Uuid);
    fn save_world(&self);
    fn load_world(&self);
}

impl ScriptsLib for Library {
    fn init(&self) -> Systems {
        let init_fn: InitSystemsFunc = unsafe { self.get("init_systems".as_bytes()).unwrap() };
        init_fn()
    }
    fn run(&self, systems: &Systems, game_state: &GameState) {
        let run_fn: RunSystemsFunc = unsafe { self.get("run_systems".as_bytes()).unwrap() };
        run_fn(systems, game_state);
    }

    fn get_render_entities(&self) -> RenderEntities {
        let get_render_entities_fn: GetRenderEntitiesFunc =
            unsafe { self.get("get_render_entities".as_bytes()).unwrap() };

        get_render_entities_fn()
    }

    fn add_model(&self, id: uuid::Uuid) {
        println!("adding model {id}");
        let add_basic_model: AddBasicModelFunc =
            unsafe { self.get("add_basic_model".as_bytes()).unwrap() };

        add_basic_model(id);
    }

    fn save_world(&self) {
        println!("saving world");
        let save_world: SaveWorldFunc = unsafe { self.get("save_world".as_bytes()).unwrap() };

        save_world();
    }

    fn load_world(&self) {
        println!("loading world");
        let load_world: LoadWorldFunc = unsafe { self.get("load_world".as_bytes()).unwrap() };

        load_world();
    }
}

pub fn load_scripts_lib() -> std::io::Result<(Library, Systems)> {
    fs_extra::dir::copy(
        "scripts/",
        ".scr/",
        &CopyOptions::new().overwrite(true).content_only(true),
    )
    .unwrap();

    let components = get_component_names()?;

    std::fs::write(
        ".scr/src/internal.rs",
        prettyplease::unparse(&get_internal_file(components)),
    )
    .unwrap();

    let source_lib_rs = std::fs::read_to_string(".scr/src/lib.rs").unwrap();

    let mut fin_string = String::from("pub mod internal;");
    fin_string.push_str(&source_lib_rs);

    std::fs::write(".scr/src/lib.rs", fin_string).unwrap();

    Command::new("cargo")
        .args(["b", "--manifest-path", ".scr/Cargo.toml"])
        .status()
        .unwrap();

    let mut name = ".scr/target/debug/".to_owned();
    name.push_str(libloading::library_filename("scripts").to_str().unwrap());
    let scripts_lib = unsafe { Library::new(name).unwrap() };
    let systems = scripts_lib.init();
    Ok((scripts_lib, systems))
}

pub fn run_scripts(lib: &Library, systems: &Systems, game_state: &GameState) {
    lib.run(systems, game_state)
}

pub fn get_render_data(lib: &Library) -> RenderEntities {
    lib.get_render_entities()
}

pub fn add_basic_model(lib: &Library, id: uuid::Uuid) {
    lib.add_model(id)
}

pub fn save_world(lib: &Library) {
    lib.save_world();
}

pub fn load_world(lib: &Library) {
    lib.load_world();
}
