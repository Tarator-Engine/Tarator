use std::path::Path;

use cgmath::SquareMatrix;
use serde::{Deserialize, Serialize};

use crate::{scene::ImportData, Error, Mat4, Quat, Result, Vec3};

use super::{store_material::StoreMaterial, store_mesh::StoreMesh, store_texture::StoreTexture};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreNode {
    pub index: usize,
    pub children: Vec<usize>,
    pub mesh: Option<usize>,
    pub rotation: Quat,
    pub scale: Vec3,
    pub translation: Vec3,
    // TODO: weights
    // weights_id: usize,
    pub name: String,
    // TODO: camera importing
    // pub camera: Option<Camera>,
    pub final_transform: Mat4,
    pub root_node: bool,
}

impl StoreNode {
    pub fn from_gltf(
        g_node: &gltf::Node<'_>,
        meshes: &mut Vec<StoreMesh>,
        materials: &mut Vec<StoreMaterial>,
        textures: &mut Vec<StoreTexture>,
        imp: &ImportData,
        base_path: &Path,
        name: &str,
        root_node: bool,
    ) -> Result<Self> {
        let (trans, rot, scale) = g_node.transform().decomposed();
        let r = rot;
        let rotation = Quat::new(r[3], r[0], r[1], r[2]);

        let name: String = name.into();

        let mut mesh = None;
        if let Some(g_mesh) = g_node.mesh() {
            if let Some(m) = meshes.iter().find(|mesh| (**mesh).index == g_mesh.index()) {
                mesh = Some(m.index);
            }

            if mesh.is_none() {
                let mesh_name = g_mesh
                    .name()
                    .map(|s| s.into())
                    .unwrap_or(name.clone() + "mesh");

                meshes.push(StoreMesh::from_gltf(
                    &g_mesh, imp, base_path, materials, textures, &name, &mesh_name,
                )?);
                mesh = Some(meshes.len() - 1);
            }
        }

        let children = g_node.children().map(|g_node| g_node.index()).collect();

        Ok(Self {
            index: g_node.index(),
            children,
            mesh,
            rotation,
            scale: scale.into(),
            translation: trans.into(),
            name,
            final_transform: Mat4::identity(),
            root_node,
        })
    }
}
