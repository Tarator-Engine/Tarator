//! This crate is used for loading and accessing compiled scripts (via the compiled scr_manager crate)

use std::process::Command;

use libloading::{Library, Symbol};
use scr_types::Systems;

type InitSystemsFunc<'lib> = Symbol<'lib, fn() -> Systems>;
type RunSystemsFunc<'lib> = Symbol<'lib, fn(&Systems)>;

pub fn load_scripts() {
    Command::new("cargo")
        .args(["b", "--manifest-path", "scripts/Cargo.toml"])
        .env("CARGO_TARGET_DIR", ".scr/")
        .status()
        .unwrap();

    let mut name = ".scr/debug/".to_owned();
    name.push_str(libloading::library_filename("scripts").to_str().unwrap());
    let scripts_lib = unsafe { Library::new(name).unwrap() };

    let init_fn: InitSystemsFunc = unsafe { scripts_lib.get("init_systems".as_bytes()).unwrap() };
    let systems = init_fn();

    let run_fn: RunSystemsFunc = unsafe { scripts_lib.get("run_systems".as_bytes()).unwrap() };

    run_fn(&systems);
}
