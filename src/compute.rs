use std::sync::Arc;

// use egui_wgpu::wgpu::{BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer, ComputePipelineDescriptor, Device};
// use egui_wgpu::wgpu::ComputePipeline;
use egui_wgpu::wgpu::*;
use egui_wgpu::wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct Compute {
    pub pipeline: ComputePipeline,
    pub point_buffer: Arc<Buffer>,
    pub points: u32,
    pub input_bind_group: BindGroup,

    pub inputs: Inputs,
    pub inputs_buffer: Buffer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Inputs {
    pub time: f32,
    pub iterations: u32,
    pub c1: f32,
    pub c2: f32,
}

impl Compute {
    pub fn new(device: &Device, point_buffer: Arc<Buffer>, points: u32) -> Self {
        let input_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Input bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry{
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer{
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let inputs = Inputs {
            time: 0.0,
            iterations: 8,
            c1: 0.25,
            c2: 0.5,
        };

        let inputs_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Inputs Buffer"),
            contents: bytemuck::cast_slice(&[inputs]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let velocities_buffer_rust: Vec<[f32; 4]> = (0..8_388_608).map(|i| {
            let phi = std::f32::consts::PI * (3.0 - (5.0f32).sqrt()); // Golden angle
            let y = 1.0 - (2.0 * i as f32 / 8_388_608.0); // y-coordinate from -1 to 1
            let radius = (1.0 - y * y).sqrt(); // radius at this y level
            let theta = phi * i as f32; // azimuthal angle

            let x = radius * theta.cos();
            let z = radius * theta.sin();

            // Perpendicular vector to the radius
            let perp_x = -z;
            let perp_z = x;

            [perp_x, 0.0, perp_z, 1.0]
        }).collect::<Vec<_>>();

        let velocities_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Inputs Buffer"),
            contents: bytemuck::cast_slice(velocities_buffer_rust.as_slice()),
            usage: BufferUsages::STORAGE,
            // size: point_buffer.size(),
            // mapped_at_creation: false,
        });

        let input_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Input Bind Group"),
            layout: &input_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: point_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: inputs_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: velocities_buffer.as_entire_binding(),
                }
            ],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Simulation pipeline layout"),
            bind_group_layouts: &[
                &input_bind_group_layout
            ],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("compute_shader.wgsl"));

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        Self {
            pipeline,
            point_buffer,
            input_bind_group,
            inputs,
            inputs_buffer,
            points,
        }
    }

    pub fn compute(&mut self, encoder: &mut CommandEncoder, queue: &Queue) {
        {
            self.inputs.time += 0.01;
            queue.write_buffer(&self.inputs_buffer, 0, bytemuck::cast_slice(&[self.inputs]));

            let mut compute_pass = encoder.begin_compute_pass(&Default::default());

            compute_pass.set_bind_group(0, &self.input_bind_group, &[]);
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.dispatch_workgroups((self.points as f32 / 256_f32).ceil() as u32, 1, 1);

            drop(compute_pass);
        }
    }

    pub fn update_inputs(&mut self, queue: &Queue){
        queue.write_buffer(&self.inputs_buffer, 0, bytemuck::cast_slice(&[self.inputs]));
    }
}
