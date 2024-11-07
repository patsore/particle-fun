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
