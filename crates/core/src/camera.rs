use glam::*;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);
// ^^ Technically not needed translates from OpenGL space to Metal's
// without this models centered on 0,0,0 halfway inside the clipping
// area arguably this is fine.

pub enum Projection {
    Orthographic,
    Perspective,
}

#[derive(Debug, Copy, Clone)]
pub struct OrthographicSize {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl OrthographicSize {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn from_width_height(width: f32, height: f32) -> Self {
        Self {
            left: -0.5 * width,
            right: 0.5 * width,
            bottom: -0.5 * height,
            top: 0.5 * height,
        }
    }

    pub fn from_ratio_height(height: f32, ratio: f32) -> Self {
        Self::from_width_height(ratio * height, height)
    }

    /// Create orthographic viewport from physical size ensuring integer boundary values
    /// Use for pixel perfect alignment
    pub fn from_size(size: PhysicalSize<u32>) -> Self {
        Self {
            left: (-0.5 * size.width as f32).ceil(),
            right: (0.5 * size.width as f32).ceil(),
            bottom: (-0.5 * size.height as f32).ceil(),
            top: (0.5 * size.height as f32).ceil(),
        }
    }

    /// Create orthographic viewport from physical size, scaled by a pixel ratio, ensuring integer boundary values
    /// Use for upscaled pixel perfect alignment
    pub fn from_size_scale(size: PhysicalSize<u32>, pixel_ratio: u32) -> Self {
        let scale = 0.5 * (pixel_ratio as f32).recip();
        Self {
            left: (-scale * size.width as f32).ceil(),
            right: (scale * size.width as f32).ceil(),
            bottom: (-scale * size.height as f32).ceil(),
            top: (scale * size.height as f32).ceil(),
        }
    }
}

impl Default for OrthographicSize {
    fn default() -> Self {
        Self::from_width_height(1.0, 1.0)
    }
}

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect_ratio: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub size: OrthographicSize,
    pub clear_color: wgpu::Color,
    pub projection: Projection,
    pub pixel_ratio: f32,
}
// todo: move from eye / target to position / rotation

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let scale = Mat4::from_scale(self.pixel_ratio * Vec3::ONE);
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = match self.projection {
            Projection::Perspective => {
                Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
            }
            Projection::Orthographic => Mat4::orthographic_rh(
                self.size.left,
                self.size.right,
                self.size.bottom,
                self.size.top,
                self.near,
                self.far,
            ),
            // todo: provide functions for orthographic and perspective camera create methods
        };
        OPENGL_TO_WGPU_MATRIX * proj * view * scale
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: (0.0, 0.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: 1.0,
            fov: 60.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 1000.0,
            size: OrthographicSize::default(),
            clear_color: wgpu::Color::BLACK,
            projection: Projection::Perspective,
            pixel_ratio: 1.0,
        }
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

/// Contains the bind group, its layout and the data to bind
pub struct CameraBindGroup {
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    uniform: CameraUniform,
}
// todo: a better name would be nice
// only one camera supported currently

impl CameraBindGroup {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        uniform.update_view_proj(&Camera::default());

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            layout,
            buffer,
            uniform,
            bind_group,
        }
    }

    pub fn update(&mut self, camera: &Camera, queue: &wgpu::Queue) {
        self.uniform.update_view_proj(camera);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        // ^^ Should probably be creating a separate buffer and copy it's contents
        // See just above - https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/#a-controller-for-our-camera
    }
}
