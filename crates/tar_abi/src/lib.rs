//! This crate is used for loading and accessing compiled scripts (via the compiled scripts internal.rs)

use std::process::Command;

use fs_extra::dir::CopyOptions;
use libloading::{Library, Symbol};
use scr_types::Systems;

type InitSystemsFunc<'lib> = Symbol<'lib, fn() -> Systems>;
type RunSystemsFunc<'lib> = Symbol<'lib, fn(&Systems)>;

const INTERNAL_RS_FILE: &str = "
use scr_types::Systems;

static mut WORLD: Option<tar_ecs::prelude::World> = None;

#[no_mangle]
pub fn run_systems(systems: &Systems) {
    unsafe {
        if WORLD.is_none() {
            WORLD = Some(tar_ecs::prelude::World::new());
        }
    }

    for sys in &systems.systems {
        let system = sys.0;
        unsafe {
            system(&mut WORLD.as_mut().unwrap());
        }
    }
}
";

pub trait ScriptsLib {
    fn init(&self) -> Systems;
    fn run(&self, systems: &Systems);
}

impl ScriptsLib for Library {
    fn init(&self) -> Systems {
        let init_fn: InitSystemsFunc = unsafe { self.get("init_systems".as_bytes()).unwrap() };
        init_fn()
    }
    fn run(&self, systems: &Systems) {
        let run_fn: RunSystemsFunc = unsafe { self.get("run_systems".as_bytes()).unwrap() };
        run_fn(systems);
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

pub fn run_scripts(lib: &Library, systems: &Systems) {
    lib.run(systems);
}
