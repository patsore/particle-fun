extern crate core;

use std::f32::consts::PI;
use std::sync::Arc;
use egui::Slider;
use egui_wgpu::{ScreenDescriptor, wgpu};
use egui_wgpu::wgpu::{InstanceDescriptor, PowerPreference, RequestAdapterOptions, TextureFormat};
use egui_wgpu::wgpu::util::DeviceExt;
use glam::Vec3;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{CursorGrabMode, Fullscreen};

use crate::egui_tools::EguiRenderer;
mod egui_tools;
mod renderer;
mod vector;
mod camera;
mod models;
mod texture;
mod state;
mod utils;
mod fluid_vec;
mod compute;

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new().unwrap();

    let builder = winit::window::WindowBuilder::new();
    let window = builder.build(&event_loop).unwrap();
    let window = Arc::new(window);
    let initial_width = 1360;
    let initial_height = 768;
    let _ = window.request_inner_size(PhysicalSize::new(initial_width, initial_height));
    window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
    window.set_maximized(true);

    let instance = egui_wgpu::wgpu::Instance::new(InstanceDescriptor::default());
    let surface = instance
        .create_surface(window.clone())
        .expect("Failed to create surface!");
    let power_pref = PowerPreference::default();
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: power_pref,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let features = wgpu::Features::POLYGON_MODE_POINT;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: features,
                required_limits: Default::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");
    let queue = Arc::new(queue);

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let selected_format = TextureFormat::Bgra8UnormSrgb;
    let swapchain_format = swapchain_capabilities
        .formats
        .iter()
        .find(|d| **d == selected_format)
        .expect("failed to select proper surface texture format!");

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: *swapchain_format,
        width: initial_width,
        height: initial_height,
        present_mode: wgpu::PresentMode::AutoVsync,
        desired_maximum_frame_latency: 0,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    let mut egui_renderer = EguiRenderer::new(&device, config.format, None, 1, &window);

    let (mut camera, camera_bind_group_layout) = crate::camera::CameraBundle::new(camera::Camera {
        pos: Vec3::new(0.0, 2.0, 3.0),
        rotation: (0.0, PI / 6.0),
        up: Vec3::Y,
        aspect_ratio: 1360.0 / 768.0,
        fov_y: 45.0,
        z_near: 0.1,
        z_far: 1.0,
    }, &device, queue.clone());

    let (camera_buffer, camera_bind_group) = camera.get_gpu_side();

    let point_buffer_rust: Vec<[f32; 4]> = (0..8_388_608).map(|i| {
        let phi = std::f32::consts::PI * (3.0 - (5.0f32).sqrt()); // Golden angle
        let y = 1.0 - (2.0 * i as f32 / 8_388_608.0); // y-coordinate from -1 to 1
        let radius = (1.0 - y * y).sqrt(); // radius at this y level
        let theta = phi * i as f32; // azimuthal angle

        let x = radius * theta.cos();
        let z = radius * theta.sin();

        [x, y, z, 1.0]
    }).collect::<Vec<_>>();

    //create a buffer that will create a force acting perpendicular to the radius of the sphere


    let point_buffer = Arc::new(device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
        label: Some("Point Buffer"),
        contents: bytemuck::cast_slice(point_buffer_rust.as_slice()),
        usage: wgpu::BufferUsages::VERTEX | egui_wgpu::wgpu::BufferUsages::STORAGE,
    }));

    let point_buffer_size = point_buffer_rust.len() as u32;
    let mut renderer = renderer::Renderer::new(
        &device,
        &config,
        &[&camera_bind_group_layout],
        camera_buffer,
        camera_bind_group,
        point_buffer.clone(),
        point_buffer_size,
    );

    let mut compute = crate::compute::Compute::new(&device, point_buffer.clone(), point_buffer_size);

    let mut scale_factor = 1.0;
    let mut process_inputs = true;
    let mut modifiers = ModifiersState::default();
    let mut v1 = 0.005_f32;
    let mut v2 = 0.50_f32;
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        camera.winit_input_helper.update(&event);

        match event {
            Event::WindowEvent { event, .. } => {
                egui_renderer.handle_input(&window, &event);
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::ModifiersChanged(new) => {
                        modifiers = new.state()
                    }
                    WindowEvent::KeyboardInput {
                        event: kb_event, ..
                    } => {
                        match kb_event.logical_key {
                            Key::Named(NamedKey::Escape) => {
                                if modifiers.shift_key() {
                                    process_inputs = true;
                                    window.set_cursor_grab(CursorGrabMode::Confined)
                                        .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                                        .unwrap();
                                } else if modifiers.alt_key() {
                                    elwt.exit();
                                } else {
                                    process_inputs = false;
                                    window.set_cursor_grab(CursorGrabMode::None).unwrap();
                                }
                            }
                            _ => {}
                        }
                    }
                    WindowEvent::ActivationTokenDone { .. } => {}
                    WindowEvent::Resized(new_size) => {
                        // Resize surface:
                        config.width = new_size.width;
                        config.height = new_size.height;
                        surface.configure(&device, &config);
                        camera.resize(&config);
                        renderer.resize(&device, &config)
                    }
                    WindowEvent::RedrawRequested => {
                        if process_inputs {
                            camera.handle_inputs();
                        }
                        let surface_texture = surface
                            .get_current_texture()
                            .expect("Failed to acquire next swap chain texture");

                        let surface_view = surface_texture
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        compute.compute(&mut encoder, &queue);
                        renderer.render(&mut encoder, &surface_view);

                        let screen_descriptor = ScreenDescriptor {
                            size_in_pixels: [config.width, config.height],
                            pixels_per_point: window.scale_factor() as f32 * scale_factor,
                        };

                        egui_renderer.draw(
                            &device,
                            &queue,
                            &mut encoder,
                            &window,
                            &surface_view,
                            screen_descriptor,
                            |ctx| {
                                egui::Window::new("")
                                    .resizable(true)
                                    .vscroll(true)
                                    .default_open(false)
                                    .show(ctx, |ui| {
                                        let v1 = Slider::new(&mut v1, 0.0..=1.0)
                                            .text("Lower bound of threshold")
                                            .drag_value_speed(0.01);
                                        ui.add(v1);

                                        let v2 = Slider::new(&mut v2, 0.0..=1.0)
                                            .text("Lower bound of threshold")
                                            .drag_value_speed(0.01);
                                        ui.add(v2);
                                    });
                            },
                        );

                        compute.inputs.c1 = v1;
                        compute.inputs.c2 = v2;


                        compute.update_inputs(&queue);

                        queue.submit(Some(encoder.finish()));
                        surface_texture.present();
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }).unwrap();
}
