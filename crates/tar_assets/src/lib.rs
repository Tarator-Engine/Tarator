use std::path::Path;
use std::{io::{BufReader, Cursor}};

use tobj::LoadError;

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate serde;

mod model;


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


pub fn import_asset_from_path(file_path: &str) -> Result<AssetRef, ImportError> {

    let path = Path::new(file_path);

    match path.extension().ok_or(ImportError::NoFileExtension)?.to_str().ok_or(ImportError::NoFileExtension)? {

        "gltf" | "glb" => {
            load_gltf(path)
        }

        "obj" => {
            load_obj(path)
        }

        "png" | "jpg" => {
            load_image(path)
        }

        "mp3" | "wav" | "ogg" => {
            todo!("audio loading")
        }

        f => {return Err(ImportError::UnsupportedFileExtension(f.to_owned()))}
    }
}


pub fn load_obj(path: &Path) -> Result<AssetRef, ImportError> {
    todo!()
}

pub fn load_gltf(path: &Path) -> Result<AssetRef, ImportError> {
    todo!()
}

pub fn load_image(path: &Path) -> Result<AssetRef, ImportError> {
    todo!()
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

pub async fn load_obj_model(
    path: &Path,
) -> Result<AssetRef, ImportError> {
    let obj_text = load_string(path).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let parent_dir = path.ancestors().nth(1).ok_or(ImportError::MissingParentDirectory)?;

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&path.join(p.to_owned())).await.map_err(|e| LoadError::OpenFileFailed)?;
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

    todo!()
}