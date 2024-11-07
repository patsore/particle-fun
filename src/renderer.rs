use std::sync::Arc;

use egui_wgpu::wgpu::{BindGroup, BindGroupLayout, BlendState, Buffer, Color, ColorTargetState, CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device, Face, FragmentState, FrontFace, include_wgsl, IndexFormat, LoadOp, Operations, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, StencilState, StoreOp, SurfaceConfiguration, TextureView, VertexState};
use egui_wgpu::wgpu::TextureFormat::Bgra8UnormSrgb;

use crate::models::{CloudPoint, Vertex};
use crate::vector::Vector;

pub struct Renderer {
    //Rendering
    render_pipeline: RenderPipeline,

    depth_texture: crate::texture::Texture,

    //Camera
    camera_buffer: Arc<Buffer>,
    camera_bind_group: Arc<BindGroup>,

    point_buffer: Arc<Buffer>,
    point_buffer_size: u32,
}

impl Renderer {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        bind_group_layouts: &[&BindGroupLayout],
        camera_buffer: Arc<Buffer>,
        camera_bind_group: Arc<BindGroup>,
        point_buffer: Arc<Buffer>,
        point_buffer_size: u32,
    ) -> Self {
        //+X is R, +Y is U, +Z is B
        let depth_texture = crate::texture::Texture::create_depth_texture(&device, config, "depth_texture");

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some("Vector Rendering Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &[
                        CloudPoint::desc()
                    ],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::PointList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Point,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: crate::texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: Default::default(),
                fragment: Some(
                    FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        compilation_options: Default::default(),
                        targets: &[Some(ColorTargetState {
                            format: Bgra8UnormSrgb,
                            blend: Some(BlendState::ALPHA_BLENDING),
                            write_mask: Default::default(),
                        })],
                    }
                ),
                multiview: None,
            }
        );

        Self {
            render_pipeline,
            depth_texture,

            camera_buffer,
            camera_bind_group,

            point_buffer,
            point_buffer_size,
        }
    }

    pub fn render(&self, encoder: &mut CommandEncoder, surface_view: &TextureView) {
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[
                    Some(RenderPassColorAttachment {
                        view: surface_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: StoreOp::Store,
                        },
                    })
                ],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });


            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.point_buffer.slice(..));

            render_pass.draw(0..self.point_buffer_size, 0..1);
        }
    }

    pub fn resize(&mut self, device: &Device, config: &SurfaceConfiguration) {
        self.depth_texture = crate::texture::Texture::create_depth_texture(device, config, "depth_texture");
    }
}
