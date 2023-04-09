pub mod material;
pub mod texture;

pub struct Model {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
    pub material: material::Material,
}
