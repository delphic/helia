use crate::{material::MaterialId, mesh::MeshId, shader::EntityUniforms};

// This is really a render object at the moment
// it is also mixing the requirements of the shader (transform / color)
// with the requirements the geometry (mesh) and the specification of
// the shader (via material, currently implicit)

// Currently only applies to sprites, and has to be used with prefabs

slotmap::new_key_type! { pub struct EntityId; }

pub struct Entity {
    // local properties
    pub transform: glam::Mat4,
    pub color: wgpu::Color,
    // render details
    pub mesh: MeshId,
    pub material: MaterialId,
    pub uniform_offset: u64,
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
