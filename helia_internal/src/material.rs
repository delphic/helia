use crate::{texture, State};

slotmap::new_key_type! { pub struct MaterialId; }

pub struct Material {
    pub diffuse_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    diffuse_texture: texture::Texture,
}
// todo: we don't want the bind group info in the public types, but that requires us to have
// an internal representation, as we can't create a bind group until we have the texture,
// so we can't store the material layout, bind group ahead of time like we can with the other types.

impl Material {
    pub fn new(
        diffuse_texture: texture::Texture,
        state: &State,
    ) -> Self {
        let device = &state.device;
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: state.get_texture_bind_group_layout_ref(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        Self {
            diffuse_bind_group,
            diffuse_texture,
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
