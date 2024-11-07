use std::sync::Arc;

use egui_wgpu::wgpu;
use egui_wgpu::wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ShaderStages, SurfaceConfiguration};
use egui_wgpu::wgpu::util::{BufferInitDescriptor, DeviceExt};
use glam::Vec3;
use winit_input_helper::WinitInputHelper;

pub use camera::Camera;

pub mod camera;

pub(crate) struct CameraBundle {
    camera: Camera,

    camera_buffer: Arc<wgpu::Buffer>,
    camera_bind_group: Arc<wgpu::BindGroup>,

    queue: Arc<wgpu::Queue>,

    pub winit_input_helper: WinitInputHelper,
}

impl CameraBundle {
    pub fn new(camera: Camera, device: &wgpu::Device, queue: Arc<wgpu::Queue>) -> (Self, BindGroupLayout) {
        // let camera = Camera {
        //     pos: Vec3::new(0.0, 1.0, 0.0),
        //     rotation: (0.0, 0.0),
        //     up: Vec3::Y,
        //     aspect_ratio: 1360.0 / 768.0,
        //     fov_y: 45.0,
        //     z_near: 0.1,
        //     z_far: 1.0,
        // };

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera.build_view_projection_matrix()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Camera Bind Group Layout"),
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        (Self {
            camera,
            camera_buffer: Arc::new(camera_buffer),
            camera_bind_group: Arc::new(camera_bind_group),
            queue,
            winit_input_helper: Default::default(),
        }, camera_bind_group_layout)
    }

    fn update(&self) {
        let view_proj = self.camera.build_view_projection_matrix();
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[view_proj]),
        );
    }

    pub fn get_gpu_side(&self) -> (Arc<Buffer>, Arc<BindGroup>){
        return (self.camera_buffer.clone(), self.camera_bind_group.clone());
    }

    fn process_keyboard_input(&mut self) {
        self.camera.process_keyboard_input(&self.winit_input_helper);
        self.update();
    }

    fn process_mouse(&mut self){
        let (dx, dy) = self.winit_input_helper.mouse_diff();
        self.camera.process_mouse(dx * 0.001, dy * 0.001);
        self.update();
    }

    pub fn handle_inputs(&mut self){
        self.process_keyboard_input();
        self.process_mouse();
    }

    pub fn resize(&mut self, config: &SurfaceConfiguration){
        self.camera.aspect_ratio = config.width as f32 / config.height as f32;
        self.update();
    }
}