use std::f32::consts::PI;

use glam::{Mat3, Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

pub struct Camera {
    pub pos: Vec3,

    pub rotation: (f32, f32),

    pub up: Vec3,
    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let quat_x = Quat::from_axis_angle(Vec3::Y, self.rotation.0);
        let quat_y = Quat::from_axis_angle(Vec3::X, self.rotation.1);

        let translation = Mat4::from_translation(-self.pos);

        let view = Mat4::from_quat(quat_y * quat_x) * translation;

        let proj = Mat4::perspective_infinite_rh(self.fov_y.to_radians(), self.aspect_ratio, self.z_near);

        return proj * view;
    }

    pub fn process_mouse(&mut self, dx: f32, dy: f32) {
        self.rotation.0 += dx;
        self.rotation.1 = (dy + self.rotation.1).clamp(-PI / 2.0, PI / 2.0);
    }

    pub fn process_keyboard_input(&mut self, input: &WinitInputHelper) {
        let input_vector = Vec3::new((input.key_held(KeyCode::KeyD) as i8 - input.key_held(KeyCode::KeyA) as i8) as f32,
                                     0.0,
                                     (input.key_held(KeyCode::KeyS) as i8 - input.key_held(KeyCode::KeyW) as i8) as f32);
        let mut input_vector = Mat3::from_axis_angle(Vec3::Y, -self.rotation.0) * input_vector;
        input_vector.y = (input.key_held(KeyCode::Space) as i8 - input.key_held(KeyCode::ShiftLeft) as i8) as f32;

        self.pos += input_vector * 0.1;
    }
}