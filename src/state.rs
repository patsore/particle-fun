use std::mem;
use std::sync::Arc;
use egui_wgpu::wgpu::{Buffer, BufferUsages, Device, Queue};
use egui_wgpu::wgpu::util::{BufferInitDescriptor, DeviceExt};
use glam::Vec3;
use crate::fluid_vec::FluidSim;
use crate::models::TestVertex;
use crate::vector::Vector;

pub(crate) struct State{
    vectors: Vec<Vector>,
    vector_buffer: Arc<Buffer>,
    vertex_buffer: Arc<Buffer>,
    vertices: Vec<TestVertex>,
    indices: Vec<u32>,
    index_buffer: Arc<Buffer>,
    queue: Arc<Queue>,
    particle_pos: Vec3,
    time: f32,
}

impl State{
    const WIDTH: i32 = 70;
    const DEPTH: i32 = 70;
    pub fn new(device: &Device, queue: Arc<Queue>) -> Self{
        let num_sides = 6; // Change this to 6 for a hexagon, or 8 for an octagon, etc.
        let shaft_radius = 0.15;
        let head_radius = 0.30;
        let shaft_height = 0.5;

        let (vertices, indices) = crate::utils::generate_arrow(num_sides, shaft_radius, head_radius, shaft_height);

        let vectors = crate::vector::create_vectors(Self::DEPTH, 20, Self::WIDTH);
        // println!("{:?}", vectors);
        let vector_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vector Buffer"),
            contents: bytemuck::cast_slice(vectors.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices.as_slice()),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices.as_slice()),
            usage: BufferUsages::INDEX,
        });

        Self{
            vectors,
            vector_buffer: Arc::new(vector_buffer),
            vertex_buffer: Arc::new(vertex_buffer),
            vertices,
            indices,
            index_buffer: Arc::new(index_buffer),
            queue,
            particle_pos: Vec3::ZERO,
            time: 0.0,
        }
    }

    pub fn update_indices(&self){
        self.queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(self.indices.as_slice()));
    }

    pub fn update_vertices(&self){
        self.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(self.vertices.as_slice()));
    }

    pub fn update_vectors(&self){
        self.queue.write_buffer(&self.vector_buffer, 0, bytemuck::cast_slice(self.vectors.as_slice()));
    }

    pub fn get_lengths(&self) -> (usize, usize){
        (self.vectors.len(), self.indices.len())
    }

    pub fn get_buffers(&self) -> (Arc<Buffer>, Arc<Buffer>, Arc<Buffer>){
        return (self.vector_buffer.clone(), self.vertex_buffer.clone(), self.index_buffer.clone());
    }
    const T: f32 = 0.001;
    const K: f32 = 10.0;

    const VISCOSITY: f32 = 0.00000001;
    const DIFFUSION_RATE: f32 = 0.01;
    const DELTA_TIME: f32 = 0.016;

    pub fn update_vectors_from_sim(&mut self, fluid_sim: &FluidSim) {
        // Convert FluidSim data to vectors
        let new_vectors = fluid_sim.to_vectors();

        // Update the State with new vectors
        self.vectors = new_vectors;

        // Update the vector buffer
        self.update_vectors();
    }

    pub fn run_sim(&mut self){
        for v in self.vectors.iter_mut(){
            let direction = Vec3::cross(v.start - self.particle_pos, Vec3::Z); // Right-hand rule for magnetic field direction
            let distance_squared = (v.start - self.particle_pos).length_squared().max(0.01); // Avoid division by zero
            let force = Self::K * 1.0 / distance_squared;
            v.magnitude = force;
            v.direction = direction.normalize();
            v.update_rotation();
        }
        self.update_vectors();

        self.time += Self::T;
        self.particle_pos = Vec3::new(0.0, 0.0, self.time.cos() * 5.0); // Wire oscillates along the Z-axis
    }



}

// pub fn run_sim(&mut self){
//     for v in self.vectors.iter_mut(){
//         let direction = v.start - self.particle_pos;
//         let length_sq = direction.length_squared();
//         let force = if length_sq < 25.0 { Self::K * 1.0 / (direction.length_squared()) } else { 0.0 };
//         v.magnitude = force;
//         v.direction = direction.normalize();
//         v.update_rotation();
//     }
//     // println!("{:?}", self.vectors);
//     self.update_vectors();
//
//     self.time += Self::T;
//     self.particle_pos = Vec3::new(self.time.cos() * 5.0, -1.5, self.time.sin() * 5.0);
// }
