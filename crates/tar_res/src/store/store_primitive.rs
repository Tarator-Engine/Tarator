use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{scene::ImportData, shader::ShaderFlags, vertex::Vertex, Error, Result};

use super::{store_material::StoreMaterial, store_texture::StoreTexture};

#[derive(Debug, Serialize, Deserialize)]
pub struct StorePrimitive {
    pub vertices: Vec<Vertex>,
    pub indices: Option<Vec<u32>>,
    pub material: usize,
}

impl StorePrimitive {
    pub fn from_gltf(
        g_primitive: &gltf::Primitive<'_>,
        primitive_index: usize,
        mesh_index: usize,
        imp: &ImportData,
        base_path: &Path,
        materials: &mut Vec<StoreMaterial>,
        textures: &mut Vec<StoreTexture>,
        object_name: &String,
    ) -> Result<Self> {
        let buffers = &imp.buffers;
        let reader = g_primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let positions = reader
            .read_positions()
            .ok_or(Error::NoPositions)?
            .collect::<Vec<_>>();

        let mut vertices: Vec<Vertex> = positions
            .into_iter()
            .map(|position| Vertex {
                position,
                ..Vertex::default()
            })
            .collect();

        let mut shader_flags = ShaderFlags::empty();

        if let Some(normals) = reader.read_normals() {
            for (i, normal) in normals.enumerate() {
                vertices[i].normal = normal;
            }
            shader_flags |= ShaderFlags::HAS_NORMALS;
        } else {
            return Err(Error::NotSupported("normal calculation".to_owned()));
        }

        if let Some(tangents) = reader.read_tangents() {
            for (i, tangent) in tangents.enumerate() {
                vertices[i].tangent = tangent;
            }
            shader_flags |= ShaderFlags::HAS_TANGENTS;
        }

        let mut tex_coord_set = 0;
        while let Some(tex_coords) = reader.read_tex_coords(tex_coord_set) {
            if tex_coord_set > 1 {
                println!(
                    "Ignoring texture coordinate set {}, \
                    only supporting 2 sets at the moment. (mesh: {}, primitive: {})",
                    tex_coord_set, mesh_index, primitive_index
                );
                tex_coord_set += 1;
                continue;
            }
            for (i, tex_coord) in tex_coords.into_f32().enumerate() {
                match tex_coord_set {
                    0 => vertices[i].tex_coord_0 = tex_coord,
                    1 => vertices[i].tex_coord_1 = tex_coord,
                    _ => unreachable!(),
                }
            }
            shader_flags |= ShaderFlags::HAS_UV;
            tex_coord_set += 1;
        }

        if let Some(colors) = reader.read_colors(0) {
            let colors = colors.into_rgba_f32();
            for (i, c) in colors.enumerate() {
                vertices[i].color_0 = c.into();
            }
            shader_flags |= ShaderFlags::HAS_COLORS;
        }

        if reader.read_colors(1).is_some() {
            println!("Ignoring further color attributes, only supporting COLOR_0. (mesh: {}, primitive: {})",
                mesh_index, primitive_index);
        }

        if let Some(joints) = reader.read_joints(0) {
            for (i, joint) in joints.into_u16().enumerate() {
                vertices[i].joints_0 = [
                    joint[0] as u32,
                    joint[1] as u32,
                    joint[2] as u32,
                    joint[3] as u32,
                ];
            }
        }
        if reader.read_joints(1).is_some() {
            println!("Ignoring further joint attributes, only supporting JOINTS_0. (mesh: {}, primitive: {})",
                mesh_index, primitive_index);
        }

        if let Some(weights) = reader.read_weights(0) {
            for (i, weights) in weights.into_f32().enumerate() {
                vertices[i].weights_0 = weights.into();
            }
        }
        if reader.read_weights(1).is_some() {
            println!("Ignoring further weight attributes, only supporting WEIGHTS_0. (mesh: {}, primitive: {})",
                mesh_index, primitive_index);
        }

        let indices = reader
            .read_indices()
            .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>());

        let g_material = g_primitive.material();

        let material_name = g_material
            .name()
            .map(|s| s.into())
            .unwrap_or(object_name.clone() + "-material");
        let mat = StoreMaterial::from_gltf(
            &g_material,
            textures,
            imp,
            shader_flags,
            base_path,
            object_name,
            &material_name,
        )?;
        materials.push(mat);
        let material = g_material.index().unwrap_or(0);

        Ok(Self {
            vertices,
            indices,
            material,
        })
    }
}
