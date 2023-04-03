use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{scene::ImportData, Result};

use super::{
    store_material::StoreMaterial, store_primitive::StorePrimitive, store_texture::StoreTexture,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMesh {
    pub index: usize,
    pub primitives: Vec<StorePrimitive>,
    // TODO!: weights
    // pub weights: Vec<?>
    pub name: String,
}

impl StoreMesh {
    pub fn from_gltf(
        g_mesh: &gltf::Mesh<'_>,
        imp: &ImportData,
        base_path: &Path,
        materials: &mut Vec<StoreMaterial>,
        textures: &mut Vec<StoreTexture>,
        object_name: &String,
        mesh_name: &String,
    ) -> Result<Self> {
        let name = g_mesh.name().map_or(mesh_name.clone(), std::convert::Into::into);

        let mut primitives: Vec<StorePrimitive> = vec![];
        for (i, g_prim) in g_mesh.primitives().enumerate() {
            primitives.push(StorePrimitive::from_gltf(
                &g_prim,
                i,
                g_mesh.index(),
                imp,
                base_path,
                materials,
                textures,
                object_name,
            )?);
        }

        Ok(Self {
            index: g_mesh.index(),
            primitives,
            name,
        })
    }
}
