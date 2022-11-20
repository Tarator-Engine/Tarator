use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::WgpuInfo;

pub struct Uniform<T: NoUninit> {
    buff: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    data: T,
}

impl<T: NoUninit> Uniform<T> {
    pub fn new(data: T, bind_group: wgpu::BindGroup, usage: String, w_info: &WgpuInfo) -> Self {
        let buff = w_info.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some((usage+"buffer").as_str()),
            contents: bytemuck::cast_slice(&[data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mut uni = Self { buff, bind_group, data };

        uni.write_buffer(w_info);
        uni
    }

    pub fn update(&mut self, data: T, w_info: &WgpuInfo) {
        self.data = data;
        self.write_buffer(w_info);
    }

    fn write_buffer(&mut self, w_info: &WgpuInfo) {
        w_info.queue.write_buffer(
            &self.buff,
            0, 
            bytemuck::cast_slice(&[self.data]),
        );
    }
}