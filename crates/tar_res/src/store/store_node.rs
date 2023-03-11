use std::path::Path;

use cgmath::SquareMatrix;
use serde::{Deserialize, Serialize};
use tar_types::prims::{Mat4, Quat, Vec3};

use crate::{scene::ImportData, Result};

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
        let timer = tar_utils::start_timer_msg("started loading store_node");
        let (trans, rot, scale) = g_node.transform().decomposed();
        let r = rot;
        let rotation = Quat::new(r[3], r[0], r[1], r[2]);

        let name: String = name.into();

        let mesh = if let Some(g_mesh) = g_node.mesh() {
            let m = if let Some(m) = meshes.iter().find(|mesh| (**mesh).index == g_mesh.index()) {
                Some(m.index)
            } else {
                let mesh_name = g_mesh
                    .name()
                    .map(|s| s.into())
                    .unwrap_or(name.clone() + "mesh");
                let index = g_mesh.index();

                meshes.push(StoreMesh::from_gltf(
                    &g_mesh, imp, base_path, materials, textures, &name, &mesh_name,
                )?);
                Some(index)
            };
            m
        } else {
            None
        };

        // println!("meshes: {meshes:?}");

        let children = g_node.children().map(|g_node| g_node.index()).collect();

        tar_utils::log_timing("generated node in ", timer);

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
