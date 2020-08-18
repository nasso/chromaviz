mod particle;

use crate::Renderer;
use particle::ParticleRenderer;
pub use particle::ParticleSettings;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct ChromaSettings {
    pub particles: ParticleSettings,
}

pub struct Chroma {
    particle_renderer: ParticleRenderer,
}

impl Chroma {
    pub fn new(device: &wgpu::Device, settings: ChromaSettings) -> Self {
        Self {
            particle_renderer: ParticleRenderer::new(device, settings.particles),
        }
    }
}

impl Renderer for Chroma {
    fn update(&mut self, delta: Duration, freq_data: &[f32]) {
        self.particle_renderer.update(delta, freq_data);
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        dest: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Vec<wgpu::CommandBuffer> {
        self.particle_renderer.render(device, dest, width, height)
    }
}
