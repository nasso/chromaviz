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
    let surface = wgpu::Surface::create(&window);

    let adapter = block_on(wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        },
        wgpu::BackendBit::PRIMARY,
    ))
    .unwrap();

    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    }));

    let mut renderer = Chroma::new(
        &device,
        ChromaSettings {
            max_particles: 128,
            frequencies: 32,
            emit_interval: Duration::from_millis(10),
            gravity: glam::Vec2::new(0.0, -9.81),
        },
    );

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let mut last_update_inst = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10));

        match event {
            Event::MainEventsCleared => {
                if last_update_inst.elapsed() >= Duration::from_millis(16) {
                    window.request_redraw();
                    renderer.update(last_update_inst.elapsed(), &[0.5; 32]);
                    last_update_inst = Instant::now();

                    *control_flow =
                        ControlFlow::WaitUntil(last_update_inst + Duration::from_millis(16));
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(..) => {
                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");

                let commands = block_on(renderer.render(&device, &frame.view));

                queue.submit(&[commands]);
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("ChromaViz")
        .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
        .build(&event_loop)
        .unwrap();

    run(event_loop, window);
}
