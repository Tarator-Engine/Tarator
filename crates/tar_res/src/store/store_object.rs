use std::{path::Path, vec};

use tar_utils::*;

use serde::{Deserialize, Serialize};

use crate::{scene::ImportData, Error, Result};

use super::{
    store_material::StoreMaterial, store_mesh::StoreMesh, store_node::StoreNode,
    store_texture::StoreTexture,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreObject {
    pub nodes: Vec<StoreNode>,
    pub meshes: Vec<StoreMesh>,
    pub materials: Vec<StoreMaterial>,
    pub textures: Vec<StoreTexture>,
    // TODO!: cameras
    // pub camera_nodes: Vec<usize>,
}

impl StoreObject {
    pub fn from_gltf(source: &str, object_name: &str) -> Result<Self> {
        let start_time = start_timer();
        println!("started importing {source:?}");
        if source.starts_with("http") {
            // TODO: implement http(s) loading
            return Err(Error::NotSupported("http loading".to_owned()));
        }

        let (doc, buffers, images) = gltf::import(source)?;
        let imp = ImportData {
            doc,
            buffers,
            images,
        };

        let start_time = relog_timing("Loaded glTF in ", start_time);

        let base_path = Path::new(source);

        let mut nodes = vec![];
        let mut meshes = vec![];
        let mut materials = vec![];
        let mut textures = vec![];
        for (i, g_node) in imp.doc.nodes().enumerate() {
            let node = StoreNode::from_gltf(
                &g_node,
                &mut meshes,
                &mut materials,
                &mut textures,
                &imp,
                base_path,
                format!("{object_name}-node-{i}").as_str(),
                false,
            )?;
            nodes.push(node);
        }
        let mut children: Vec<usize> = vec![];
        for node in &mut nodes {
            children.append(&mut node.children.clone())
        }
        for node in &mut nodes {
            if !children.contains(&node.index) {
                node.root_node = true;
            }
        }

        log_timing(
            &format!(
                "Loaded {} nodes, {} meshes in ",
                imp.doc.nodes().count(),
                imp.doc.meshes().len()
            ),
            start_time,
        );

        Ok(Self {
            nodes,
            meshes,
            materials,
            textures,
        })
    }
}
