pub struct AudioManager {}

impl AudioManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_file(&mut self, _file: &str) -> uuid::Uuid {
        todo!("file loading")
    }
}
