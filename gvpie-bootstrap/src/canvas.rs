use std::sync::Arc;

use wgpu::util::DeviceExt;
use wgpu::{
    util, BufferUsages, CommandEncoderDescriptor, Device, Extent3d, ImageCopyBuffer, ImageCopyTexture,
    ImageDataLayout, Origin3d, Queue, Texture, TextureAspect,
};

use crate::text_cpu::CpuTextSurface;

pub struct WgpuHybridCanvas {
    device: Arc<Device>,
    queue: Arc<Queue>,
    width: u32,
    height: u32,
    cpu: CpuTextSurface,
}

impl WgpuHybridCanvas {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, width: u32, height: u32) -> Self {
        let cpu = CpuTextSurface::new(width.max(1), height.max(1));
        Self {
            device,
            queue,
            width: width.max(1),
            height: height.max(1),
            cpu,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.cpu = CpuTextSurface::new(self.width, self.height);
    }

    pub fn set_pixels(&mut self, pixels: &[u8]) {
        self.cpu.set_pixels(pixels);
    }

    pub fn present(&mut self, texture: &Texture) {
        let (w, h) = (self.width, self.height);
        let row_bytes = w * 4;
        let padded_row_bytes = ((row_bytes + 255) / 256) * 256;

        let mut staged = vec![0u8; (padded_row_bytes * h) as usize];
        let src = self.cpu.bytes();
        for row in 0..h as usize {
            let src_off = row * row_bytes as usize;
            let dst_off = row * padded_row_bytes as usize;
            staged[dst_off..dst_off + row_bytes as usize]
                .copy_from_slice(&src[src_off..src_off + row_bytes as usize]);
        }

        let staging = self.device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("hybrid-canvas-staging"),
            contents: &staged,
            usage: BufferUsages::COPY_SRC,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("hybrid-canvas-encoder"),
            });

        encoder.copy_buffer_to_texture(
            ImageCopyBuffer {
                buffer: &staging,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(h),
                },
            },
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit([encoder.finish()]);
    }
}
