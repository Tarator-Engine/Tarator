use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Frequency {
    Update,
    Startup,
    FixedUpdate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct System {
    pub name: String,
    pub frequency: Frequency,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    name: String,
    structure: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FileContent {
    System(System),
    Component(Component),
}
