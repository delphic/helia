use crate::camera::Camera;
use crate::shader::{EntityUniforms, ShaderRenderPipeline};
use crate::CameraBindGroup;
use crate::EntityBindGroup;
use crate::entity::*;
use crate::prefab::*;
use crate::mesh::*;
use crate::material::*;
// ^^ should probably consider a prelude, although I do prefer this to throwing everything in the prelude
use slotmap::{DenseSlotMap, SlotMap};

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
    pub render_objects: Vec<EntityId>,
    entities: SlotMap<EntityId, Entity>,
}

impl Scene {
    pub fn new(shader_render_pipeline: ShaderRenderPipeline, camera_bind_group: CameraBindGroup, entity_bind_group: EntityBindGroup) -> Self{
        Self {
            shader_render_pipeline,
            camera_bind_group,
            entity_bind_group,
            camera: Camera::default(),
            prefabs: DenseSlotMap::with_key(),
            render_objects: Vec::new(),
            entities: SlotMap::with_key(),
        }
    }

    pub fn create_prefab(&mut self, mesh: Mesh, material: Material) -> PrefabId {
        self.prefabs.insert(Prefab::new(mesh, material))
    }

    pub fn add_instance(&mut self, prefab_id: PrefabId, transform: glam::Mat4, color: wgpu::Color) -> EntityId {
        let entity_id = self.entities.insert(Entity { transform, color, mesh: None, material: None });
        self.prefabs[prefab_id].instances.push(entity_id);
        entity_id
    }

    pub fn add_entity(&mut self, transform: glam::Mat4, color: wgpu::Color, mesh: Mesh, material: Material) -> EntityId {
        let entity_id = self.entities.insert(Entity { transform, color, mesh: Some(mesh), material: Some(material) });
        self.render_objects.push(entity_id);
        entity_id
    }

    pub fn get_entity(&self, id: EntityId) -> &Entity {
        &self.entities[id]
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> &mut Entity {
        &mut self.entities[id]
    }

    pub fn update(&mut self, _elapsed: f32, queue: &wgpu::Queue, device: &wgpu::Device) {
        self.camera_bind_group.update(&self.camera, &queue);

        // todo: check if any prefab instance has had material or mesh set
        // if so, assign the other (if required) and move to entities and remove from instances

        let capacity = self.entity_bind_group.entity_capacity;
        let entity_count = self.entities.len() as u64;
        if capacity < entity_count {
            let mut target_capacity = 2 * entity_count;
            while target_capacity < entity_count {
                target_capacity *= 2;
            }
            self.entity_bind_group.recreate_entity_buffer(target_capacity, device);
        }

        let entity_aligment = self.entity_bind_group.alignment;

        // todo: move the render logic to this class so that the
        // dependency on order of evaluation between buffer creation
        // and rendering matches
        for (i, entity) in self.render_objects.iter().map(|id| &self.entities[*id]).enumerate() {
            let data = EntityUniforms {
                model: entity.transform.to_cols_array_2d(),
                color: [
                    entity.color.r as f32,
                    entity.color.g as f32,
                    entity.color.b as f32,
                    entity.color.a as f32,
                ],
            };
            let offset = i as u64 * entity_aligment;
            queue.write_buffer(
                &self.entity_bind_group.buffer,
                offset as wgpu::BufferAddress,
                bytemuck::bytes_of(&data),
            );
        }

        let mut running_offset : usize = self.render_objects.len();
        for prefab in self.prefabs.values() {
            for (i, entity) in prefab.instances.iter().map(|id| &self.entities[*id]).enumerate() {
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
            running_offset += prefab.instances.len();
        }
    }
} 
