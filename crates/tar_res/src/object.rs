use std::collections::HashMap;

use tar_types::camera::CameraParams;

use crate::{material::PerFrameData, mesh::StaticMesh, node::Node};

pub struct Object {
    pub nodes: Vec<Node>,
}

impl<'a> Object {
    pub fn update_per_frame(
        &mut self,
        cam_params: &CameraParams,
        data: &PerFrameData,
        queue: &wgpu::Queue,
        meshes: &HashMap<uuid::Uuid, StaticMesh>,
    ) {
        for node in &mut self.nodes {
            node.update_per_frame(cam_params, data, queue, meshes);
        }
    }

    pub fn draw<'b: 'a>(
        &'b self,
        render_pass: &mut wgpu::RenderPass<'a>,
        meshes: &'a HashMap<uuid::Uuid, StaticMesh>,
    ) {
        for node in &self.nodes {
            node.draw(render_pass, meshes);
        }
    }
}
