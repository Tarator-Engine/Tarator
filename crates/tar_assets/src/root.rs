use std::{sync::Arc, collections::HashMap};

use std::path::Path;

use crate::{Result, WgpuInfo};
use crate::{node::Node, mesh::Mesh, texture::Texture, material::Material, shader::{PbrShader, ShaderFlags}, scene::ImportData};

#[derive(Default)]
pub struct Root {
    pub nodes: Vec<Node>,
    pub meshes: Vec<Arc<Mesh>>,
    pub textures: Vec<Arc<Texture>>,
    pub materials: Vec<Arc<Material>>,
    pub shaders: HashMap<ShaderFlags, Arc<PbrShader>>,

    // TODO: cameras
    // pub camera_nodes: Vec<usize>,
}
impl Root {
    pub fn from_gltf(imp: &ImportData, base_path: &Path, w_info: WgpuInfo) -> Result<Self> {
        let mut root = Root::default();
        let mut nodes = vec![];
        for g_node in imp.doc.nodes() {
            nodes.push(Node::from_gltf(&g_node, &mut root, imp, base_path, &w_info)?);
        }
        root.nodes = nodes;
        Ok(root)
    }

    /// Get a mutable reference to a node without borrowing `Self` or `Self::nodes`.
    /// Safe for tree traversal (visiting each node ONCE and NOT keeping a reference)
    /// as long as the gltf is valid, i.e. the scene actually is a tree.
    pub fn unsafe_get_node_mut(&mut self, index: usize) ->&'static mut Node {
        unsafe {
            &mut *(&mut self.nodes[index] as *mut Node)
        }
    }
}