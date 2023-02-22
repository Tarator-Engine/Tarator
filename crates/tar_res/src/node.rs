use cgmath::{Matrix4, Quaternion, Vector3};
use tar_types::camera::CameraParams;

use crate::{material::PerFrameData, mesh::Mesh, Result};

pub struct Node {
    pub index: usize,
    pub children: Vec<Node>,
    pub mesh: Option<Mesh>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    // TODO: weights
    // weights_id: usize,
    pub translation: Vector3<f32>,
    // TODO: camera importing
    // pub camera: Option<Camera>,
    pub name: String,
    pub final_transform: Matrix4<f32>, // includes parent transforms
}

impl Node {
    pub fn update_transform(&mut self, parent_transform: &Matrix4<f32>) -> Result<()> {
        self.final_transform = *parent_transform;

        // TODO: cache local tranform when adding animations?
        self.final_transform = self.final_transform
            * Matrix4::from_translation(self.translation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
            * Matrix4::from(self.rotation);

        for node in &mut self.children {
            node.update_transform(&self.final_transform)?;
        }

        Ok(())
    }
    pub fn update_per_frame(
        &mut self,
        cam_params: &CameraParams,
        data: &PerFrameData,
        queue: &wgpu::Queue,
    ) {
        if let Some(mesh) = &mut self.mesh {
            let mut data = (*data).clone();
            let mvp_matrix =
                cam_params.projection_matrix * cam_params.view_matrix * self.final_transform;
            data.u_model_matrix = self.final_transform.into();
            data.u_mpv_matrix = mvp_matrix.into();
            data.u_camera = cam_params.position.into();
            mesh.update_per_frame(&data, queue);
        }
        for child in &mut self.children {
            child.update_per_frame(cam_params, data, queue);
        }
    }

    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        if let Some(mesh) = &self.mesh {
            mesh.draw(render_pass);
        }
        for child in &self.children {
            child.draw(render_pass);
        }
    }
}
