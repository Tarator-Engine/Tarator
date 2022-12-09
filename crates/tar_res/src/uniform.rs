use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::WgpuInfo;

pub struct Uniform<T: NoUninit> {
    pub buff: wgpu::Buffer,
    data: T,
}

impl<T: NoUninit> Uniform<T> {
    pub fn new(data: T, usage: String, w_info: &WgpuInfo) -> Self {
        let buff = w_info.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some((usage+"buffer").as_str()),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut uni = Self { buff, data };

        uni.write_buffer(&w_info.queue);
        uni
    }

    pub fn update(&mut self, data: T, queue: &wgpu::Queue) {
        self.data = data;
        self.write_buffer(queue);
    }

    fn write_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buff,
            0, 
            bytemuck::cast_slice(&[self.data]),
        );
    }
}