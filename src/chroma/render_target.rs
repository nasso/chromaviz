pub struct RenderTarget {
    pub width: u32,
    pub height: u32,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
}

impl RenderTarget {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        Self {
            width,
            height,
            texture,
            view,
            bind_group,
        }
    }
}

pub struct SwapBufferPair {
    pub source: RenderTarget,
    pub dest: RenderTarget,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub format: wgpu::TextureFormat,
}

impl SwapBufferPair {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                    },
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            source: RenderTarget::new(device, &bind_group_layout, &sampler, width, height, format),
            dest: RenderTarget::new(device, &bind_group_layout, &sampler, width, height, format),
            bind_group_layout,
            sampler,
            format,
        }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.source, &mut self.dest);
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.source = RenderTarget::new(
            device,
            &self.bind_group_layout,
            &self.sampler,
            width,
            height,
            self.format,
        );

        self.dest = RenderTarget::new(
            device,
            &self.bind_group_layout,
            &self.sampler,
            width,
            height,
            self.format,
        );
    }
}
