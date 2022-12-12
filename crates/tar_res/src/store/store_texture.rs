use std::path::Path;

use gltf::image::Source;
use serde::{Deserialize, Serialize};

use crate::{scene::ImportData, Error, Result};

use image::ImageFormat;

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreTexture {
    pub index: usize,
    pub path: String,
}

impl StoreTexture {
    pub fn from_gltf(
        g_texture: &gltf::Texture<'_>,
        tex_coord: u32,
        imp: &ImportData,
        base_path: &Path,
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
        let path = format!("res_int/{}", g_texture.name());

        img.save(path);

        todo!()
    }
}
