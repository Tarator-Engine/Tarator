use std::{path::Path, sync::Arc};

use cgmath::{Matrix4, Vector3};
use wgpu::RenderPass;

use crate::{primitive::Primitive, scene::ImportData, Error, Result, WgpuInfo};

pub struct Mesh {
    pub index: usize,
    pub primitives: Vec<Primitive>,
    // TODO: weights
    // pub weights: Vec<?>
    pub name: String,
}

impl Mesh {
    pub fn draw<'a, 'b>(
        &'a self,
        render_pass: &'b mut RenderPass<'a>,
        model_matrix: &Matrix4<f32>,
        mvp_matrix: &Matrix4<f32>,
        camera_position: &Vector3<f32>,
    ) -> Result<()> {
        for primitive in &self.primitives {
            primitive.draw(render_pass, model_matrix, mvp_matrix, camera_position)?
        }
        Ok(())
    }
}
