use wgpu::util::DeviceExt;

pub mod material;
pub mod texture;

pub struct Model {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub num_vertices: u32,
    pub num_indices: Option<u32>,
    pub material: material::Material,
}

impl Model {
    pub fn from_stored(
        stored: tar_res::model::Model,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(stored.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (index_buffer, num_indices) = if let Some(i) = stored.indices {
            (
                Some(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(i.as_slice()),
                        usage: wgpu::BufferUsages::INDEX,
                    }),
                ),
                Some(i.len() as u32),
            )
        } else {
            (None, None)
        };

        let num_vertices = stored.vertices.len() as u32;

        let material = material::Material::from_stored(stored.material, device, queue);

        Self {
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
            material,
        }
    }
}
