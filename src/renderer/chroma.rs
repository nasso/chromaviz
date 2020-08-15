use super::Renderer;
use glam::Vec2;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChromaSettings {
    pub frequencies: u64,
    pub max_particles: u64,
    pub emit_interval: Duration,
    pub gravity: Vec2,
}

pub struct Chroma {
    pub settings: ChromaSettings,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    particles: Vec<Particle>,
    last_emit: Instant,
}

impl Chroma {
    pub fn new(device: &wgpu::Device, settings: ChromaSettings) -> Self {
        let vs_module =
            device.create_shader_module(wgpu::include_spirv!("shaders/shader.vert.spv"));

        let fs_module =
            device.create_shader_module(wgpu::include_spirv!("shaders/shader.frag.spv"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: 4 * 2, // 4 == sizeof f32
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float2],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: settings.max_particles * 2 * 4,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::MAP_WRITE,
        });

        Self {
            render_pipeline,
            bind_group,
            particles: Vec::with_capacity(settings.max_particles as usize),
            buffer,
            last_emit: Instant::now(),
            settings,
        }
    }

    fn emit_particle(&mut self, pos: impl Into<Vec2>, vel: impl Into<Vec2>) {
        self.particles.push(Particle {
            pos: pos.into(),
            vel: vel.into(),
        });
    }
}

#[async_trait::async_trait]
impl Renderer for Chroma {
    fn update(&mut self, delta: Duration, _freq_data: &[f64]) {
        if self.last_emit.elapsed() > self.settings.emit_interval {
            self.last_emit = Instant::now();

            let theta = rand::random::<f32>() * std::f32::consts::PI * 2.0;
            let force = 1.0 + rand::random::<f32>();
            self.emit_particle((0.0, 0.0), (theta.cos() * force, theta.sin() * force));
        }

        self.particles.retain(|particle| {
            particle.pos.x() > -1.0
                && particle.pos.x() < 1.0
                && particle.pos.y() > -1.0
                && particle.pos.y() < 1.0
        });

        for particle in self.particles.iter_mut() {
            particle.vel += self.settings.gravity * delta.as_secs_f32();
            particle.pos += particle.vel * delta.as_secs_f32();
        }
    }

    async fn render(&self, device: &wgpu::Device, dest: &wgpu::TextureView) -> wgpu::CommandBuffer {
        if !self.particles.is_empty() {
            let buffer_slice = self.buffer.slice(..);
            let buffer_future = buffer_slice.map_async(wgpu::MapMode::Write);

            device.poll(wgpu::Maintain::Wait);

            if let Ok(()) = buffer_future.await {
                let mut buf = buffer_slice.get_mapped_range_mut();

                for (i, particle) in self.particles.iter().enumerate() {
                    for (j, b) in particle.pos.x().to_ne_bytes().iter().enumerate() {
                        buf[8 * i + j] = *b;
                    }

                    for (j, b) in particle.pos.y().to_ne_bytes().iter().enumerate() {
                        buf[8 * i + 4 + j] = *b;
                    }
                }

                drop(buf);
                self.buffer.unmap();
            }
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: dest,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.buffer.slice(..self.particles.len() as u64 * 2 * 4));
            rpass.draw(0..4, 0..self.particles.len() as u32);
        }

        encoder.finish()
    }
}
