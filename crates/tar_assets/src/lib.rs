// use std::hash::Hash;
use std::{path::PathBuf, collections::HashMap};

#[macro_use]
extern crate thiserror;

use std::time::Instant;

// #[macro_use]
// extern crate cfg_if;

// #[macro_use]
// extern crate serde;


// TODO:    custom gltf importer (ugh reference maybe?)
//          support for running the whole thing on a seperate thread (send back the RawModel through a channel)
// 

mod model;

use model::*;
use three_d_asset::{io::{load_async, Serialize}, Positions, Indices};
use uuid::Uuid;
use wgpu::util::DeviceExt;

use rayon::prelude::*;

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

impl Vec4Slice<f32>  for three_d_asset::Color {
    fn as_slice(self) -> [f32; 4] {
        [self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0, self.a as f32 / 255.0]
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io Error")]
    Io {
        #[from]
        e: std::io::Error
    },
    #[error("Image Error")]
    Image {
        #[from]
        e: image::ImageError
    },
    #[error("gltf Error")]
    Gltf {
        #[from]
        e: Box<dyn std::error::Error>,
    },
    #[error("three-d-asset Error")]
    ThreeDAsset {
        #[from]
        e: three_d_asset::Error,
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
    #[error("The positions have to be given as f32 not f64")]
    WrongPosFormat,
    #[error("Rust MessagePack encode error")]
    RpmEncode {
        #[from]
        e: rmp_serde::encode::Error,
    },
    #[error("Rust MessagePack decode error")]
    RpmDecode {
        #[from]
        e: rmp_serde::decode::Error,
    },
    #[error("A path is required when loading images from Model struct")]
    MissingPath,
    #[error("The provided id is not in the asset cache")]
    NonExistentID,

    #[error("The image was invalid")]
    InvalidImage,
}

type Result<T> = std::result::Result<T, Error>;

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
    println!("importing {path:?}");
    let t = Instant::now();

    let tt = t.clone();

    let model: three_d_asset::Model = load_async(&[path.clone()]).await?.deserialize(path)?;

    println!("Importing took {:?}", t.elapsed());

    let mut meshes = vec![];

    let mut material_map: HashMap<String, usize> = HashMap::new();

    let mut materials = vec![];

    for material in model.materials {
        let em = material.emissive;
        let mut mat = StoreMaterial {
            diffuse_tex: None,
            normal_tex: None,
            occlusion_metallic_roughness_tex: None,
            metallic_roughness_tex: None,
            occlusion_tex: None,
            emissive_tex: None,
            diffuse_factor: material.albedo.as_slice(),
            normal_factor: material.normal_scale,
            metallic_factor: material.metallic,
            roughness_factor: material.roughness,
            occlusion_factor: material.occlusion_strength,
            emissive_factor: [em.r as f32 / 255.0, em.g as f32 / 255.0, em.b as f32 / 255.0, ],
        };

        let t = Instant::now();

        if let Some(diffuse) = material.albedo_texture {
            println!("saving diffuse");
            let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "diffuse"));
            mat.diffuse_tex = Some(path.clone());
            diffuse.serialize(path)?.save()?;
        }

        if let Some(normal) = material.normal_texture {
            println!("saving normal");
            let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "normal"));
            mat.normal_tex = Some(path.clone());
            normal.serialize(path)?.save()?;
        }

        if let Some(omr) = material.occlusion_metallic_roughness_texture {
            println!("saving occlusion-metallic-roughness");
            let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "occ-met-rou"));
            mat.occlusion_metallic_roughness_tex = Some(path.clone());
            omr.serialize(path)?.save()?;
        }

        if let Some(mr) = material.metallic_roughness_texture {
            println!("saving metallic-roughness");
            let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "met-rou"));
            mat.metallic_roughness_tex = Some(path.clone());
            mr.serialize(path)?.save()?;
        }

        if let Some(occlusion) = material.occlusion_texture {
            println!("saving occlusion");
            let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "occlusion"));
            mat.occlusion_tex = Some(path.clone());
            occlusion.serialize(path)?.save()?;
        }

        if let Some(emissive) = material.emissive_texture {
            println!("saving emissive");
            let path = std::path::PathBuf::from(ASSET_PATH).join(format_img_name(material.name.clone(), "emissive"));
            mat.emissive_tex = Some(path.clone());
            emissive.serialize(path)?.save()?;
        }
        materials.push(mat);

        material_map.insert(material.name, materials.len()-1);

        println!("importing textures took {:?}", t.elapsed());
    }

    let t = Instant::now();

    for mesh in model.geometries {

        let positions = {if let Positions::F32(pos) = mesh.positions {
            pos.par_iter().map(|p|p.as_slice()).collect()
        }
        else {
            return Err(Error::WrongPosFormat);
        }};

        let normals = {
            if let Some(n) = mesh.normals {
                Some(n.par_iter().map(|n| n.as_slice()).collect())
            } 
            else {
                None
            }
        };

        let tangent = {
            if let Some(t) = mesh.tangents {
                Some(t.par_iter().map(|t| t.as_slice()).collect())
            }
            else {
                None
            }
        };

        let tex_coords = {
            if let Some(tc) = mesh.uvs {
                Some(tc.par_iter().map(|tc| tc.as_slice()).collect())
            }
            else {
                None
            }
        };

        let indices = {
            match mesh.indices {
                Indices::None => return Err(Error::NoIndexBuffer),
                Indices::U8(v) => v.par_iter().map(|v| *v as usize).collect(),
                Indices::U16(v) => v.par_iter().map(|v| *v as usize).collect(),
                Indices::U32(v) => v.par_iter().map(|v| *v as usize).collect(),
            }
        };

        meshes.push(StoreMesh { 
            positions,
            normals,
            tangents: tangent,
            tex_coords,
            indices,
            material: *material_map.get(&mesh.material_name.unwrap_or("".to_string())).unwrap_or(&(0 as usize)),
        });
    }

    println!("mesh import took {:?}", t.elapsed());

    let id = match pre_id { None => Uuid::new_v4(), Some(id) => id };

    let m = StoreModel {
        id,
        meshes,
        materials,
        instances: vec![StoreInstance{position: [0.0; 3], rotation: [0.0; 4]}],
        instance_num: 1,
    };

    let t = Instant::now();
    println!("saving model");
    let path = PathBuf::from(ASSET_PATH).join(format_model_name(id));
    
    std::fs::write(path.clone(), rmp_serde::to_vec(&m)?)?;

    update_cache(m.id, path).await?;
    println!("saving took {:?}", t.elapsed());
    println!("total time {:?}", tt.elapsed());

    Ok(id)
}

