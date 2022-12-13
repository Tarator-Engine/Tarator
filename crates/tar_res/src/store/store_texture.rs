use std::fmt::Debug;
use std::path::Path;

use gltf::image::Source;
use serde::{Deserialize, Serialize};

use crate::{scene::ImportData, Error, Result, ASSET_PATH};

use image::ImageFormat;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum TextureType {
    base_color,
    metallic_roughness,
    normal,
    occlusion,
    emissive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreTexture {
    pub index: usize,
    pub path: String,
}

impl StoreTexture {
    pub fn from_gltf(
        g_texture: &gltf::Texture<'_>,
        imp: &ImportData,
        base_path: &Path,
        tex_ty: TextureType,
        object_name: &String,
        material_name: &String,
    ) -> Result<Self> {
        let buffers = &imp.buffers;

        let g_img = g_texture.source();

        let img = match g_img.source() {
            Source::View { view, mime_type } => {
                let parent_buffer_data = &buffers[view.buffer().index()].0;
                let begin = view.offset();
                let end = begin + view.length();
                let data = &parent_buffer_data[begin..end];
                match mime_type {
                    "image/jpeg" => image::load_from_memory_with_format(data, ImageFormat::Jpeg),
                    "image/png" => image::load_from_memory_with_format(data, ImageFormat::Png),
                    _ => {
                        return Err(Error::NotSupported(format!(
                            "unsupported image type (image: {}, mime_type: {})",
                            g_img.index(),
                            mime_type
                        )))
                    }
                }
            }
            Source::Uri { uri, mime_type } => {
                if uri.starts_with("data:") {
                    let encoded = uri.split(',').nth(1).unwrap();
                    let data = base64::decode(&encoded).unwrap();
                    let mime_type = if let Some(ty) = mime_type {
                        ty
                    } else {
                        uri.split(',')
                            .nth(0)
                            .unwrap()
                            .split(':')
                            .nth(1)
                            .unwrap()
                            .split(';')
                            .nth(0)
                            .unwrap()
                    };

                    match mime_type {
                        "image/jpeg" => {
                            image::load_from_memory_with_format(&data, ImageFormat::Jpeg)
                        }
                        "image/png" => image::load_from_memory_with_format(&data, ImageFormat::Png),
                        _ => {
                            return Err(Error::NotSupported(format!(
                                "unsupported image type (image: {}, mime_type: {})",
                                g_img.index(),
                                mime_type
                            )))
                        }
                    }
                } else if let Some(mime_type) = mime_type {
                    let path = base_path
                        .parent()
                        .unwrap_or_else(|| Path::new("./"))
                        .join(uri);
                    let file = std::fs::File::open(path).unwrap();
                    let reader = std::io::BufReader::new(file);
                    match mime_type {
                        "image/jpeg" => image::load(reader, ImageFormat::Jpeg),
                        "image/png" => image::load(reader, ImageFormat::Png),
                        _ => {
                            return Err(Error::NotSupported(format!(
                                "unsupported image type (image: {}, mime_type: {})",
                                g_img.index(),
                                mime_type
                            )))
                        }
                    }
                } else {
                    let path = base_path
                        .parent()
                        .unwrap_or_else(|| Path::new("./"))
                        .join(uri);
                    image::open(path)
                }
            }
        }?;
        let dir = format!("{ASSET_PATH}{object_name}/");
        let path = format!(
            "{dir}{}-{}-{}.png",
            material_name,
            g_texture.name().map(|s| s.into()).unwrap_or("texture"),
            format!("{tex_ty:?}")
        );

        // println!("saving to {}", path);

        std::fs::create_dir_all(dir)?;

        img.save(path.clone())?;

        Ok(Self {
            index: g_texture.index(),
            path,
        })
    }
}
