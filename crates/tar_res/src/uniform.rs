use std::{marker::PhantomData, sync::Arc};

use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

use crate::WgpuInfo;

pub struct Uniform<T: NoUninit> {
    pub buff: wgpu::Buffer,
    pub data: PhantomData<T>,
}

impl<T: NoUninit> Uniform<T> {
    pub fn new(data: T, usage: String, w_info: Arc<WgpuInfo>) -> Self {
        let buff = w_info
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some((usage + "buffer").as_str()),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let uni = Self {
            buff,
            data: PhantomData,
        };

        uni.write_buffer(&w_info.queue, data);
        uni
    }

    pub fn update(&self, data: T, queue: &wgpu::Queue) {
        self.write_buffer(queue, data);
    }

    fn write_buffer(&self, queue: &wgpu::Queue, data: T) {
        queue.write_buffer(&self.buff, 0, bytemuck::cast_slice(&[data]));
    }
}
