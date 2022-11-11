#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate serde;

mod model;

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

pub struct FSID {
    pub id: uuid::Uuid,
    pub name: String,
}

pub fn import_gltf(path: std::path::PathBuf, instances: Vec<model::Instance>) -> Result<Vec<FSID>, ImportError> {
    let scenes = easy_gltf::load(path).map_err(|e| ImportError::Gltf { source: e })?;

    let fs_ids = vec![];

    for scene in scenes {
        for model in scene.models {
            
            let vertecies = model.vertices();

            let vertecies: Vec<model::StoreVertex> = vertecies.iter().map(|v| {
                model::StoreVertex {
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

            todo!("Import the rest of the model");
        }
    }

    Ok(fs_ids)
}