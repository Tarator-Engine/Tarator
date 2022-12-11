use std::{path::Path, sync::Arc};

use cgmath::{Matrix4, Vector3};
use wgpu::RenderPass;

use crate::{primitive::Primitive, root::Root, scene::ImportData, Error, Result, WgpuInfo};

pub struct Mesh {
    pub index: usize,
    pub primitives: Vec<Primitive>,
    // TODO: weights
    // pub weights: Vec<?>
    pub name: Option<String>,
}

impl Mesh {
    pub fn from_gltf(
        g_mesh: &gltf::Mesh<'_>,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path,
        w_info: &WgpuInfo,
    ) -> Result<Mesh> {
        let mut primitives: Vec<usize> = vec![];
        for (i, g_prim) in g_mesh.primitives().enumerate() {
            let mesh = Arc::new(Primitive::from_gltf(
                &g_prim,
                i,
                g_mesh.index(),
                root,
                imp,
                base_path,
                w_info,
            )?);
            root.primitives.push(mesh);
            primitives.push(root.primitives.len() - 1);
        }

        Ok(Mesh {
            index: g_mesh.index(),
            primitives,
            name: g_mesh.name().map(|s| s.into()),
        })
    }

    pub fn draw(
        &self,
        root: &Root,
        render_pass: &mut RenderPass,
        model_matrix: &Matrix4<f32>,
        mvp_matrix: &Matrix4<f32>,
        camera_position: &Vector3<f32>,
    ) -> Result<()> {
        for primitive in &self.primitives {
            primitive.draw(root, render_pass, model_matrix, mvp_matrix, camera_position)?
        }
        Ok(())
    }
}
