use crate::OPENGL_TO_WGPU_MATRIX;
use glam::*;
use wgpu::util::DeviceExt;

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

#[repr(C)] // Required for rust to store data in correct format for shaders
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)] // so we can store in a buffer
pub struct CameraUniform {
    // bytemuck requires 4x4 f32 array rather than a Mat4
    view_proj: [[f32; 4]; 4],
}
// Needing to make new structs for each uniform is tiresome, wonder if grayolson's lib might be more helpful than bytemuck

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().to_cols_array_2d();
    }
}

pub struct CameraRenderInfo {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    uniform: CameraUniform,
}
// todo: a better name would be nice
// only one camera supported currently

impl CameraRenderInfo {
    pub fn new(device: &wgpu::Device, camera: Option<&Camera>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let mut uniform = CameraUniform::new();
        if let Some(camera) = camera {
            uniform.update_view_proj(&camera);
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            bind_group_layout,
            buffer,
            uniform,
            bind_group,
        }
    }

    pub fn update(&mut self, camera: &Camera, queue: &mut wgpu::Queue) {
        self.uniform.update_view_proj(camera);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        // ^^ Should probably be creating a separate buffer and copy it's contents
        // See just above - https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/#a-controller-for-our-camera
    }
}
