use wgpu::RenderPass;

use crate::{material::PerFrameData, primitive::Primitive};

pub struct Mesh {
    pub index: usize,
    pub primitives: Vec<Primitive>,
    // TODO: weights
    // pub weights: Vec<?>
    pub name: String,
}

impl Mesh {
    pub fn update_per_frame(&mut self, data: &PerFrameData, queue: &wgpu::Queue) {
        for prim in &mut self.primitives {
            prim.update_per_frame(data, queue)
        }
    }

    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut RenderPass<'a>) {
        for primitive in &self.primitives {
            primitive.draw(render_pass);
        }
    }
}
