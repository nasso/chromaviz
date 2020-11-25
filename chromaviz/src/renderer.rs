pub trait Renderer {
    fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Vec<wgpu::CommandBuffer>;

    fn render(
        &mut self,
        device: &wgpu::Device,
        dest: &wgpu::TextureView,
    ) -> Vec<wgpu::CommandBuffer>;
}
