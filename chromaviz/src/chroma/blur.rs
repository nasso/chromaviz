use super::render_target::{RenderTarget, RenderTargetFamily};

#[derive(Debug, Clone)]
struct Uniforms {
    frame_size: (f32, f32),
}

impl Uniforms {
    fn raw(&self) -> [u8; std::mem::size_of::<Self>()] {
        bytemuck::cast([self.frame_size.0, self.frame_size.1])
    }
}

pub enum BlurDirection {
    Horizontal,
    Vertical,
}

pub struct BlurRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    staging_belt: wgpu::util::StagingBelt,
    uniform_buf: wgpu::Buffer,
}

impl BlurRenderer {
    pub fn new(device: &wgpu::Device, family: &RenderTargetFamily) -> Self {
        let vs_module = device.create_shader_module(wgpu::include_spirv!("shaders/blur.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shaders/blur.frag.spv"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout, &family.bind_group_layout],
            push_constant_ranges: &[],
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform buffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            size: std::mem::size_of::<Uniforms>() as u64,
            mapped_at_creation: false,
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
                format: family.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let staging_belt = wgpu::util::StagingBelt::new(0x100);

        Self {
            render_pipeline,
            bind_group,
            staging_belt,
            uniform_buf,
        }
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        width: u32,
        height: u32,
    ) {
        self.staging_belt
            .write_buffer(
                encoder,
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

        self.staging_belt.finish();
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: &RenderTarget,
        dest_view: &wgpu::TextureView,
        dir: BlurDirection,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: dest_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_bind_group(1, &source.bind_group, &[]);
        rpass.draw(
            0..4,
            match dir {
                BlurDirection::Horizontal => 0..1,
                BlurDirection::Vertical => 1..2,
            },
        );
    }
}
