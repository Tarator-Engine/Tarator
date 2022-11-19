// use std::hash::Hash;
use std::{path::PathBuf, collections::HashMap, sync::Arc};

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate bitflags;

// #[macro_use]
// extern crate cfg_if;

// #[macro_use]
// extern crate serde;


// TODO:    custom gltf importer (ugh reference maybe?)
//          support for running the whole thing on a seperate thread (send back the RawModel through a channel)
// 

// mod model;
mod scene;
mod node;
mod root;
mod primitive;
mod mesh;
mod texture;
mod material;
mod shader;

// use model::*;
use uuid::Uuid;

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

// impl Vec4Slice<f32>  for three_d_asset::Color {
//     fn as_slice(self) -> [f32; 4] {
//         [self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0, self.a as f32 / 255.0]
//     }
// }

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error {e}")]
    Io {
        #[from]
        e: std::io::Error,
    },
    #[error("Rust Message Pack encode Error {e}")]
    RmpE {
        #[from]
        e: rmp_serde::encode::Error,
    },
    #[error("Rust Message Pack decode Error {e}")]
    RmpD {
        #[from]
        e: rmp_serde::decode::Error,
    },
    #[error("Image Error {e}")]
    Image {
        #[from]
        e: image::ImageError,
    },
    #[error("GlTF Error {e}")]
    GlTF {
        #[from]
        e: gltf::Error,
    },
    #[error("The given Id does not exist")]
    NonExistentID,
    #[error("The given path does not have a file extension")]
    NoFileExtension,
    #[error("The provided image is not valid")]
    InvalidImage,
    #[error("The feature '{0}' is not yet supported")]
    NotSupported(String),
    #[error("The provided meshes do not contain position data")]
    NoPositions,
    #[error("The provided meshes do not contain normal data")]
    NoNormals,
}

type Result<T> = std::result::Result<T, Error>;

