use std::{fs, io, any::type_name};

use serde::{Serialize, Deserialize};

const SCRIPTS_PATH: &'static str = "scripts/";
const INT_SCRIPTS_PATH: &'static str = ".scr/";

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    name: String,
    structure: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Script {
    functions: Vec<String>,
    components: Vec<Component>
}

pub struct Scripting {
    scripts: Vec<Script>,
}

impl Scripting {
    pub fn load_scripts(&mut self) -> io::Result<()> {
        for e in fs::read_dir(INT_SCRIPTS_PATH)? {
            let entry = e?;
            let n = entry.file_name();
            let name = n.to_str().unwrap();
            if name.ends_with(".scr") {
                // unwrap-justify: there are only files with scr that are readable
                let content = fs::read_to_string(entry.path()).unwrap();
                // unwrap-justify: files should only be written to by macro
                self.scripts.push(ron::from_str(&content).unwrap());
            }
        }

        println!("{:?}", self.scripts);

        Ok(())
    }
}

impl Default for Scripting {
    fn default() -> Self {
        Self {
            scripts: vec![]
        }
    }
}