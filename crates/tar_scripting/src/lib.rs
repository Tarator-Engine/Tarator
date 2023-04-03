use std::{fs, io, process::Command};

use libc::c_void;
use libloading::{Library, Symbol};
use tar_ecs::{component::ComponentHashId, world::World};
use tar_types::{
    components::Transform,
    script::{Component, FileContent, System},
};

#[rustfmt::skip]
enum SystemSym<'lib> {
    Sys1(Symbol<'lib, fn (*mut c_void)>),
    Sys2(Symbol<'lib, fn (*mut c_void, *mut c_void, )>),
    Sys3(Symbol<'lib, fn (*mut c_void, *mut c_void, *mut c_void, )>),
    Sys4(Symbol<'lib, fn (*mut c_void, *mut c_void, *mut c_void, *mut c_void, )>),
    Sys5(Symbol<'lib, fn (*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void, )>),
    Sys6(Symbol<'lib, fn (*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void, )>),
    Sys7(Symbol<'lib, fn (*mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void, *mut c_void, )>),
}

const MANIFEST_PATH: &str = "scripts/Cargo.toml";
const INT_SCRIPTS_PATH: &str = ".scr/";

#[derive(Debug, Default)]
pub struct Scripting {
    systems: Vec<System>,
    components: Vec<Component>,
}

impl Scripting {
    pub fn load_scripts(&mut self) -> io::Result<()> {
        Command::new("cargo")
            .args(["b", "--manifest-path", MANIFEST_PATH])
            .env("CARGO_TARGET_DIR", ".scr/")
            .status()
            .unwrap();
        for e in fs::read_dir(INT_SCRIPTS_PATH)? {
            let entry = e?;
            let n = entry.file_name();
            let name = n.to_str().unwrap();
            if name.ends_with(".scr") {
                // unwrap-justify: there are only files with scr that are readable
                let content = fs::read_to_string(entry.path()).unwrap();
                // unwrap-justify: files should only be written to by macro
                let file: FileContent = ron::from_str(&content).unwrap();
                match file {
                    FileContent::System(s) => self.systems.push(s),
                    FileContent::Component(c) => self.components.push(c),
                }
            }
        }

        println!("{:?}", self.systems);

        Ok(())
    }

    pub fn test_scripting(&mut self) {
        // unwrap-justify: the file should be available and correct since this is a testing function
        let lib = unsafe { libloading::Library::new(".scr/debug/scripts.dll").unwrap() };

        let mut world = tar_ecs::world::World::new();

        let e1 = world.entity_create();
        let e2 = world.entity_create();

        world.entity_set(e1, Transform::default());
        world.entity_set(e2, Transform::default());

        run_system(&self.systems[0], &mut world, &lib);

        let transforms = world.component_collect::<Transform>();

        for t in transforms {
            println!("{t:?}");
        }
    }
}

fn run_system(sys: &System, world: &mut World, lib: &Library) {
    let sym = match sys.inputs.len() {
        1 => SystemSym::Sys1(unsafe { lib.get(sys.name.as_bytes()).unwrap() }),
        2 => SystemSym::Sys2(unsafe { lib.get(sys.name.as_bytes()).unwrap() }),
        3 => SystemSym::Sys3(unsafe { lib.get(sys.name.as_bytes()).unwrap() }),
        4 => SystemSym::Sys4(unsafe { lib.get(sys.name.as_bytes()).unwrap() }),
        5 => SystemSym::Sys5(unsafe { lib.get(sys.name.as_bytes()).unwrap() }),
        6 => SystemSym::Sys6(unsafe { lib.get(sys.name.as_bytes()).unwrap() }),
        7 => SystemSym::Sys7(unsafe { lib.get(sys.name.as_bytes()).unwrap() }),

        _ => panic!("only 1-7 inputs are supported"),
    };

    let tmp_inputs = vec![
        "tar_types::components::Transform",
        // "tar_types::components::Rendering",
    ];

    let mut fin_tables: Vec<Vec<*const u8>> = vec![];

    for input in tmp_inputs {
        let tables = world.component_query_tables(input);

        let inp = tables
            .iter()
            .map(|t| {
                let table = t.read();

                let mut transforms = vec![];
                for i in 0..table.len() {
                    let transform = unsafe {
                        table
                            .get_unchecked_raw(ComponentHashId::new_from_str(input), i)
                            .unwrap()
                    };
                    transforms.push(transform);
                }
                transforms
            })
            .collect::<Vec<_>>();

        let t = inp[0].clone();
        fin_tables.push(t);
    }

    // func(unsafe { std::mem::transmute(&mut t) });

    match sym {
        SystemSym::Sys1(s) => s(unsafe { std::mem::transmute(&mut fin_tables[0]) }),
        SystemSym::Sys2(s) => s(unsafe { std::mem::transmute(&mut fin_tables[0]) }, unsafe {
            std::mem::transmute(&mut fin_tables[1])
        }),

        _ => todo!("support a larger range of inputs"),
    }
}
