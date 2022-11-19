use std::{sync::Arc, collections::HashMap};

use gltf::Mesh;

use crate::node::Node;

pub struct Root {
    pub nodes: Vec<Node>,
    pub meshes: Vec<Arc<Mesh>>,
    pub textures: Vec<Arc<Texture>>,
    pub materials: Vec<Arc<Material>>,
    pub shaders: HashMap<ShaderFlags, Arc<PbrShader>>,

    // TODO: cameras
    // pub camera_nodes: Vec<usize>,
}