pub struct WgpuInfo {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AssetCache {
    cache: HashMap<uuid::Uuid, PathBuf>,
    orig_name: HashMap<String, Uuid>,
    last_update: chrono::DateTime<chrono::Utc>,
}

const ASSET_PATH: &'static str = "C:/Users/slackers/rust/Tarator/assets/";
const CACHE_NAME: &'static str = "cache.rmp";

pub type FSID = uuid::Uuid;

pub async fn import_gltf(path: std::path::PathBuf, pre_id: Option<Uuid>) -> Result<FSID> {
    todo!();
    // println!("importing {path:?}");
    // let t = Instant::now();

    // let tt = t.clone();

    // let model: three_d_asset::Model = load_async(&[path.clone()]).await?.deserialize(path)?;

    // println!("Importing took {:?}", t.elapsed());

    // let mut meshes = vec![];

    // let mut material_map: HashMap<String, usize> = HashMap::new();

    // let mut materials = vec![];

    // for material in model.materials {
    //     let em = material.emissive;
    //     let mut mat = StoreMaterial {
    //         diffuse_tex: None,
    //         normal_tex: None,
    //         occlusion_metallic_roughness_tex: None,
    //         metallic_roughness_tex: None,
    //         occlusion_tex: None,
    //         emissive_tex: None,
    //         diffuse_factor: material.albedo.as_slice(),
    //         normal_factor: material.normal_scale,
    //         metallic_factor: material.metallic,
    //         roughness_factor: material.roughness,
    //         occlusion_factor: material.occlusion_strength,
    //         emissive_factor: [em.r as f32 / 255.0, em.g as f32 / 255.0, em.b as f32 / 255.0, ],
    //     };

    //     let t = Instant::now();

    //     if let Some(diffuse) = material.albedo_texture {
    //         println!("saving diffuse");
    //         let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "diffuse"));
    //         mat.diffuse_tex = Some(path.clone());
    //         diffuse.serialize(path)?.save()?;
    //     }

    //     if let Some(normal) = material.normal_texture {
    //         println!("saving normal");
    //         let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "normal"));
    //         mat.normal_tex = Some(path.clone());
    //         normal.serialize(path)?.save()?;
    //     }

    //     if let Some(omr) = material.occlusion_metallic_roughness_texture {
    //         println!("saving occlusion-metallic-roughness");
    //         let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "occ-met-rou"));
    //         mat.occlusion_metallic_roughness_tex = Some(path.clone());
    //         omr.serialize(path)?.save()?;
    //     }

    //     if let Some(mr) = material.metallic_roughness_texture {
    //         println!("saving metallic-roughness");
    //         let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "met-rou"));
    //         mat.metallic_roughness_tex = Some(path.clone());
    //         mr.serialize(path)?.save()?;
    //     }

    //     if let Some(occlusion) = material.occlusion_texture {
    //         println!("saving occlusion");
    //         let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "occlusion"));
    //         mat.occlusion_tex = Some(path.clone());
    //         occlusion.serialize(path)?.save()?;
    //     }

    //     if let Some(emissive) = material.emissive_texture {
    //         println!("saving emissive");
    //         let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "emissive"));
    //         mat.emissive_tex = Some(path.clone());
    //         emissive.serialize(path)?.save()?;
    //     }
    //     materials.push(mat);

    //     material_map.insert(material.name, materials.len()-1);

    //     println!("importing textures took {:?}", t.elapsed());
    // }

    // let t = Instant::now();

    // for mesh in model.geometries {

    //     let positions = {if let Positions::F32(pos) = mesh.positions {
    //         pos.par_iter().map(|p|p.as_slice()).collect()
    //     }
    //     else {
    //         return Err(Error::WrongPosFormat);
    //     }};

    //     let normals = {
    //         if let Some(n) = mesh.normals {
    //             Some(n.par_iter().map(|n| n.as_slice()).collect())
    //         } 
    //         else {
    //             None
    //         }
    //     };

    //     let tangent = {
    //         if let Some(t) = mesh.tangents {
    //             Some(t.par_iter().map(|t| t.as_slice()).collect())
    //         }
    //         else {
    //             None
    //         }
    //     };

    //     let tex_coords = {
    //         if let Some(tc) = mesh.uvs {
    //             Some(tc.par_iter().map(|tc| tc.as_slice()).collect())
    //         }
    //         else {
    //             None
    //         }
    //     };

    //     let indices = {
    //         match mesh.indices {
    //             Indices::None => return Err(Error::NoIndexBuffer),
    //             Indices::U8(v) => v.par_iter().map(|v| *v as usize).collect(),
    //             Indices::U16(v) => v.par_iter().map(|v| *v as usize).collect(),
    //             Indices::U32(v) => v.par_iter().map(|v| *v as usize).collect(),
    //         }
    //     };

    //     meshes.push(StoreMesh { 
    //         positions,
    //         normals,
    //         tangents: tangent,
    //         tex_coords,
    //         indices,
    //         material: *material_map.get(&mesh.material_name.unwrap_or("".to_string())).unwrap_or(&(0 as usize)),
    //     });
    // }

    // println!("mesh import took {:?}", t.elapsed());

    // let id = match pre_id { None => Uuid::new_v4(), Some(id) => id };

    // let m = StoreModel {
    //     id,
    //     meshes,
    //     materials,
    //     instances: vec![StoreInstance{position: [0.0; 3], rotation: [0.0; 4]}],
    //     instance_num: 1,
    // };

    // let t = Instant::now();
    // println!("saving model");
    // let path = PathBuf::from(ASSET_PATH).join(format_model_name(id));
    
    // std::fs::write(path.clone(), rmp_serde::to_vec(&m)?)?;

    // update_cache(m.id, path).await?;
    // println!("saving took {:?}", t.elapsed());
    // println!("total time {:?}", tt.elapsed());

    // Ok(id)
}

pub async fn update_cache(id: Uuid, location: PathBuf) -> Result<()> {

    let path = PathBuf::from(ASSET_PATH).join(CACHE_NAME);
    
    let mut cache = get_cache().await?;

    cache.cache.insert(id, location.clone());
    cache.orig_name.insert(location.file_name().ok_or(Error::NoFileExtension)?.to_str().unwrap().to_owned(), id);
    cache.last_update = chrono::offset::Utc::now();

    std::fs::write(path, rmp_serde::to_vec(&cache)?)?;

    Ok(())
}

pub async fn get_cache() -> Result<AssetCache> {
    let path = PathBuf::from(ASSET_PATH).join(CACHE_NAME);
    rmp_serde::from_slice(std::fs::read(path)?.as_slice())
        .map_err(|e| Error::RmpD {e})
}

pub fn format_model_name(model_id: uuid::Uuid) -> String {
    format!("model-{model_id}.tarm")
}

pub fn format_img_name(mat_name: String, ty: &'static str) -> String {
    format!("img-{mat_name}-{ty}.png")
}

pub async fn reset_cache() -> Result<()> {
    let path = PathBuf::from(ASSET_PATH).join(CACHE_NAME);
    let cache = AssetCache {
        cache: HashMap::new(),
        orig_name: HashMap::new(),
        last_update: chrono::offset::Utc::now(),
    };

    std::fs::write(path, rmp_serde::to_vec(&cache)?).map_err(|e| Error::Io { e })
}