use std::sync::Arc;

pub struct Primitive {
    pub vertices: wgpu::Buffer,
    pub num_vertices: u32,

    pub indices: wgpu::Buffer,
    pub num_indices: u32,

    material: Arc<Material>,
    pbr_shader: Arc<PbrShader>,
}