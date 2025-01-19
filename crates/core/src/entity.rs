

use glam::{Mat4, Vec2};

use crate::{material::MaterialId, mesh::MeshId, shader::EntityUniforms, transform::Transform};

// This is really a render object at the moment
// it is also mixing the requirements of the shader (transform / color)
// with the requirements the geometry (mesh) and the specification of
// the shader (via material, currently implicit)

// Currently storing requirements of all supported shaders, eventually
// we'll need the ability to retrieve information modularly if we want
// the game to be able to extend the properties a shader can act upon
// and if want to avoid properties that have no effect for certain entities

pub struct InstancePropertiesBuilder {
    properties: InstanceProperties,
}

impl InstancePropertiesBuilder {
    pub fn new() -> Self {
        Self {
            properties: InstanceProperties::default(),
        }
    }

    pub fn build(&self) -> InstanceProperties {
        self.properties
    }

    pub fn with_color(&mut self, color: wgpu::Color) -> &mut Self {
        self.properties.color = color;
        self
    }

    pub fn with_matrix(&mut self, matrix: Mat4) -> &mut Self {
        self.properties.world_matrix = matrix;
        self
    }

    pub fn with_uv_offset_scale(&mut self, uv_offset: Vec2, uv_scale: Vec2) -> &mut Self {
        self.properties.uv_offset = uv_offset;
        self.properties.uv_scale = uv_scale;
        self
    }

    pub fn with_uv_offset(&mut self, uv_offset: Vec2) -> &mut Self {
        self.properties.uv_offset = uv_offset;
        self
    }

    pub fn with_uv_scale(&mut self, uv_scale: Vec2) -> &mut Self {
        self.properties.uv_scale = uv_scale;
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct InstanceProperties {
    pub world_matrix: Mat4,
    pub color: wgpu::Color,
    pub uv_offset: Vec2,
    pub uv_scale: Vec2,
}

impl Default for InstanceProperties {
    fn default() -> Self {
        Self {
            world_matrix: Mat4::IDENTITY,
            color: wgpu::Color::WHITE,
            uv_offset: Vec2::ZERO,
            uv_scale: Vec2::ONE,
        }
    }
}

impl InstanceProperties {
    pub fn builder() -> InstancePropertiesBuilder {
        InstancePropertiesBuilder::new()
    }

    pub fn from_transform(transform: Transform) -> Self {
        Self {
            world_matrix: transform.into(),
            color: wgpu::Color::WHITE,
            uv_offset: Vec2::ZERO,
            uv_scale: Vec2::ONE,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EntityDrawInstruction {
    pub mesh: MeshId,
    pub material: MaterialId,
    pub uniform_offset: u64,
    pub instance: InstanceProperties,
}

impl EntityDrawInstruction {
    pub fn new(mesh: MeshId, material: MaterialId, instance: InstanceProperties) -> Self {
        Self {
            mesh,
            material,
            uniform_offset: 0,
            instance,
        }
    }
}

pub struct EntityBindGroup {
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffer: wgpu::Buffer,
    pub alignment: wgpu::BufferAddress,
    pub entity_capacity: u64,
}

impl EntityBindGroup {
    pub fn new(entity_uniforms_size: usize, device: &wgpu::Device) -> Self {
        let entity_uniforms_size = entity_uniforms_size as wgpu::BufferAddress;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(entity_uniforms_size),
                },
                count: None,
            }],
            label: None,
        });

        let alignment = wgpu::util::align_to(
            entity_uniforms_size,
            device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress,
        );

        const INITIAL_ENTITY_CAPACITY: u64 = 32;
        let buffer = Self::create_buffer(INITIAL_ENTITY_CAPACITY, alignment, device);
        let bind_group = Self::create_bind_group(&layout, &buffer, device);

        Self {
            layout,
            bind_group,
            buffer,
            alignment,
            entity_capacity: INITIAL_ENTITY_CAPACITY,
        }
    }

    pub fn recreate_entity_buffer(&mut self, capacity: u64, device: &wgpu::Device) {
        self.entity_capacity = capacity;
        self.buffer = Self::create_buffer(self.entity_capacity, self.alignment, device);
        self.bind_group = Self::create_bind_group(&self.layout, &self.buffer, device);
    }

    fn create_buffer(
        entity_capacity: u64,
        alignment: wgpu::BufferAddress,
        device: &wgpu::Device,
    ) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: entity_capacity * alignment,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_bind_group(
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
        device: &wgpu::Device,
    ) -> wgpu::BindGroup {
        let entity_uniforms_size = std::mem::size_of::<EntityUniforms>() as wgpu::BufferAddress;
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(entity_uniforms_size),
                }),
            }],
            label: None,
        })
    }
}
