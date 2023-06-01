//! This crate is used for loading and accessing compiled scripts (via the compiled scripts internal.rs)

use std::process::Command;

use fs_extra::dir::CopyOptions;
use libloading::{Library, Symbol};
use scr_types::{game_state::GameState, RenderEntities, Systems};

type InitSystemsFunc<'lib> = Symbol<'lib, fn() -> Systems>;
type RunSystemsFunc<'lib> = Symbol<'lib, fn(&Systems, &GameState)>;
type GetRenderEntitiesFunc<'lib> = Symbol<'lib, fn() -> RenderEntities>;
type AddBasicModelFunc<'lib> = Symbol<'lib, fn(uuid::Uuid)>;

const INTERNAL_RS_FILE: &str = "
use scr_types::{game_state::GameState, Systems, RenderEntities};
use scr_types::prelude::*;
use tar_ecs::prelude::*;

static mut WORLD: Option<tar_ecs::prelude::World> = None;

#[no_mangle]
pub fn run_systems(systems: &Systems, game_state: &GameState) {
    unsafe {
        if WORLD.is_none() {
            WORLD = Some(tar_ecs::prelude::World::new());
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
            let mut res = vec![];
            world.component_query::<(Transform, Rendering)>().for_each(|e| {
                res.push((e.0.clone(), e.1.clone()));
            });
            return RenderEntities {
                entities: res,
            }
        }
    }

    RenderEntities {
        entities: vec![]
    }
}

#[no_mangle]
pub fn add_basic_model(id: uuid::Uuid) {
    println!(\"adding moddel\");
    unsafe {
        if let Some(world) = &mut WORLD {
            let e = world.entity_create();
            world.entity_set(e, (scr_types::prelude::Transform::default(), scr_types::prelude::Rendering{model_id: id}));
        }
    }
}
";

pub trait ScriptsLib {
    fn init(&self) -> Systems;
    fn run(&self, systems: &Systems, game_state: &GameState);
    fn get_render_entities(&self) -> RenderEntities;
    fn add_model(&self, id: uuid::Uuid);
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
}

pub fn load_scripts_lib() -> std::io::Result<(Library, Systems)> {
    fs_extra::dir::copy(
        "scripts/",
        ".scr/",
        &CopyOptions::new().overwrite(true).content_only(true),
    )
    .unwrap();

    std::fs::write(".scr/src/internal.rs", INTERNAL_RS_FILE).unwrap();

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
