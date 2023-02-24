use wgpu::RenderPass;

use crate::{material::PerFrameData, primitive::Primitive};

pub struct StaticMesh {
    pub index: usize,
    pub primitives: Vec<Primitive>,
    // TODO: weights
    // pub weights: Vec<?>
    pub name: String,
}

impl StaticMesh {
    pub fn update_per_frame(&self, data: &PerFrameData, queue: &wgpu::Queue) {
        for prim in &self.primitives {
            prim.update_per_frame(data, queue)
        }
    }

    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut RenderPass<'a>) {
        for primitive in &self.primitives {
            primitive.draw(render_pass);
        }
    }
}
