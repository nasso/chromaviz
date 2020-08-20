mod blur;
mod particle;
mod render_target;

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
            blur_renderer: BlurRenderer::new(device, width, height, 0.5, format),
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
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.particle_renderer
            .resize(device, &mut encoder, width, height);
        self.blur_renderer.resize(device, width, height, 0.5);

        vec![encoder.finish()]
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        dest: &wgpu::TextureView,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.particle_renderer
            .render(device, &mut encoder, self.blur_renderer.source());
        self.blur_renderer.render(device, &mut encoder, dest, 1);

        vec![encoder.finish()]
    }
}
