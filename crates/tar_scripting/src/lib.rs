use std::{fs, io, process::Command};

use tar_types::script::{Component, FileContent, System};

const MANIFEST_PATH: &'static str = "scripts/Cargo.toml";
const INT_SCRIPTS_PATH: &'static str = ".scr/";

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
}
