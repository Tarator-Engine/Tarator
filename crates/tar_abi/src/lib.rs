//! This crate is used for loading and accessing compiled scripts (via the compiled scr_manager crate)

use std::process::Command;

use libloading::{Library, Symbol};
use scr_types::Systems;

type InitSystemsFunc<'lib> = Symbol<'lib, fn() -> Systems>;

pub fn load_scripts() -> Systems {
    Command::new("cargo")
        .args(["b", "--manifest-path", "scripts/Cargo.toml"])
        .env("CARGO_TARGET_DIR", ".scr/")
        .status()
        .unwrap();

    let scripts_lib =
        unsafe { Library::new(libloading::library_filename(".scr/systems")).unwrap() };

    let init_fn: InitSystemsFunc = unsafe { scripts_lib.get("init_systems".as_bytes()).unwrap() };

    init_fn()
}
