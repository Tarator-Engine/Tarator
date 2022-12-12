use std::{path::Path, sync::Arc};

use cgmath::{Matrix4, Vector3};

use crate::{
    material::PbrMaterial, scene::ImportData, shader::ShaderFlags, vertex::Vertex, Error, Result,
    WgpuInfo,
};

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}
impl Instance {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 13,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 14,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
pub struct Primitive {
    pub vertices: wgpu::Buffer,
    pub num_vertices: u32,

    pub indices: wgpu::Buffer,
    pub num_indices: u32,

    pub material: PbrMaterial,
}

impl Primitive {
    pub fn new(
        vertices: &[Vertex],
        indices: Option<Vec<u32>>,
        material: PbrMaterial,
        w_info: &WgpuInfo,
    ) -> Result<Self> {
        let num_indices = indices.as_ref().map(|i| i.len()).unwrap_or(0) as u32;
        let num_vertices = vertices.len() as u32;

        let vertices = w_info
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        if indices.is_none() {
            return Err(Error::NotSupported("models without indicies".into()));
        };
        let indices = w_info
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index buffer"),
                contents: bytemuck::cast_slice(&indices.unwrap()),
                usage: wgpu::BufferUsages::INDEX,
            });

        let mut prim = Primitive {
            num_vertices,
            num_indices,
            vertices,
            indices,
            material,
        };
        Ok(prim)
    }

    // pub fn from_gltf(
    //     g_primitive: &gltf::Primitive<'_>,
    //     primitive_index: usize,
    //     mesh_index: usize,
    //     root: &mut Root,
    //     imp: &ImportData,
    //     base_path: &Path,
    //     w_info: &WgpuInfo,
    // ) -> Result<Self> {
    //     let buffers = &imp.buffers;
    //     let reader = g_primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    //     let positions = reader
    //         .read_positions()
    //         .ok_or(Error::NoPositions)?
    //         .collect::<Vec<_>>();

    //     let mut vertices: Vec<Vertex> = positions
    //         .into_iter()
    //         .map(|position| Vertex {
    //             position,
    //             ..Vertex::default()
    //         })
    //         .collect();

    //     let mut shader_flags = ShaderFlags::empty();

    //     if let Some(normals) = reader.read_normals() {
    //         for (i, normal) in normals.enumerate() {
    //             vertices[i].normal = normal;
    //         }
    //         shader_flags |= ShaderFlags::HAS_NORMALS;
    //     } else {
    //         return Err(Error::NotSupported("normal calculation".to_owned()));
    //     }

    //     if let Some(tangents) = reader.read_tangents() {
    //         for (i, tangent) in tangents.enumerate() {
    //             vertices[i].tangent = tangent;
    //         }
    //         shader_flags |= ShaderFlags::HAS_TANGENTS;
    //     }

    //     let mut tex_coord_set = 0;
    //     while let Some(tex_coords) = reader.read_tex_coords(tex_coord_set) {
    //         if tex_coord_set > 1 {
    //             println!(
    //                 "Ignoring texture coordinate set {}, \
    //                 only supporting 2 sets at the moment. (mesh: {}, primitive: {})",
    //                 tex_coord_set, mesh_index, primitive_index
    //             );
    //             tex_coord_set += 1;
    //             continue;
    //         }
    //         for (i, tex_coord) in tex_coords.into_f32().enumerate() {
    //             match tex_coord_set {
    //                 0 => vertices[i].tex_coord_0 = tex_coord,
    //                 1 => vertices[i].tex_coord_1 = tex_coord,
    //                 _ => unreachable!(),
    //             }
    //         }
    //         shader_flags |= ShaderFlags::HAS_UV;
    //         tex_coord_set += 1;
    //     }

    //     if let Some(colors) = reader.read_colors(0) {
    //         let colors = colors.into_rgba_f32();
    //         for (i, c) in colors.enumerate() {
    //             vertices[i].color_0 = c.into();
    //         }
    //         shader_flags |= ShaderFlags::HAS_COLORS;
    //     }

    //     if reader.read_colors(1).is_some() {
    //         println!("Ignoring further color attributes, only supporting COLOR_0. (mesh: {}, primitive: {})",
    //             mesh_index, primitive_index);
    //     }

    //     if let Some(joints) = reader.read_joints(0) {
    //         for (i, joint) in joints.into_u16().enumerate() {
    //             vertices[i].joints_0 = [
    //                 joint[0] as u32,
    //                 joint[1] as u32,
    //                 joint[2] as u32,
    //                 joint[3] as u32,
    //             ];
    //         }
    //     }
    //     if reader.read_joints(1).is_some() {
    //         println!("Ignoring further joint attributes, only supporting JOINTS_0. (mesh: {}, primitive: {})",
    //             mesh_index, primitive_index);
    //     }

    //     if let Some(weights) = reader.read_weights(0) {
    //         for (i, weights) in weights.into_f32().enumerate() {
    //             vertices[i].weights_0 = weights.into();
    //         }
    //     }
    //     if reader.read_weights(1).is_some() {
    //         println!("Ignoring further weight attributes, only supporting WEIGHTS_0. (mesh: {}, primitive: {})",
    //             mesh_index, primitive_index);
    //     }

    //     let indices = reader
    //         .read_indices()
    //         .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>());

    //     let g_material = g_primitive.material();

    //     let mut material = None;
    //     if let Some(mat) = root
    //         .materials
    //         .iter()
    //         .enumerate()
    //         .find(|(_, m)| (***m).index == g_material.index())
    //     {
    //         material = mat.0.into()
    //     }

    //     if material.is_none() {
    //         // no else due to borrow checker madness
    //         let mat = Arc::new(PbrMaterial::from_gltf(
    //             &g_material,
    //             root,
    //             imp,
    //             shader_flags,
    //             base_path,
    //             w_info,
    //         )?);
    //         root.materials.push(Arc::clone(&mat));
    //         material = Some(root.materials.len() - 1);
    //     };
    //     let material = material.unwrap();

    //     Primitive::new(&vertices, indices, material, w_info)
    // }

    pub fn draw<'a, 'b>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'a>,
        model_matrix: &Matrix4<f32>,
        mvp_matrix: &Matrix4<f32>,
        camera_position: &Vector3<f32>,
    ) -> Result<()> {
        // TODO!: determine if shader+material already active to reduce work...

        self.material.set_render_pass(render_pass);

        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
        self.material.set_bind_groups(render_pass, 0);

        Ok(())
    }
}
