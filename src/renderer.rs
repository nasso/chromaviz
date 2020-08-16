use std::time::Duration;

pub trait Renderer {
    fn update(&mut self, delta: Duration, data: &[f64]);
    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);
    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, dest: &wgpu::TextureView);
}
