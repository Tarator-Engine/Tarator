use std::{path::Path, sync::Arc};

use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use serde::{Deserialize, Serialize};

use crate::{mesh::Mesh, root::Root, scene::ImportData, CameraParams, Error, Result, WgpuInfo};

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub index: usize,
    pub children: Vec<usize>,
    pub mesh: Option<usize>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    // TODO: weights
    // weights_id: usize,
    pub translation: Vector3<f32>,
    // TODO: camera importing
    // pub camera: Option<Camera>,
    pub name: Option<String>,
    pub final_transform: Matrix4<f32>, // includes parent transforms
}

impl Node {
    pub fn new(
        index: usize,
        children: Vec<usize>,
        mesh: Option<usize>,
        rotation: Quaternion<f32>,
        scale: Vector3<f32>,
        translation: Vector3<f32>,
        name: Option<String>,
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

    pub fn from_gltf(
        g_node: &gltf::Node<'_>,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path,
        w_info: &WgpuInfo,
    ) -> Result<Self> {
        let (trans, rot, scale) = g_node.transform().decomposed();
        let r = rot;
        let rotation = Quaternion::new(r[3], r[0], r[1], r[2]);

        let mut mesh = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some(existing_mesh) = root
                .meshes
                .iter()
                .enumerate()
                .find(|(_, mesh)| (***mesh).index == g_mesh.index())
            {
                mesh = Some(existing_mesh.0);
            }

            if mesh.is_none() {
                let n_mesh = Arc::new(Mesh::from_gltf(&g_mesh, root, imp, base_path, w_info)?);
                root.meshes.push(n_mesh);
                mesh = Some(root.meshes.len() - 1);
            }
        }

        let children: Vec<_> = g_node.children().map(|g_node| g_node.index()).collect();

        Ok(Self::new(
            g_node.index(),
            children,
            mesh,
            rotation,
            scale.into(),
            trans.into(),
            g_node.name().map(|s| s.into()),
            Matrix4::identity(),
        ))
    }

    pub fn update_transform(
        &mut self,
        root: &mut Root,
        parent_transform: &Matrix4<f32>,
    ) -> Result<()> {
        self.final_transform = *parent_transform;

        // TODO: cache local tranform when adding animations?
        self.final_transform = self.final_transform
            * Matrix4::from_translation(self.translation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
            * Matrix4::from(self.rotation);

        for node_id in &self.children {
            let node = root.get_node_mut(*node_id).ok_or(Error::NonExistentID)?;
            if let Ok(mut node) = node.lock() {
                node.update_transform(root, &self.final_transform);
            } else {
                return Err(Error::LockFailed);
            };
        }

        Ok(())
    }

    pub fn draw(
        &self,
        render_pass: &mut wgpu::RenderPass,
        root: &mut Root,
        cam_params: &CameraParams,
    ) -> Result<()> {
        if let Some(ref m_id) = self.mesh {
            let mvp_matrix =
                cam_params.projection_matrix * cam_params.view_matrix * self.final_transform;

            let mesh = root.get_mesh(*m_id).ok_or(Error::NonExistentID)?;

            mesh.draw(
                &root,
                render_pass,
                &self.final_transform,
                &mvp_matrix,
                &cam_params.position,
            );
        }
        for node_id in &self.children {
            let node = root.get_node_mut(*node_id).ok_or(Error::NonExistentID)?;
            if let Ok(node) = node.lock() {
                node.draw(render_pass, root, cam_params);
            } else {
                return Err(Error::LockFailed);
            };
        }
        Ok(())
    }
}
