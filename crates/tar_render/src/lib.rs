pub mod camera;
pub mod render;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// the GameObject is a enum which is used to pass something to a
/// Renderer's add_object()
pub enum GameObject<'a> {
    Object(tar_res::object::Object),
    ModelPath(&'a str, &'a str),
    ImportedPath(&'a str),
    Camera(camera::Camera),
}
