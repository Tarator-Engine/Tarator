use std::path::Path;

use tar_utils::*;

use crate::{mesh::Mesh, node::Node, root::Root, Error, Quat, Result, Vec1, Vec3, WgpuInfo};

pub struct ImportData {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

pub struct Object {
    nodes: Vec<Node>,
    meshes: Vec<Mesh>,
    position: Vec3,
    rotation: Quat,
    //TODO!: other types eg: camera, light etc.
}

impl Object {
    fn new(nodes: Vec<Node>, meshes: Vec<Mesh>, position: Vec3, rotation: Quat) -> Self {
        Self {
            nodes,
            meshes,
            position,
            rotation,
        }
    }

    fn from_gltf(path: &str, w_info: &WgpuInfo) -> Result<Self> {
        let start_time = start_timer();
        println!("started importing {path:?}");
        if path.starts_with("http") {
            // TODO: implement http(s) loading
            return Err(Error::NotSupported("http loading".to_owned()));
        }

        let (doc, buffers, images) = gltf::import(path)?;
        let imp = ImportData {
            doc,
            buffers,
            images,
        };

        let start_time = relog_timing("Imported glTF in ", start_time);

        let base_path = Path::new(path);
        let mut nodes = vec![];
        let mut meshes = vec![];
        let root_transform = cgmath::Matrix4::identity();
        for g_node in imp.doc.nodes() {
            nodes.push(Node::from_gltf(
                &g_node,
                &mut meshes,
                &imp,
                base_path,
                &w_info,
            )?);
            nodes[nodes.len() - 1].update_transform(&root_transform)
        }

        log_timing(
            &format!(
                "Loaded {} nodes, {} meshes in ",
                imp.doc.nodes().count(),
                imp.doc.meshes().len()
            ),
            start_time,
        );

        Ok(Self::new(nodes, meshes, position, rotation))
    }
}
