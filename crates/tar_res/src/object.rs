use crate::{material::PerFrameData, node::Node, CameraParams};

pub struct Object {
    pub nodes: Vec<Node>,
}

impl Object {
    pub fn update_per_frame(
        &mut self,
        cam_params: &CameraParams,
        data: &PerFrameData,
        queue: &wgpu::Queue,
    ) {
        for node in &mut self.nodes {
            node.update_per_frame(cam_params, data, queue);
        }
    }

    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        for node in &self.nodes {
            node.draw(render_pass);
        }
    }
}
