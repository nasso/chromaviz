use crate::Renderer;
use glam::Vec2;
use rand::distributions::{Distribution, Uniform as UniformDistribution};
use std::time::Duration;
use wgpu::util::DeviceExt;

#[derive(Debug, Clone)]
struct Uniforms {
    frame_size: (f32, f32),
}

impl Uniforms {
    fn raw(&self) -> [u8; std::mem::size_of::<Self>()] {
        bytemuck::cast([self.frame_size.0, self.frame_size.1])
    }
}

#[derive(Debug, Clone)]
struct Particle {
    init_pos: Vec2,
    init_vel: Vec2,
    age: Duration,
    lifetime: Duration,
    pub size: f32,
}

impl Particle {
    pub fn pos(&self, g: Vec2) -> Vec2 {
        let t = self.age.as_secs_f32();

        0.5 * g * t * t + self.init_vel * t + self.init_pos
    }
}

struct ParticleSystem {
    pub max_particles: usize,
    particles: Vec<Particle>,
}

impl ParticleSystem {
    fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
        }
    }

    fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    fn count(&self) -> usize {
        self.particles.len()
    }

    fn particles(&self) -> impl Iterator<Item = &Particle> {
        self.particles.iter()
    }

    fn emit_particle(&mut self, particle: Particle) {
        if self.particles.len() + 1 < self.max_particles {
            self.particles.push(particle);
        }
    }

    fn update(&mut self, delta: Duration) {
        self.particles
            .retain(|particle| particle.age < particle.lifetime);

        for particle in self.particles.iter_mut() {
            particle.age += delta;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChromaSettings {
    pub gravity: Vec2,
    pub frequencies: u64,
    pub frequencies_spread: f32,
    pub max_particles: u64,
    pub particles_per_second: u64,
    pub angular_spread: f32,
    pub velocity_spread: f32,
    pub size_range: std::ops::Range<f32>,
}

pub struct Chroma {
    settings: ChromaSettings,
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    staging_belt: wgpu::util::StagingBelt,
    particle_buffer: wgpu::Buffer,
    uniform_buf: wgpu::Buffer,
    particle_system: ParticleSystem,
    time_since_last_emit: Duration,
}

impl Chroma {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, settings: ChromaSettings) -> Self {
        let vs_module =
            device.create_shader_module(wgpu::include_spirv!("shaders/shader.vert.spv"));

        let fs_module =
            device.create_shader_module(wgpu::include_spirv!("shaders/shader.frag.spv"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: bytemuck::cast_slice(&[width as f32, height as f32]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
            }],
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
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<f32>() as u64 * 3,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float3],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            size: settings.max_particles * std::mem::size_of::<f32>() as u64 * 3,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let staging_belt = wgpu::util::StagingBelt::new(0x100, device);

        Self {
            particle_system: ParticleSystem::new(settings.max_particles as usize),
            time_since_last_emit: Duration::from_secs(0),
            staging_belt,
            particle_buffer,
            uniform_buf,
            render_pipeline,
            bind_group,
            settings,
        }
    }

    fn gen_particles(&mut self, delta: Duration) {
        let mut rng = rand::thread_rng();
        let freq_dist: UniformDistribution<u64> = (0..self.settings.frequencies).into();
        let spread_dist: UniformDistribution<f32> = (-0.5..0.5).into();
        let size_dist: UniformDistribution<f32> = self.settings.size_range.clone().into();
        let period = Duration::from_secs_f64(1.0 / self.settings.particles_per_second as f64);

        self.time_since_last_emit += delta;

        let new_count = self.time_since_last_emit.as_nanos() / period.as_nanos();
        self.time_since_last_emit -= period.mul_f64(new_count as f64);

        // spawn new ones
        for i in 0..new_count {
            let freq = {
                let freq =
                    freq_dist.sample(&mut rng) as f32 / (self.settings.frequencies - 1) as f32;
                let spread = spread_dist.sample(&mut rng) * self.settings.frequencies_spread;

                freq + spread / (self.settings.frequencies - 1) as f32
            };
            let newborn_age = delta - period.mul_f64(i as f64);

            let angle =
                (90.0 + spread_dist.sample(&mut rng) * self.settings.angular_spread).to_radians();
            let velocity = self.velocity_for(freq, self.settings.gravity)
                + spread_dist.sample(&mut rng) * self.settings.velocity_spread;

            let init_vel = (angle.cos() * velocity, angle.sin() * velocity).into();

            self.particle_system.emit_particle(Particle {
                init_pos: (freq, 0.0).into(),
                age: newborn_age,
                init_vel,
                lifetime: Duration::from_secs_f32(
                    (-init_vel.y() / self.settings.gravity.y()).max(0.0),
                ),
                size: size_dist.sample(&mut rng),
            });
        }
    }

    fn velocity_for(&self, freq: f32, g: Vec2) -> f32 {
        let target = (freq * 20.0).sin() * 0.1 + 0.5;

        // U = m * g * y
        // K = mv² / 2
        //
        // Problem: what should be the initial velocity to reach height H?
        //
        // at y = H:
        //   > U_top = m * g * H
        //   > K_top = 0
        //
        // at y = 0:
        //   > U_bot = 0
        //   > K_bot = U_top = m * g * H     # energy conservation!
        //
        // K_bot = mv² / 2
        // U_top = mgH
        //
        // mv² / 2 = mgH
        // v² / 2 = gH
        // v² = 2gH
        // v = sqrt(2gH)
        (2.0 * g.y().abs() * target).sqrt()
    }
}

impl Renderer for Chroma {
    fn update(&mut self, delta: Duration, _freq_data: &[f64]) {
        // update the particle generators
        self.gen_particles(delta);

        // update the particle system
        self.particle_system.update(delta);
    }

    fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.staging_belt
            .write_buffer(
                &self.uniform_buf,
                0,
                wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64).unwrap(),
                device,
            )
            .copy_from_slice(
                &Uniforms {
                    frame_size: (width as f32, height as f32),
                }
                .raw(),
            );
    }

    fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, dest: &wgpu::TextureView) {
        if !self.particle_system.is_empty() {
            let mut buf = self.staging_belt.write_buffer(
                &self.particle_buffer,
                0,
                wgpu::BufferSize::new(
                    (self.particle_system.count() * 3 * 4) as wgpu::BufferAddress,
                )
                .unwrap(),
                device,
            );

            for (i, particle) in self.particle_system.particles().enumerate() {
                let pos = particle.pos(self.settings.gravity);
                let addr = 12 * i;

                buf[addr + 0..addr + 4].copy_from_slice(&pos.x().to_ne_bytes());
                buf[addr + 4..addr + 8].copy_from_slice(&pos.y().to_ne_bytes());
                buf[addr + 8..addr + 12].copy_from_slice(
                    &(particle.size
                        * (1.0
                            - (particle.age.as_secs_f32() / particle.lifetime.as_secs_f32())
                                .powi(2))
                        .max(0.0))
                    .to_ne_bytes(),
                );
            }
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let particle_count = self.particle_system.count();
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
            rpass.set_vertex_buffer(
                0,
                self.particle_buffer.slice(..particle_count as u64 * 3 * 4),
            );
            rpass.draw(0..4, 0..particle_count as u32);
        }

        queue.submit(vec![self.staging_belt.flush(device), encoder.finish()]);
    }
}
