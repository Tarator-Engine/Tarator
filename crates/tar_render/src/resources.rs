use std::{io::{BufReader, Cursor}};

use cfg_if::cfg_if;
use wgpu::util::DeviceExt;
use thiserror::Error;

trait Vec3Slice<T> {
    fn as_slice(self) -> [T; 3];
}

impl<T> Vec3Slice<T> for cgmath::Vector3<T> {
    fn as_slice(self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}



#[derive(Error, Debug)]
pub enum ImportError {
    #[error("unsupported type: {0:?}")]
    UnsupportedType(String),
    #[error("Missing file ending")]
    NoType,
    #[error("Io Error")]
    Io {
        #[from]
        source: std::io::Error
    },
    #[error("Image Error")]
    Image {
        #[from]
        source: image::ImageError
    },
    #[error("obj Error")]
    Tobj {
        #[from]
        source: tobj::LoadError,
    },
    #[error("gltf Error")]
    Gltf {
        source: Box<dyn std::error::Error>,
    },

    #[error("There has to be an index buffer")]
    NoIndexBuffer,
    #[error("There has to be a Normal map")]
    NoNormalMap,
    #[error("There has to be a Base map")]
    NoBaseMap,
}
use crate::{model::{self}, texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        location.origin().unwrap(),
        option_env!("RES_PATH").unwrap_or("res"),
    ))
    .unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> Result<String, ImportError> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            println!("loading: {:?}", file_name);
            let path = std::path::Path::new(file_name);
            let txt = std::fs::read_to_string(path).map_err(|e| e)?;
        }
    }

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> Result<Vec<u8>, ImportError> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            println!("loading {:?}", file_name);
            let path = std::path::Path::new(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> Result<texture::RawTexture, ImportError> {
    let data = load_binary(file_name).await?;
    match texture::RawTexture::from_bytes(device, queue, &data, file_name, is_normal_map) {
        Ok(v) => Ok(v),
        Err(e) => Err(ImportError::Image { source: e }),
    }
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    instance_buffer: wgpu::Buffer,
    instance_num: u32,
) -> Result<model::RawModel, ImportError> {

    let parts = file_name.split('.').collect::<Vec<&str>>();

    let l = parts.len();

    if l != 0 {
        match parts[l-1].to_lowercase().as_str() {
            "obj" => {
                return load_obj_model(file_name, device, queue, layout, instance_buffer, instance_num).await;
            }

            "glb" |
            "gltf" => {
                todo!("gltf loading");
                //return load_gltf_model(file_name, device, queue, layout, instance_buffer, instance_num).await;
            }


            f => {
                return Err(ImportError::UnsupportedType(f.to_string()));
            }
        }
    }
    else {
        return Err(ImportError::NoType)
    }
}

// pub async fn load_gltf_model(
//     file_name: &str,
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
//     layout: &wgpu::BindGroupLayout,
//     instance_buffer: wgpu::Buffer,
//     instance_num: u32,
// ) -> Result<model::RawModel, ImportError> {
//     println!("started loading");
//     let scenes = easy_gltf::load(file_name).map_err(|e| ImportError::Gltf {source: e})?;

//     println!("gltf imported");
//     let mut meshes = vec![];
//     let mut materials = vec![];
//     for scene in scenes {
//         for model in scene.models {
//             let o_verts = model.vertices();
//             let mut n_verts = vec![];
//             let o_inds = model.indices().ok_or(ImportError::NoIndexBuffer)?;
//             let num_elements = o_inds.len() as u32;
//             let n_inds: Vec<u32> = o_inds.iter().map(|x| *x as u32).collect();

//             let mat = model.material();
//             let diffuse_texture = (*mat).pbr.base_color_texture.to_owned().ok_or(ImportError::NoBaseMap)?;
//             let dimensions = diffuse_texture.dimensions();
//             let diffuse_texture = texture::RawTexture::from_image_buffer(device, queue, &diffuse_texture, dimensions, Some("basetext"), false)?;

//             let normal_texture = mat.normal.to_owned().ok_or(ImportError::NoNormalMap)?;
//             let normal_texture = normal_texture.texture;
//             let normal_texture = texture::RawTexture::from_image(device, queue, &image::DynamicImage::ImageRgb8((*normal_texture).to_owned()), Some("normaltext"), true)?;
            
//             for vert in o_verts { // TODO: find out why the easy-gltf library omits bitangents (precomputation)
//                 let bitangent = cgmath::Vector3::cross(cgmath::Vector3::new(vert.tangent.x, vert.tangent.y, vert.tangent.z), vert.normal);
//                 n_verts.push(model::ModelVertex {
//                     position: vert.position.as_slice(),
//                     tangent: [vert.tangent.x, vert.tangent.y, vert.tangent.z],
//                     normal: vert.normal.as_slice(),
//                     tex_coords: [vert.tex_coords.x, vert.tex_coords.y],
//                     bitangent: bitangent.as_slice(),
//                 })
//             }

//             let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some(&format!("{:?} Vertex Buffer", file_name)),
//                 contents: bytemuck::cast_slice(&n_verts),
//                 usage: wgpu::BufferUsages::VERTEX,
//             });
//             let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some(&format!("{:?} Index Buffer", file_name)),
//                 contents: bytemuck::cast_slice(&n_inds),
//                 usage: wgpu::BufferUsages::INDEX,
//             });



//             materials.push(model::RawMaterial::new(device, "material", diffuse_texture, normal_texture, layout));
//             meshes.push(model::RawMesh {
//                 num_elements,
//                 material: materials.len()-1,
//                 index_buffer,
//                 vertex_buffer
//             });

//             println!("loaded one model");
//         }
//     }
    
//     Ok(model::RawModel {
//         id: 0,
//         meshes,
//         materials,
//         instance_buffer,
//         instance_num,
//     })
// }

pub async fn load_obj_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    instance_buffer: wgpu::Buffer,
    instance_num: u32,
) -> Result<model::RawModel, ImportError> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let path = std::path::Path::new(file_name).ancestors().nth(1).unwrap().to_str().unwrap();

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string([path, "/", &p].concat().as_str()).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    if obj_materials.is_ok() {
        for m in obj_materials? {
            let diffuse_texture = load_texture([path, "/", &m.diffuse_texture].concat().as_str(), false, device, queue).await?;
            
            let normal_texture = load_texture([path, "/", &m.normal_texture].concat().as_str(), true, device, queue).await?;

            materials.push(model::RawMaterial::new(
                device,
                &m.name,
                diffuse_texture,
                normal_texture,
                layout,
            ));
        }
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    // We'll calculate these later
                    tangent: [0.0; 4],
                    zero: 0.0,
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y;

                let tangent = cgmath::Vector4::new(tangent.x, tangent.y, tangent.z, r); // TODO: -r?

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent =
                    (tangent + cgmath::Vector4::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + cgmath::Vector4::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + cgmath::Vector4::from(vertices[c[2] as usize].tangent)).into();

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // Average the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (cgmath::Vector4::from(v.tangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::RawMesh {
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::RawModel {
        id: 0,
        instance_buffer,
        instance_num,
        meshes,
        materials,
    })
}
