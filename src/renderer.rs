use std::time::Duration;

pub trait Renderer {
    fn update(&mut self, delta: Duration, data: &[f32]);
    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32);
    fn render(
        &mut self,
        device: &wgpu::Device,
        dest: &wgpu::TextureView,
    ) -> Vec<wgpu::CommandBuffer>;
}
