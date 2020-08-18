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
    pub fn new(device: &wgpu::Device, width: u32, height: u32, settings: ChromaSettings) -> Self {
        Self {
            particle_renderer: ParticleRenderer::new(device, width, height, settings.particles),
        }
    }
}

impl Renderer for Chroma {
    fn update(&mut self, delta: Duration, freq_data: &[f32]) {
        self.particle_renderer.update(delta, freq_data);
    }

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.particle_renderer.resize(device, width, height);
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, dest: &wgpu::TextureView) {
        self.particle_renderer.render(device, queue, dest);
    }
}
