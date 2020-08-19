mod blur;
mod particle;
mod swap_buffer;

use crate::Renderer;
use blur::BlurRenderer;
use particle::ParticleRenderer;
pub use particle::ParticleSettings;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct ChromaSettings {
    pub particles: ParticleSettings,
}

pub struct Chroma {
    particle_renderer: ParticleRenderer,
    blur_renderer: BlurRenderer,
}

impl Chroma {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        settings: ChromaSettings,
    ) -> Self {
        Self {
            particle_renderer: ParticleRenderer::new(device, format, settings.particles),
            blur_renderer: BlurRenderer::new(device, width, height, format),
        }
    }

    pub fn update(&mut self, delta: Duration, data: &[f32]) {
        self.particle_renderer.update(delta, data);
    }
}

impl Renderer for Chroma {
    fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut commands = Vec::new();

        commands.append(&mut self.particle_renderer.resize(device, width, height));
        commands.append(&mut self.blur_renderer.resize(device, width, height));
        commands
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        dest: &wgpu::TextureView,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut commands = Vec::new();

        commands.append(
            &mut self
                .particle_renderer
                .render(device, self.blur_renderer.source()),
        );
        commands.append(&mut self.blur_renderer.render(device, dest));
        commands
    }
}