pub async fn load_model(model_id: FSID, device: wgpu::Device, queue: wgpu::Queue) -> Result<RawModel>{
    let cache = get_cache().await?;

    let path = cache.cache.get(&model_id).ok_or(Error::NonExistentID)?;

    let store_model: StoreModel = rmp_serde::from_slice(std::fs::read(path)?.as_slice())?;

    let meshes: Vec<RawMesh> = store_model.meshes.par_iter().map(|mesh| {
        let positions = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} positions", store_model.id)),
            contents: bytemuck::cast_slice(&mesh.positions),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let normals = {
            if mesh.normals.is_none() {
                None
            }
            else {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} normals", store_model.id)),
                    contents: bytemuck::cast_slice(&mesh.normals.as_ref().unwrap()),
                    usage: wgpu::BufferUsages::VERTEX,
                }))
            }
        };
        let tangents = {
            if mesh.tangents.is_none() {
                None
            }
            else {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} tangents", store_model.id)),
                    contents: bytemuck::cast_slice(&mesh.tangents.as_ref().unwrap()),
                    usage: wgpu::BufferUsages::VERTEX,
                }))
            }
        };
        let tex_coords = {
            if mesh.tex_coords.is_none() {
                None
            }
            else {
                Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} tex_coords", store_model.id)),
                    contents: bytemuck::cast_slice(&mesh.tex_coords.as_ref().unwrap()),
                    usage: wgpu::BufferUsages::VERTEX,
                }))
            }
        };
        let num_indices = mesh.indices.len();
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} indices", store_model.id)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        RawMesh {
            positions,
            normals,
            tangents,
            tex_coords,
            indices,
            num_indices: num_indices as u32,
            material: mesh.material as u32,
        }
    }).collect();

    let mut materials = vec![];
    for m in store_model.materials.iter() {

        let diffuse = if let Some(diff) = &m.diffuse_tex {
            let img = image::open(diff)?;
            Some(RawTexture::diffuse_texture(&device, &queue, &img, diff.to_str())?)
        } else {
            None
        };

        let normal = if let Some(norm) = &m.normal_tex {
            let img = image::open(norm)?;
            Some(RawTexture::normal_texture(&device, &queue, &img, norm.to_str())?)
        } else {
            None
        };

        let omr = if let Some(omr) = &m.occlusion_metallic_roughness_tex {
            let img = image::open(omr)?;
            Some(RawTexture::occ_met_rou_texture(&device, &queue, &img, omr.to_str())?)
        } else {
            None
        };

        let mr = if let Some(mr) = &m.metallic_roughness_tex {
            let img = image::open(mr)?;
            Some(RawTexture::occ_met_rou_texture(&device, &queue, &img, mr.to_str())?)
        } else {
            None
        };

        let occlusion = if let Some(occlusion) = &m.occlusion_tex {
            let img = image::open(occlusion)?;
            Some(RawTexture::occ_met_rou_texture(&device, &queue, &img, occlusion.to_str())?)
        } else {
            None
        };

        let emissive = if let Some(emissive) = &m.emissive_tex {
            let img = image::open(emissive)?;
            Some(RawTexture::occ_met_rou_texture(&device, &queue, &img, emissive.to_str())?)
        } else {
            None
        };



        // materials.push(RawMaterial {

        // })
    }


    let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} instances", store_model.id)),
        contents: bytemuck::cast_slice(&vec![StoreInstance {position: [0.0; 3], rotation: [0.0; 4]}]),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let model = RawModel {
        id: store_model.id,
        meshes,
        materials,
        instances,
        instance_num: 1,
    };


    todo!()
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
        .map_err(|e| Error::RpmDecode {e})
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