use glam::*;

use crate::{
    camera::CameraBindGroup,
    entity::{Entity, EntityBindGroup},
    texture,
};

// This is a perfectly legit Sprite Vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct EntityUniforms {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
}
// for sprite shader

impl EntityUniforms {
    pub fn write_bytes(entity: &Entity, bytes: &mut Vec<u8>) {
        let props = entity.properties;
        let data = EntityUniforms {
            model: props.transform.to_cols_array_2d(),
            color: [
                props.color.r as f32,
                props.color.g as f32,
                props.color.b as f32,
                props.color.a as f32,
            ],
            uv_offset: props.uv_offset.to_array(),
            uv_scale: props.uv_scale.to_array(),
        };
        bytes.clear();
        bytes.extend_from_slice(bytemuck::bytes_of(&data));
    }
}

pub struct Instance {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: Mat4::from_rotation_translation(self.rotation, self.position).to_cols_array_2d(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

slotmap::new_key_type! { pub struct ShaderId; }

pub struct Shader {
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: CameraBindGroup,
    pub entity_bind_group: EntityBindGroup,
    // ^^ these last two should be shared between shaders where possible
    pub requires_ordering: bool,
    bytes_delegate: fn(entity: &Entity, bytes: &mut Vec<u8>),
    bytes_buffer: Vec<u8>,
}

impl Shader {
    pub fn new(
        device: &wgpu::Device,
        module_descriptor: wgpu::ShaderModuleDescriptor,
        texture_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        alpha_blending: bool, // todo: enum, cause also pre-multiplied
        entity_uniforms_size: usize,
        to_bytes_delegate: fn(entity: &Entity, bytes: &mut Vec<u8>),
    ) -> Self {
        let camera_bind_group = CameraBindGroup::new(device);
        // Much of what's in camera.rs w.r.t. CameraBindGroup is dependent on shader implementation
        // Note: this bind group can and arguably should be shared between shaders, however waiting
        // for a use case

        let entity_bind_group = EntityBindGroup::new(entity_uniforms_size, &device);
        // Entity Bind Group is specific on shader implementation (the fact it's an individual uniform
        // in binding 0) and it's bound per entity, but this is extremely general, it is also depednent
        // upon the size of the uniforms for the specific shader, however we anticipate it may still be
        // sharable. We may also want to consider splitting between more universal (model matrix) properties
        // and material specific elements (color, uvs etc) to encourage reuse if we get to the point of sharing

        // bind group layouts order has to match the @group declarations in the shader
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group.layout,
                &entity_bind_group.layout,
                texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        // You could conceivably share pipeline layouts between shaders with similar bind group requirements

        let blend_state = if alpha_blending {
            Some(wgpu::BlendState::ALPHA_BLENDING)
        } else {
            Some(wgpu::BlendState::REPLACE)
        };

        let shader_module = device.create_shader_module(module_descriptor);
        // there is a pipeline per shader, determines how many buffers you send!
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()], //, InstanceRaw::desc() for particle systems
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: blend_state,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                // Could arguably be None for 2D
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: !alpha_blending,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            render_pipeline,
            camera_bind_group,
            entity_bind_group,
            requires_ordering: alpha_blending,
            bytes_delegate: to_bytes_delegate,
            bytes_buffer: Vec::new(),
        }
    }

    pub fn write_entity_uniforms(&mut self, entity: &mut Entity, offset: u64, queue: &wgpu::Queue) {
        // previously the writing to the queue as done as part of the delegate,
        // which avoided the use of a Vec just for returning uniform data per entity
        // however this formulation has 'cleaner' separation of responsibility. We should probably
        // profile this to see if there is significant performance impact and consider reverting
        // to the delegate doing the queue write to avoid the unnecessary shuffling with Vec.
        // The use of a delegates is to avoid requiring type information when storing the shader.
        entity.uniform_offset = offset * self.entity_bind_group.alignment;
        (self.bytes_delegate)(entity, &mut self.bytes_buffer);
        queue.write_buffer(
            &self.entity_bind_group.buffer,
            entity.uniform_offset as wgpu::BufferAddress,
            self.bytes_buffer.as_slice(),
        );
    }
}
