use std::time::{Duration, Instant};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use chromaviz::prelude::*;

use futures::executor::block_on;

fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
    }))
    .unwrap();

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
            limits: wgpu::Limits::default(),
            shader_validation: true,
        },
        None,
    ))
    .unwrap();

    let mut renderer = Chroma::new(
        &device,
        size.width,
        size.height,
        ChromaSettings {
            particles: ParticleSettings {
                gravity: (0.0, -2.0).into(),
                frequencies: 32,
                frequencies_spread: 1.0,
                max_particles: 10000,
                particles_per_second: 2000,
                angular_spread: 5.0,
                velocity_spread: 0.1,
                size_range: 4.0..6.0,
            },
        },
    );

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Immediate,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let mut last_update_inst = Instant::now();
    let start_inst = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                if last_update_inst.elapsed() >= Duration::from_millis(16) {
                    let t = start_inst.elapsed().as_secs_f32();
                    let phase = t * 4.0;
                    let freq_data: Vec<f32> = (0..32)
                        .map(|f| (f as f32 + phase).sin() * 0.1 + 0.5)
                        .collect();
                    renderer.update(last_update_inst.elapsed(), &freq_data);

                    let frame = match swap_chain.get_current_frame() {
                        Ok(frame) => frame,
                        Err(_) => {
                            swap_chain = device.create_swap_chain(&surface, &sc_desc);
                            swap_chain
                                .get_current_frame()
                                .expect("Failed to get next swapchain texture!")
                        }
                    };

                    queue.submit(renderer.render(&device, &frame.output.view));

                    last_update_inst = Instant::now();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
                renderer.resize(&device, size.width, size.height);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("ChromaViz")
        .with_inner_size(winit::dpi::PhysicalSize::new(640, 480))
        .build(&event_loop)
        .unwrap();

    run(event_loop, window);
}
