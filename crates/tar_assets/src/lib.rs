use std::collections::HashMap;
use std::path::Path;
use std::{io::{BufReader, Cursor}};

use image::DynamicImage;
use tobj::LoadError;

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate serde;

mod model;

use model::*;

#[derive(Error, Debug)]
pub enum ImportError {
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
    #[error("There has to be a file and file extension")]
    NoFileExtension,
    #[error("The file extensions {0:?} is not supported")]
    UnsupportedFileExtension(String),
    #[error("There has to be a parent directory")]
    MissingParentDirectory,

    #[error("Error during saving of the resulting model")]
    ExportError {
        #[from]
        source: ExportError
    }
}

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("io Error")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Rust MessagePack encode error")]
    RpmEncode {
        #[from]
        source: rmp_serde::encode::Error,
    },
    #[error("Rust MessagePack decode error")]
    RpmDecode {
        #[from]
        source: rmp_serde::decode::Error,
    },
    
    #[error("Image Error")]
    Image {
        #[from]
        source: image::ImageError
    },
    #[error("A path is required when loading images from Model struct")]
    MissingPath,
}

pub struct AssetRef {
    name: String,
    fs_id: u32,
}


pub async fn load_asset_from_path(file_path: &str) -> Result<AssetRef, ImportError> {

    let path = Path::new(file_path);

    match path.extension().ok_or(ImportError::NoFileExtension)?.to_str().ok_or(ImportError::NoFileExtension)? {

        "gltf" | "glb" => {
            import_gltf(path)
        }

        // "obj" => {
        //     import_obj(path).await
        // }

        "mp3" | "wav" | "ogg" => {
            todo!("audio loading")
        }

        f => {return Err(ImportError::UnsupportedFileExtension(f.to_owned()))}
    }
}

pub fn import_gltf(path: &Path) -> Result<AssetRef, ImportError> {
    todo!()
}

pub fn import_image(path: &Path) -> Result<DynamicImage, ImportError> {
    image::open(path).map_err(|e| ImportError::Image { source: e })
}

pub async fn load_binary(path: &Path) -> Result<Vec<u8>, ImportError> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            println!("loading binary {:?}", path);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

pub async fn load_string(path: &Path) -> Result<String, ImportError> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            println!("loading string: {:?}", path);
            let txt = std::fs::read_to_string(path).map_err(|e| e)?;
        }
    }

    Ok(txt)
}

// pub async fn import_obj(
//     path: &Path,
// ) -> Result<AssetRef, ImportError> {
//     let obj_text = load_string(path).await?;
//     let obj_cursor = Cursor::new(obj_text);
//     let mut obj_reader = BufReader::new(obj_cursor);

//     let parent_dir = path.ancestors().nth(1).ok_or(ImportError::MissingParentDirectory)?;

//     let (models, obj_materials) = tobj::load_obj_buf_async(
//         &mut obj_reader,
//         &tobj::LoadOptions {
//             triangulate: true,
//             single_index: true,
//             ..Default::default()
//         },
//         |p| async move {
//             let mat_text = load_string(&path.join(p.to_owned())).await.map_err(|e| LoadError::OpenFileFailed)?;
//             tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
//         },
//     )
//     .await?;
    
//     let mut images = HashMap::new();
//     if obj_materials.is_ok() {
//         for m in obj_materials? {
//             let diffuse_texture = image::open(path.join(m.diffuse_texture)).map_err(|e| ExportError::Image { source: e })?;
//             let normal_texture = image::open(path.join(m.normal_texture)).map_err(|e| ExportError::Image { source: e })?;
//             images.insert(m.diffuse_texture, diffuse_texture);
//             images.insert(m.normal_texture, normal_texture);

//         }
//     }

//     let meshes = models
//     .into_iter()
//     .map(|m| {
//         let mut vertices = (0..m.mesh.positions.len() / 3)
//             .map(|i| Vertex {
//                 position: [
//                     m.mesh.positions[i * 3],
//                     m.mesh.positions[i * 3 + 1],
//                     m.mesh.positions[i * 3 + 2],
//                 ],
//                 tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
//                 normal: [
//                     m.mesh.normals[i * 3],
//                     m.mesh.normals[i * 3 + 1],
//                     m.mesh.normals[i * 3 + 2],
//                 ],
//                 // We'll calculate these later
//                 tangent: [0.0; 4],
//             })
//             .collect::<Vec<_>>();

//         let indices = &m.mesh.indices;
//         let mut triangles_included = vec![0; vertices.len()];

//         // Calculate tangents and bitangets. We're going to
//         // use the triangles, so we need to loop through the
//         // indices in chunks of 3
//         for c in indices.chunks(3) {
//             let v0 = vertices[c[0] as usize];
//             let v1 = vertices[c[1] as usize];
//             let v2 = vertices[c[2] as usize];

//             let pos0: cgmath::Vector3<_> = v0.position.into();
//             let pos1: cgmath::Vector3<_> = v1.position.into();
//             let pos2: cgmath::Vector3<_> = v2.position.into();

//             let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
//             let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
//             let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

//             // Calculate the edges of the triangle
//             let delta_pos1 = pos1 - pos0;
//             let delta_pos2 = pos2 - pos0;

//             // This will give us a direction to calculate the
//             // tangent and bitangent
//             let delta_uv1 = uv1 - uv0;
//             let delta_uv2 = uv2 - uv0;

//             // Solving the following system of equations will
//             // give us the tangent and bitangent.
//             //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
//             //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
//             // Luckily, the place I found this equation provided
//             // the solution!
//             let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
//             let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y);

//             let tangent = cgmath::Vector4::new(tangent.x, tangent.y, tangent.z, r); // TODO: -r?

//             // We'll use the same tangent/bitangent for each vertex in the triangle
//             vertices[c[0] as usize].tangent =
//                 (tangent + cgmath::Vector4::from(vertices[c[0] as usize].tangent)).into();
//             vertices[c[1] as usize].tangent =
//                 (tangent + cgmath::Vector4::from(vertices[c[1] as usize].tangent)).into();
//             vertices[c[2] as usize].tangent =
//                 (tangent + cgmath::Vector4::from(vertices[c[2] as usize].tangent)).into();

//             // Used to average the tangents/bitangents
//             triangles_included[c[0] as usize] += 1;
//             triangles_included[c[1] as usize] += 1;
//             triangles_included[c[2] as usize] += 1;
//         }

//         // Average the tangents/bitangents
//         for (i, n) in triangles_included.into_iter().enumerate() {
//             let denom = 1.0 / n as f32;
//             let mut v = &mut vertices[i];
//             v.tangent = (cgmath::Vector4::from(v.tangent) * denom).into();
//         }

//         Model {

//         }
//     })
//     .collect::<Vec<_>>();


//     let model = crate::model::Model {
//         // unwrap_justified as there would have been an error earlyer
//         name: path.to_str().unwrap().to_owned(),
//         vertices: meshes[0],

//     };

//     save_model(model, images, None).await.map_err(|e| ImportError::ExportError { source: e })
// }