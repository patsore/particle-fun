use std::mem;

use egui_wgpu::wgpu::{BufferAddress, vertex_attr_array, VertexBufferLayout, VertexStepMode};
use glam::{Mat3, Vec3};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vector {
    pub(crate) start: Vec3,
    pub(crate) direction: Vec3,
    pub(crate) magnitude: f32,
    pub(crate) rotation: Mat3,
}

impl Vector {
    pub(crate) const DESC: VertexBufferLayout<'static> =
        VertexBufferLayout {
            array_stride: mem::size_of::<Vector>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Float32x3,
                1 => Float32x3,
                2 => Float32,
                3 => Float32x3,
                4 => Float32x3,
                5 => Float32x3,
            ],
        };

    pub fn new(start: Vec3, direction: Vec3, magnitude: f32) -> Self {
        Self {
            start,
            direction,
            magnitude,
            rotation: Self::calculate_rotation_matrix(direction),
        }
    }

    pub fn update_rotation(&mut self) {
        self.rotation = Self::calculate_rotation_matrix(self.direction);
    }

    fn calculate_rotation_matrix(direction: Vec3) -> Mat3 {
        let up = Vec3::new(0.0, 1.0, 0.0);

        if direction.abs_diff_eq(up, f32::EPSILON) {
            // If direction is parallel to up, return identity matrix (no rotation needed)
            return Mat3::IDENTITY;
        } else if direction.abs_diff_eq(-up, f32::EPSILON) {
            // If direction is antiparallel to up, return 180-degree rotation matrix around x-axis
            return Mat3::from_rotation_x(std::f32::consts::PI);
        }

        let rotation_axis = up.cross(direction);
        let rotation_angle = up.dot(direction).acos();
        Mat3::from_axis_angle(rotation_axis.normalize(), rotation_angle)
    }
}

pub fn create_vectors(x: i32, y: i32, z: i32) -> Vec<Vector>{
    let mut vectors = vec![];
    for x in -x/2..x/2{
        if y <= 1{
            for z in -z/2..z/2{
                vectors.push(Vector::new(Vec3::new(x as f32, 0f32, z as f32), Vec3::X, 1.0))
            }
        }else{
        for y in -y/2..y/2{
            for z in -z/2..z/2{
                vectors.push(Vector::new(Vec3::new(x as f32, y as f32, z as f32), Vec3::X, 1.0))
            }
        }
        }
    };
    vectors
}

const K: f32 = 10.0;

pub fn animate_vectors(vectors: &mut [Vector], time: f32) {
    // Skip the first 6 vectors (axis vectors)
    for (i, vector) in vectors.iter_mut().enumerate().skip(6) {
        // Calculate new position and direction
        let angle = time + i as f32 * 0.5; // Slightly offset each vector's rotation for variety
        let radius = 2.5;

        // Update position (circular motion in the XZ plane)
        let new_position = Vec3::new(
            radius * angle.cos(),
            0.0,
            radius * angle.sin(),
        );

        // Direction remains pointing upwards
        let new_direction = Vec3::new(0.0, 1.0, 0.0) + new_position;
        vector.direction = new_direction.normalize();
        vector.magnitude =  10.0 * time.sin();
        vector.update_rotation();
    }
}
