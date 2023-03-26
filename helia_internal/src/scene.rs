use crate::camera::Camera;
use crate::shader::{EntityUniforms, ShaderRenderPipeline};
use crate::CameraBindGroup;
use crate::EntityBindGroup;
use crate::entity::*;
use crate::prefab::*;
use crate::mesh::*;
use crate::material::*;
// ^^ should probably consider a prelude, although I do prefer this to throwing everything in the prelude
use slotmap::DenseSlotMap;

pub struct Scene {
    pub shader_render_pipeline: ShaderRenderPipeline,
    // this feels like it should be part of a shader struct
    pub camera_bind_group: CameraBindGroup,
    // this feels like renderer / context internal state
    pub entity_bind_group: EntityBindGroup,
    // todo: this is specific per shader, so we need different buffers for each shader not one for the whole scene
    // ^^ if we move methods from lib into scene impl might be able to make these fields private again
    
    pub camera: Camera,
    pub prefabs: DenseSlotMap<PrefabId, Prefab>,
    pub entity_count: usize,
}

impl Scene {
    pub fn create_prefab(&mut self, mesh: Mesh, material: Material) -> PrefabId {
        self.prefabs.insert(Prefab::new(mesh, material))
    }

    pub fn add_instance(&mut self, prefab_id: PrefabId, transform: glam::Mat4, color: wgpu::Color) {
        self.entity_count += 1;
        self.prefabs[prefab_id].entities.push(Entity { 
            transform,
            color,
        });
        // ^^ todo: return EntityId
    }

    pub fn update(&mut self, _elapsed: f32, queue: &wgpu::Queue, device: &wgpu::Device) {
        self.camera_bind_group.update(&self.camera, &queue);

        let capacity = self.entity_bind_group.entity_capacity;
        if capacity < self.entity_count as u64 {
            let mut target_capacity = 2 * capacity;
            while target_capacity < self.entity_count as u64 {
                target_capacity *= 2;
            }
            self.entity_bind_group.recreate_entity_buffer(target_capacity, device);
        }

        let mut running_offset : usize = 0;
        let entity_aligment = self.entity_bind_group.alignment;
        for prefab in self.prefabs.values() {
            for (i, entity) in prefab.entities.iter().enumerate() {
                let data = EntityUniforms {
                    model: entity.transform.to_cols_array_2d(),
                    color: [
                        entity.color.r as f32,
                        entity.color.g as f32,
                        entity.color.b as f32,
                        entity.color.a as f32,
                    ],
                };
                let offset = (i + running_offset) as u64 * entity_aligment;
                queue.write_buffer(
                    &self.entity_bind_group.buffer,
                    offset as wgpu::BufferAddress,
                    bytemuck::bytes_of(&data),
                );
            }
            running_offset += prefab.entities.len();
        }
    }
} 
