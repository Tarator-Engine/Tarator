use std::path::PathBuf;

#[macro_use]
extern crate thiserror;

// #[macro_use]
// extern crate cfg_if;

// #[macro_use]
// extern crate serde;

mod model;

use model::*;

trait Vec2Slice<T> {
    fn as_slice(self) -> [T; 2];
}

impl<T> Vec2Slice<T> for cgmath::Vector2<T> {
    fn as_slice(self) -> [T; 2] {
        [self.x, self.y]
    }
}

trait Vec3Slice<T> {
    fn as_slice(self) -> [T; 3];
}

impl<T> Vec3Slice<T> for cgmath::Vector3<T> {
    fn as_slice(self) -> [T; 3] {
        [self.x, self.y, self.z]
    }
}

trait Vec4Slice<T> {
    fn as_slice(self) -> [T; 4];
}

impl<T> Vec4Slice<T> for cgmath::Vector4<T> {
    fn as_slice(self) -> [T; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

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
    #[error("gltf Error")]
    Gltf {
        #[from]
        source: Box<dyn std::error::Error>,
    },
    #[error("Rmp Error")]
    Rmp {
        #[from]
        source: rmp_serde::encode::Error,
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
    #[error("There have to be tangents, normals, and texture coordinates for all vertices")]
    MissingValues,

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

const ASSET_PATH: &'static str = "C:/Users/slackers/rust/Tarator/assets/";

pub type FSID = uuid::Uuid;

pub fn import_gltf(path: std::path::PathBuf) -> Result<Vec<FSID>, ImportError> {
    let scenes = easy_gltf::load(path).map_err(|e| ImportError::Gltf { source: e })?;

    let mut fs_ids = vec![];

    for scene in scenes {
        let id = uuid::Uuid::new_v4();
        let mut meshes = vec![];
        let mut materials = vec![];
        for (i, model) in scene.models.iter().enumerate() {
            
            let vertices = model.vertices();

            let vertices: Vec<StoreVertex> = vertices.iter().map(|v| {
                StoreVertex {
                    position: v.position.as_slice(),
                    normal: v.normal.as_slice(),
                    tangent: v.tangent.as_slice(),
                    tex_coords: v.tex_coords.as_slice(),
                }
            }).collect();

            let indices =  model.indices().ok_or(ImportError::NoIndexBuffer)?;

            if !(model.has_normals() && model.has_tangents() && model.has_tex_coords()) {
                return Err(ImportError::MissingValues);
            }

            let material = model.material();

            let mut mat = StoreMaterial {
                diffuse_tex: None,
                normal_tex: None,
                metallic_tex: None,
                roughness_tex: None,
                occlusion_tex: None,
                emissive_tex: None, // TODO: sensible defaults that will cause a working render
                diffuse_factor: [0.0; 4],
                metallic_factor: 0.0,
                roughness_factor: 0.0,
                normal_factor: 0.0,
                occlusion_factor: 0.0,
                emissive_factor: [0.0; 3],
            };

            if let Some(diffuse) = &material.pbr.base_color_texture {
                let name = format_img_name(id, i, "diffuse");

                diffuse.save(std::path::PathBuf::from(ASSET_PATH).join(name.clone()))?;

                mat.diffuse_tex = Some(PathBuf::from(name));
                mat.diffuse_factor = material.pbr.base_color_factor.as_slice();
            }

            if let Some(normal) = &material.normal {
                let name = format_img_name(id, i, "normal");

                normal.texture.save(std::path::PathBuf::from(ASSET_PATH).join(name.clone()))?;


                mat.normal_tex = Some(PathBuf::from(name));
                mat.normal_factor = normal.factor;
            }

            if let Some(roughness) = &material.pbr.roughness_texture {
                let name = format_img_name(id, i, "roughness");
                
                roughness.save(std::path::PathBuf::from(ASSET_PATH).join(name.clone()))?;


                mat.roughness_tex = Some(PathBuf::from(name));
                mat.roughness_factor = material.pbr.roughness_factor;
            }

            if let Some(metallic) = &material.pbr.metallic_texture {
                let name = format_img_name(id, i, "metallic");

                metallic.save(std::path::PathBuf::from(ASSET_PATH).join(name.clone()))?;

                mat.metallic_tex = Some(PathBuf::from(name));
                mat.metallic_factor = material.pbr.metallic_factor;
            }

            if let Some(occlusion) = &material.occlusion {
                let name = format_img_name(id, i, "metallic");

                occlusion.texture.save(std::path::PathBuf::from(ASSET_PATH).join(name.clone()))?;

                mat.occlusion_tex = Some(PathBuf::from(name));
                mat.occlusion_factor = occlusion.factor;
            }

            if let Some(emissive) = &material.emissive.texture {
                let name = format_img_name(id, i, "metallic");

                emissive.save(std::path::PathBuf::from(ASSET_PATH).join(name.clone()))?;

                mat.emissive_tex = Some(PathBuf::from(name));
                mat.emissive_factor = material.emissive.factor.as_slice();
            }

            meshes.push(StoreMesh {
                vertices,
                indices: indices.clone(),
                material: materials.len(),
            });

            materials.push(mat);
             

            todo!("Import the rest of the model");
        }

        let m = StoreModel {
            id,
            meshes,
            materials,
            instances: vec![StoreInstance {position: [0.0; 3], rotation: [0.0; 4]}],
            instance_num: 1,
        };
        
        fs_ids.push(m.id);
        std::fs::write(format_model_name(m.id), rmp_serde::to_vec(&m)?.as_slice())?;
    }

    Ok(fs_ids)
}

pub fn format_model_name(model_id: uuid::Uuid) -> String {
    format!("img-{model_id}.tarm")
}

pub fn format_img_name(model_id: uuid::Uuid, mesh_index: usize, ty: &'static str) -> String {
    format!("img-{model_id}-{mesh_index}-{ty}.png")
}