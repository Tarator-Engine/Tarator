use crate::{node::Node, CameraParams};

pub struct Object {
    pub nodes: Vec<Node>,
}

impl Object {
    pub fn draw<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        cam_params: &CameraParams,
    ) {
        for node in &self.nodes {
            node.draw(render_pass, cam_params);
        }
    }
}
