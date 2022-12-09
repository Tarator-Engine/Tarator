use gltf;
use std::path::Path;
use crate::{
    Result, 
    Error,
    root::Root,
    node::Node, WgpuInfo, CameraParams,
};
use tar_utils::*;
use cgmath::SquareMatrix;

pub struct ImportData {
    pub doc: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

pub struct Scene {
    pub root: Root,
    pub nodes: Vec<usize>,
}

impl Scene {
    pub fn new(source: &str, w_info: WgpuInfo) -> Result<Self> {
        let start_time = start_timer();
        if source.starts_with("http") {
            // TODO: implement http(s) loading
            return Err(Error::NotSupported("http loading".to_owned()))
        }

        let (doc, buffers, images) = gltf::import(source)?;
        let imp = ImportData {doc, buffers, images};

        let start_time = relog_timing("Imported glTF in ", start_time);

        let base_path = Path::new(source);
        let mut root = Root::from_gltf(&imp, base_path, w_info)?;
        let nodes = Self::nodes_from_gltf(imp.doc.scenes(), &mut root);

        log_timing(&format!("Loaded {} nodes, {} meshes in ",
            imp.doc.nodes().count(), imp.doc.meshes().len()), start_time);

        Ok(Self {
            root,
            nodes
        })
    }

    pub fn nodes_from_gltf(g_scenes: gltf::iter::Scenes, root: &mut Root) -> Vec<usize> {
        let mut nodes = vec![];

        for scene in g_scenes {
            let mut ns = scene.nodes()
                .map(|g_node| g_node.index())
                .collect();
            nodes.append(&mut ns);

            let root_transform = cgmath::Matrix4::identity();
            for node_id in &nodes {
                let node = root.unsafe_get_node_mut(*node_id);
                node.update_transform(root, &root_transform);
            }
        }

        nodes
    }

    // TODO: flatten the call hirarchy (global Vec<Primitives>)
    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass, root: &mut Root, cam_params: &CameraParams) {
        // TODO!: for correct alpha blending, sort by material alpha mode and
        // render opaque objects first.
        for node_id in &self.nodes {
            let node = root.unsafe_get_node_mut(*node_id);
            node.draw(render_pass, root, cam_params);
        }
    }
}