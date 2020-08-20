mod blur;
mod compositor;
mod particle;
mod render_target;

use crate::Renderer;
use blur::{BlurDirection, BlurRenderer};
use compositor::Compositor;
use particle::ParticleRenderer;
pub use particle::ParticleSettings;
use render_target::{RenderTarget, RenderTargetFamily};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct ChromaSettings {
    pub particles: ParticleSettings,
    pub decay: f64,
}

pub struct Chroma {
    pub settings: ChromaSettings,
    render_target_family: RenderTargetFamily,
    particle_renderer: ParticleRenderer,
    blur_renderer: BlurRenderer,
    compositor: Compositor,
    low_res_targets: (RenderTarget, RenderTarget),
    particles_target: RenderTarget,
    accumulator: RenderTarget,
}

impl Chroma {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        settings: ChromaSettings,
    ) -> Self {
        let render_target_family = RenderTargetFamily::new(device, format);
        let particle_renderer = ParticleRenderer::new(device, &render_target_family);
        let blur_renderer = BlurRenderer::new(device, &render_target_family);
        let compositor = Compositor::new(device, &render_target_family);

        Self {
            low_res_targets: (
                render_target_family.create_target(device, width / 2, height / 2),
                render_target_family.create_target(device, width / 2, height / 2),
            ),
            particles_target: render_target_family.create_target(device, width, height),
            accumulator: render_target_family.create_target(device, width, height),
            settings,
            render_target_family,
            particle_renderer,
            blur_renderer,
            compositor,
        }
    }

    pub fn update(&mut self, delta: Duration, data: &[f32]) {
        self.particle_renderer
            .update(delta, data, &self.settings.particles);
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
        self.blur_renderer
            .resize(device, &mut encoder, width / 2, height / 2);

        self.low_res_targets = (
            self.render_target_family
                .create_target(device, width / 2, height / 2),
            self.render_target_family
                .create_target(device, width / 2, height / 2),
        );
        self.particles_target = self
            .render_target_family
            .create_target(device, width, height);
        self.accumulator = self
            .render_target_family
            .create_target(device, width, height);

        vec![encoder.finish()]
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        dest: &wgpu::TextureView,
    ) -> Vec<wgpu::CommandBuffer> {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.blur_renderer.render(
            &mut encoder,
            &self.accumulator,
            &self.low_res_targets.0.view,
            BlurDirection::Horizontal,
        );
        self.blur_renderer.render(
            &mut encoder,
            &self.low_res_targets.0,
            &self.low_res_targets.1.view,
            BlurDirection::Vertical,
        );

        self.compositor.render(
            &mut encoder,
            &self.low_res_targets.1,
            &self.accumulator.view,
            0.0,
        );

        self.particle_renderer.render(
            device,
            &mut encoder,
            &self.particles_target.view,
            &self.settings.particles,
        );

        self.compositor.render(
            &mut encoder,
            &self.particles_target,
            &self.accumulator.view,
            self.settings.decay,
        );

        self.compositor
            .render(&mut encoder, &self.accumulator, &dest, 0.0);

        vec![encoder.finish()]
    }
}
