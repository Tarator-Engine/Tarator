use std::{path::Path, sync::Arc};

use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};

use crate::{mesh::Mesh, object::ImportData, CameraParams, Result, WgpuInfo};

pub struct Node {
    pub index: usize,
    pub children: Vec<Node>,
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
    pub fn from_gltf(
        g_node: &gltf::Node<'_>,
        nodes: &mut Node,
        meshes: &mut Vec<Mesh>,
        imp: &ImportData,
        base_path: &Path,
        w_info: &WgpuInfo,
    ) -> Result<Self> {
        let (trans, rot, scale) = g_node.transform().decomposed();
        let r = rot;
        let rotation = Quaternion::new(r[3], r[0], r[1], r[2]);

        let mut mesh = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some((i, _)) = meshes
                .iter()
                .enumerate()
                .find(|(_, mesh)| (**mesh).index == g_mesh.index())
            {
                mesh = Some(i);
            }

            if mesh.is_none() {
                meshes.push(Mesh::from_gltf(&g_mesh, imp, base_path, w_info)?);
                mesh = Some(meshes.len() - 1);
            }
        }

        let children: Vec<_> = g_node.children().map(|g_node| g_node.index()).collect();

        Ok(Node {
            index: g_node.index(),
            children,
            mesh,
            rotation,
            scale: scale.into(),
            translation: trans.into(),
            name: g_node.name().map(|s| s.into()),
            final_transform: Matrix4::identity(),
        })
    }

    pub fn update_transform(&mut self, nodes: &mut Node, parent_transform: &Matrix4<f32>) {
        self.final_transform = *parent_transform;

        // TODO: cache local tranform when adding animations?
        self.final_transform = self.final_transform
            * Matrix4::from_translation(self.translation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
            * Matrix4::from(self.rotation);

        for node_id in &self.children {
            let node = root.unsafe_get_node_mut(*node_id);
            node.update_transform(root, &self.final_transform);
        }
    }

    pub fn draw(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        root: &mut Root,
        cam_params: &CameraParams,
    ) {
        if let Some(ref mesh) = self.mesh {
            let mvp_matrix =
                cam_params.projection_matrix * cam_params.view_matrix * self.final_transform;

            (*mesh).draw(
                render_pass,
                &self.final_transform,
                &mvp_matrix,
                &cam_params.position,
            );
        }
        for node_id in &self.children {
            let node = root.unsafe_get_node_mut(*node_id);
            node.draw(render_pass, root, cam_params);
        }
    }
}
