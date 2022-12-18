use crate::material::{PbrMaterial, PerFrameData};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}
impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![8 => Float32x4, 9 => Float32x4, 10 => Float32x4, 11 => Float32x4, 12 => Float32x3, 13 => Float32x3, 14 => Float32x3];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
pub struct Primitive {
    pub vertices: wgpu::Buffer,
    pub num_vertices: u32,

    pub indices: wgpu::Buffer,
    pub num_indices: u32,

    pub instances: wgpu::Buffer,
    pub num_instances: u32,

    pub material: PbrMaterial,
}

impl Primitive {
    pub fn update_per_frame(&mut self, data: &PerFrameData, queue: &wgpu::Queue) {
        self.material.update_per_frame(data, queue)
    }

    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        // TODO!: determine if shader+material already active to reduce work...

        self.material.set_pipeline(render_pass);

        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.slice(..));
        render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
        self.material.set_bind_groups(render_pass);
        render_pass.draw(0..self.num_vertices, 0..1);
    }
}
