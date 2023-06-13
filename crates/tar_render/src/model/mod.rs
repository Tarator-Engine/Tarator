use scr_types::prims::Mat4;
use tar_shader::shader;
use wgpu::util::DeviceExt;

pub mod material;
pub mod texture;

pub struct Model {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub num_vertices: u32,
    pub num_indices: Option<u32>,
    pub material: material::Material,
    pub instances: Vec<shader::Instance>,
    pub instance_buffer: wgpu::Buffer,
}

impl Model {
    pub fn from_stored(
        stored: tar_res::model::Model,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
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

        let material =
            material::Material::from_stored(stored.material, device, queue, target_format);

        let instances = vec![shader::Instance {
            model_matrix_0: Mat4::IDENTITY.x_axis,
            model_matrix_1: Mat4::IDENTITY.y_axis,
            model_matrix_2: Mat4::IDENTITY.z_axis,
            model_matrix_3: Mat4::IDENTITY.w_axis,
        }];

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(instances.as_slice()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_vertices,
            num_indices,
            material,
            instances,
            instance_buffer,
        }
    }

    pub fn update_transform(&mut self, t: &scr_types::prelude::Transform, queue: &wgpu::Queue) {
        let mat = glam::Mat4::from_scale_rotation_translation(t.scale.into(), t.rot, t.pos.into());

        let instance = vec![shader::Instance {
            model_matrix_0: mat.x_axis,
            model_matrix_1: mat.y_axis,
            model_matrix_2: mat.z_axis,
            model_matrix_3: mat.w_axis,
        }];

        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(instance.as_slice()),
        );
    }

    pub fn render<'rps>(&'rps self, render_pass: &mut wgpu::RenderPass<'rps>) {
        render_pass.set_pipeline(&self.material.pipeline);
        self.material.bind_group.set(render_pass);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        if let Some(i_buff) = &self.index_buffer {
            render_pass.set_index_buffer(i_buff.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices.unwrap(), 0, 0..1);
        } else {
            render_pass.draw(0..self.num_vertices, 0..1);
        }
    }
}
