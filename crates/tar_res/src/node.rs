use std::{path::Path, sync::Arc};

use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use serde::{Deserialize, Serialize};

use crate::{mesh::Mesh, scene::ImportData, CameraParams, Error, Result, WgpuInfo};

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
    pub fn new(
        index: usize,
        children: Vec<Node>,
        mesh: Option<Mesh>,
        rotation: Quaternion<f32>,
        scale: Vector3<f32>,
        translation: Vector3<f32>,
        name: String,
        final_transform: Matrix4<f32>,
    ) -> Self {
        Self {
            index,
            children,
            mesh,
            rotation,
            scale,
            translation,
            name,
            final_transform,
        }
    }

    pub fn update_transform(&mut self, parent_transform: &Matrix4<f32>) -> Result<()> {
        self.final_transform = *parent_transform;

        // TODO: cache local tranform when adding animations?
        self.final_transform = self.final_transform
            * Matrix4::from_translation(self.translation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
            * Matrix4::from(self.rotation);

        for node in &mut self.children {
            node.update_transform(&self.final_transform);
        }

        Ok(())
    }

    pub fn draw<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        cam_params: &CameraParams,
    ) {
        if let Some(ref m_id) = self.mesh {
            let mvp_matrix =
                cam_params.projection_matrix * cam_params.view_matrix * self.final_transform;

            if let Some(mesh) = &self.mesh {
                mesh.draw(
                    render_pass,
                    &self.final_transform,
                    &mvp_matrix,
                    &cam_params.position,
                );
            }
        }
        for node in &self.children {
            node.draw(render_pass, cam_params);
        }
    }
}
