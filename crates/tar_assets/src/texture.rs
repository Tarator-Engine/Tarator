use std::path::Path;

use crate::scene::ImportData;

pub struct Texture {
    pub index: usize,
    pub name: Option<String>,

    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub tex_coord: u32,
}

impl Texture {
    pub fn from_gltf(g_texture: &gltf::Texture<'_>, tex_coord: u32, imp: &ImportData, base_path: &Path) -> Texture {
        let buffers = &imp.buffers;
        todo!("wgpu texture importing")
    }
}