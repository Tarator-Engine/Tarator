use winit::{dpi::PhysicalSize, window::Window};

use crate::GameObject;
use async_trait::async_trait;

pub mod deferred;
pub mod forward;

#[async_trait]
pub trait Renderer<'a> {
    async fn new(window: &Window) -> Self;
    fn resize(&mut self, new_size: PhysicalSize<u32>);
    fn select_camera(&mut self, cam: u32);
    async fn add_object(&mut self, obj: GameObject<'a>) -> tar_res::Result<()>;
    fn render(&mut self) -> Result<(), wgpu::SurfaceError>;
}
