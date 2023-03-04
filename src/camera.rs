use crate::OPENGL_TO_WGPU_MATRIX;
use glam::*;

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub clear_color: wgpu::Color,
}
// todo: move from eye / target to position / rotation

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
