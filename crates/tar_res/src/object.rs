use crate::{node::Node, CameraParams};

pub struct Object {
    pub nodes: Vec<Node>,
}

impl Object {
    pub fn update_per_frame(
        &mut self,
        cam_params: &CameraParams,
        u_light_direction: [f32; 3],
        u_light_color: [f32; 3],
        u_ambient_light_color: [f32; 3],
        u_ambient_light_intensity: f32,
        u_alpha_blend: f32,
        u_alpha_cutoff: f32,
        queue: &wgpu::Queue,
    ) {
        for node in &mut self.nodes {
            node.update_per_frame(
                cam_params,
                u_light_direction,
                u_light_color,
                u_ambient_light_color,
                u_ambient_light_intensity,
                u_alpha_blend,
                u_alpha_cutoff,
                queue,
            );
        }
    }

    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        for node in &self.nodes {
            node.draw(render_pass);
        }
    }
}
