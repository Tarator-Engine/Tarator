use std::sync::{Arc, Mutex};

use std::path::Path;

use crate::primitive::Primitive;
use crate::{material::PbrMaterial, mesh::Mesh, node::Node, scene::ImportData, texture::Texture};
use crate::{Result, WgpuInfo};

#[derive(Default)]
pub struct Root {
    pub nodes: Vec<Arc<Mutex<Node>>>,
    pub meshes: Vec<Arc<Mesh>>,
    pub primitives: Vec<Arc<Primitive>>,
    pub materials: Vec<Arc<PbrMaterial>>,
    pub textures: Vec<Arc<Texture>>,
}
impl Root {
    pub fn from_gltf(imp: &ImportData, base_path: &Path, w_info: WgpuInfo) -> Result<Self> {
        let mut root = Root::default();
        let mut nodes = vec![];
        for g_node in imp.doc.nodes() {
            nodes.push(Arc::new(Mutex::new(Node::from_gltf(
                &g_node, &mut root, imp, base_path, &w_info,
            )?)));
        }
        root.nodes = nodes;
        Ok(root)
    }

    /// Get a mutable reference to a node
    pub fn get_node_mut(&self, index: usize) -> Option<Arc<Mutex<Node>>> {
        return self.nodes.get(index).and_then(option_clone);
    }

    pub fn get_mesh(&self, index: usize) -> Option<Arc<Mesh>> {
        return self.meshes.get(index).and_then(option_clone);
    }

    pub fn get_primitieve(&self, index: usize) -> Option<Arc<Primitive>> {
        return self.primitives.get(index).and_then(option_clone);
    }

    pub fn get_material(&self, index: usize) -> Option<Arc<PbrMaterial>> {
        return self.materials.get(index).and_then(option_clone);
    }

    pub fn get_texture(&self, index: usize) -> Option<Arc<Texture>> {
        return self.textures.get(index).and_then(option_clone);
    }
}

fn option_clone<T>(inp: &Arc<T>) -> Option<Arc<T>> {
    Some(inp.clone())
}
