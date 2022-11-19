use std::{sync::Arc, path::Path};

use cgmath::{Quaternion, Vector3, Matrix4, SquareMatrix};

use crate::{mesh::Mesh, scene::ImportData, WgpuInfo, root::Root, Result};

pub struct Node {
    pub index: usize,
    pub children: Vec<usize>,
    pub mesh: Option<Arc<Mesh>>,
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
            if let Some(existing_mesh) = root.meshes.iter().find(|mesh| (***mesh).index == g_mesh.index()) {
                mesh = Some(Arc::clone(existing_mesh));
            }

            if mesh.is_none() {
                mesh = Some(Arc::new(Mesh::from_gltf(&g_mesh, root, imp, base_path, w_info)?));
                root.meshes.push(mesh.clone().unwrap());
            }
        }

        let children: Vec<_> = g_node.children()
            .map(|g_node| g_node.index())
            .collect();

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

    pub fn update_transform(&mut self, root: &mut Root, parent_transform: &Matrix4<f32>) {
        self.final_transform = *parent_transform;

        // TODO: cache local tranform when adding animations?
        self.final_transform = self.final_transform *
            Matrix4::from_translation(self.translation) *
            Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
            Matrix4::from(self.rotation);

        for node_id in &self.children {
            let node = root.unsafe_get_node_mut(*node_id);
            node.update_transform(root, &self.final_transform);
        }
    }

}