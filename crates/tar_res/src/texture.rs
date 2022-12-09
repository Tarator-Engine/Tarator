use std::num::NonZeroU32;
use std::path::Path;

use gltf::image::Source;

use crate::{WgpuInfo, Error};
use crate::{scene::ImportData, Result};

use image::ImageFormat;

use image::DynamicImage::*;

pub struct Texture {
    pub index: usize,
    pub name: Option<String>,

    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub tex_coord: u32,
}

impl Texture {
    pub fn from_gltf(g_texture: &gltf::Texture<'_>, tex_coord: u32, imp: &ImportData, base_path: &Path, w_info: &WgpuInfo) -> Result<Texture> {
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
                    _ => panic!("unsupported image type (image: {}, mime_type: {})",
                        g_img.index(), mime_type),
                }
            },
            Source::Uri { uri, mime_type } => {
                if uri.starts_with("data:") {
                    let encoded = uri.split(',').nth(1).unwrap();
                    let data = base64::decode(&encoded).unwrap();
                    let mime_type = if let Some(ty) = mime_type {
                        ty
                    } else {
                        uri.split(',')
                            .nth(0).unwrap()
                            .split(':')
                            .nth(1).unwrap()
                            .split(';')
                            .nth(0).unwrap()
                    };

                    match mime_type {
                        "image/jpeg" => image::load_from_memory_with_format(&data, ImageFormat::Jpeg),
                        "image/png" => image::load_from_memory_with_format(&data, ImageFormat::Png),
                        _ => panic!("unsupported image type (image: {}, mime_type: {})",
                            g_img.index(), mime_type),
                    }
                }
                else if let Some(mime_type) = mime_type {
                    let path = base_path.parent().unwrap_or_else(|| Path::new("./")).join(uri);
                    let file = std::fs::File::open(path).unwrap();
                    let reader = std::io::BufReader::new(file);
                    match mime_type {
                        "image/jpeg" => image::load(reader, ImageFormat::Jpeg),
                        "image/png" => image::load(reader, ImageFormat::Png),
                        _ => panic!("unsupported image type (image: {}, mime_type: {})",
                            g_img.index(), mime_type),
                    }
                }
                else {
                    let path = base_path.parent().unwrap_or_else(||Path::new("./")).join(uri);
                    image::open(path)
                }
            }
        }?;

        let dims = (img.width(), img.height());

        let (format, data_layout, data): (_, _, Vec<u8>) = match img {
            ImageLuma8(d) => (
                wgpu::TextureFormat::R8Unorm, // TODO: confirm if these are correct
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(dims.0),
                    rows_per_image: NonZeroU32::new(dims.1)
                },
                d.to_vec()
            ),
            ImageLumaA8(d) => (wgpu::TextureFormat::Rg8Unorm,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(2* dims.0),
                    rows_per_image: NonZeroU32::new(dims.1)
                },
                d.to_vec()
            ),
            ImageRgb8(d) => (wgpu::TextureFormat::Rgba8Unorm,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4* dims.0),
                    rows_per_image: NonZeroU32::new(dims.1)
                },
                d.to_vec()
            ),
            ImageRgba8(d) => (wgpu::TextureFormat::Rgba8Unorm,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4* dims.0),
                    rows_per_image: NonZeroU32::new(dims.1)
                },
                d.to_vec()
            ),
            _ => return Err(Error::NotSupported("image formats with pixel parts that are not 8 bit".to_owned()))
        };

        let size = wgpu::Extent3d {
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };

        let texture = w_info.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::all(),
        });

        w_info.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO
            }, 
            &data,
            data_layout, 
            size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = w_info.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });


        Ok(Self {
            index: g_texture.index(),
            name: g_texture.name().map(|s| s.into()),
            texture,
            view,
            sampler,
            tex_coord,
        })
    }
}