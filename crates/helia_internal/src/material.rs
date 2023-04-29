use crate::{shader::ShaderId, texture::TextureId, State};

slotmap::new_key_type! { pub struct MaterialId; }

pub struct Material {
    pub shader: ShaderId,
    pub texture: TextureId,
    pub diffuse_bind_group: wgpu::BindGroup,
}
// todo: we don't want the bind group info in the public types, but that requires us to have
// an internal representation, as we can't create a bind group until we have the texture,
// so we can't store the material layout, bind group ahead of time like we can with the other types.
// It's tricky though, we need the particular texture to create the bind group, but the layout is technically
// specific to the shader although we don't support a different laytout right now

// A note on per instance vs per material properties
// In Fury you could mix and match per material and per instance properties for the same material
// However a choice needs to be made ahead of time per material, and properties must be grouped
// accordingly, you could in theory use the same shader with the same properties uniform be per material
// or per entity as long as the properties you wish to mix and match were separated by binding group,
// but due to how we currently define and execute the binding code, you can not do this without
// changing engine code.

// If we wish to make it so this is possible we will want to be able to track binding group rebinds
// and order our scene graph to minimise texture group rebinds (which are still presumably more expense),
// (if wgpu does internally prevents unnecessary rebinds we simply need to order the scene graph appropriately)
// we should investigate this before we attempt to extend our existing scene structure which does track
// the current bindings, although only at the mesh and material level (where as really it should be per bind group)
impl Material {
    pub fn new(shader: ShaderId, texture: TextureId, state: &State) -> Self {
        let id = texture;
        let texture = &state.resources.textures[id];
        // todo: would be nice to provide an overload that takes a enum of BuildInShaders
        // and that we keep track of enum -> ShaderId, that way the user only has to worry about
        // shader ids for shaders they've created
        let device = &state.device;
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: state.get_texture_bind_group_layout_ref(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        Self {
            shader,
            texture: id,
            diffuse_bind_group,
        }
    }

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        // todo: probably want to expose filtering at some point
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }
}
