use winit::{dpi::PhysicalSize, window::Window};

use crate::GameObject;

mod deferred;
mod foreward;

pub trait Renderer {
    async fn new(window: &Window) -> Self;
    fn resize(&mut self, new_size: PhysicalSize<u32>);
    fn select_camera(&mut self, cam: u32);
    fn add_object(&mut self, obj: GameObject<'static>) -> tar_res::Result<()>;
    fn render(&mut self, out_view: wgpu::TextureView);
}
