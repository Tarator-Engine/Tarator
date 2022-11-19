use std::path::Path;

use crate::{primitive::Primitive, root::Root, scene::ImportData, Result, WgpuInfo};

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
        let mut primitives: Vec<Primitive> = vec![];
        for (i, g_prim) in g_mesh.primitives().enumerate(){
            primitives.push(Primitive::from_gltf(&g_prim, i, g_mesh.index(), root, imp, base_path, w_info)?);
        }
                   

        Ok(Mesh { 
            index: g_mesh.index(), 
            primitives, 
            name: g_mesh.name().map(|s| s.into())
        })
    }
}