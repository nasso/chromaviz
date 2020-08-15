use std::time::Duration;

#[async_trait::async_trait]
pub trait Renderer {
    fn update(&mut self, delta: Duration, data: &[f64]);
    async fn render(&self, device: &wgpu::Device, dest: &wgpu::TextureView) -> wgpu::CommandBuffer;
}

pub mod chroma;
