use std::collections::HashMap;

use crate::camera::Camera;
use crate::entity::*;
use crate::material::*;
use crate::mesh::*;
use crate::prefab::*;
use crate::shader::ShaderId;
use crate::Resources;
// ^^ should probably consider a prelude, although I do prefer this to throwing everything in the prelude
use slotmap::{DenseSlotMap, SlotMap};

pub struct Scene {
    pub camera: Camera,
    pub prefabs: DenseSlotMap<PrefabId, Prefab>,
    pub render_objects: Vec<EntityId>,
    entities: SlotMap<EntityId, Entity>,
    scene_graph: Vec<EntityId>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::default(),
            prefabs: DenseSlotMap::with_key(),
            render_objects: Vec::new(),
            entities: SlotMap::with_key(),
            scene_graph: Vec::new(),
        }
    }

    pub fn create_prefab(&mut self, mesh: MeshId, material: MaterialId) -> PrefabId {
        self.prefabs.insert(Prefab::new(mesh, material))
    }

    // todo: having the scene properties take each potential entity property as arguments is not scalable,
    // need to either take an entity directly or some kind of properties object which locals and then these
    // methods control the addition of mesh / material to the actual entity, I think the latter is probably 
    // preferable as otherwise the prefab concept loses meaning.

    pub fn add_instance(
        &mut self,
        prefab_id: PrefabId,
        transform: glam::Mat4,
    ) -> EntityId {
        let prefab = self.prefabs.get_mut(prefab_id).unwrap();
        let entity_id = self.entities.insert(Entity::with_transform(prefab.mesh, prefab.material, transform));
        prefab.instances.push(entity_id);
        entity_id
    }

    pub fn add_entity(
        &mut self,
        transform: glam::Mat4,
        mesh: MeshId,
        material: MaterialId,
    ) -> EntityId {
        let entity_id = self.entities.insert(Entity::with_transform(mesh, material, transform));
        self.render_objects.push(entity_id);
        entity_id
    }

    pub fn get_entity(&self, id: EntityId) -> &Entity {
        &self.entities[id]
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> &mut Entity {
        &mut self.entities[id]
    }

    pub fn update(
        &mut self,
        resources: &mut Resources,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
    ) {
        // This fills the camera and entity buffers with the appropriate information
        // all shaders which will be used are updated

        // todo: check if any prefab instance has had material or mesh not matching prefab defn
        // if so, move to entities and remove from instances
        // although could argue the game code should explicitly break the prefab connection rather
        // than checking every frame for something that's going to be quite rare

        // Build list of entities by shader so we can build the uniform buffers appropraitely
        // Arguably we could just iterate over all entities, if the shader were to keep track of
        // it's current uniform offset... would probably be faster than building a hashmap.
        let mut entities_by_shader = HashMap::new();

        for (id, entity) in self
            .render_objects
            .iter()
            .map(|id| (id, &self.entities[*id]))
        {
            let material = &resources.materials[entity.material];
            if !entities_by_shader.contains_key(&material.shader) {
                entities_by_shader.insert(material.shader, Vec::new());
            }
            entities_by_shader
                .get_mut(&material.shader)
                .unwrap()
                .push(*id);
        }

        for prefab in self.prefabs.values() {
            let material = &resources.materials[prefab.material];
            if !entities_by_shader.contains_key(&material.shader) {
                entities_by_shader.insert(material.shader, Vec::new());
                resources.shaders[material.shader]
                    .camera_bind_group
                    .update(&self.camera, queue);
            }
            // NOTE: does not support mutating the material id of prefab instances
            // but it does read them directly so you could easily cause an issue by changing
            // a prefab instance material id to a one using a shader which was not otherwise
            // used in the scene. We could consider separating MeshId / MaterialId store from
            // the data we do expect the game to mutate (e.g. transform).

            // Also by just adding entities individually to the very basic scene graph
            // we've lost the ability to check bindings once per prefab, however we had not
            // profiled that so we should not lament its loss. If we wanted to see if it was an
            // improvement, then we would need an entity buffer per prefab
            let entities = &mut entities_by_shader.get_mut(&material.shader).unwrap();
            for id in prefab.instances.iter() {
                entities.push(*id);
            }
        }
        // todo: remove the straight get_mut unwraps?

        // Enumerate over shader to entity map and build entity buffers and ordered scene graph
        let mut alpha_entities = Vec::new();
        self.scene_graph.clear();

        // NOTE: This currently has dependnecy on scene.camera so it's explicitly a pre-render step for a
        // specific camera and as such, todo: we should probably call it then instead, alternatively we could
        // create a set of shaders which will be used from this update, and then loop through and run an update for
        // the camera prior to the render step for the specific cameras instead
        for (shader_id, entities) in entities_by_shader.iter_mut() {
            let shader = &mut resources.shaders[*shader_id];

            shader.camera_bind_group.update(&self.camera, queue);

            let capacity = shader.entity_bind_group.entity_capacity;
            let entity_count = entities.len() as u64;
            if capacity < entity_count {
                let mut target_capacity = 2 * entity_count;
                while target_capacity < entity_count {
                    target_capacity *= 2;
                }
                shader
                    .entity_bind_group
                    .recreate_entity_buffer(target_capacity, device);
            }

            for (i, id) in entities.iter().enumerate() {
                let entity = self.entities.get_mut(*id).unwrap();
                shader.write_entity_uniforms(entity, i as u64, queue);
            }

            if shader.requires_ordering {
                alpha_entities.append(entities);
            } else {
                self.scene_graph.append(entities);
            }
        }

        // All the opaque objects are in the 'graph', now add depth ordered alpha objects
        let camera_transform =
            glam::Mat4::look_at_rh(self.camera.eye, self.camera.target, glam::Vec3::Y);
        alpha_entities.sort_by(|a, b| {
            // This quite possibly works because transform_point results in -translation
            // and then we're sorting from front to back, rather than back to front
            let world_pos_a = self.entities[*a]
                .transform
                .transform_point3(glam::Vec3::ZERO);
            let world_pos_b = self.entities[*b]
                .transform
                .transform_point3(glam::Vec3::ZERO);
            let a_z = camera_transform.transform_point3(world_pos_a).z;
            let b_z = camera_transform.transform_point3(world_pos_b).z;
            a_z.total_cmp(&b_z)
        });
        self.scene_graph.append(&mut alpha_entities);
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        resources: &Resources,
    ) {
        let camera = &self.camera;

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[
                // This is what @location(0) in fragment shader targets
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(camera.clear_color),
                        store: true,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let mut currently_bound_shader_id: Option<ShaderId> = None;
        let mut currently_bound_mesh_id: Option<MeshId> = None;
        let mut currently_bound_material_id: Option<MaterialId> = None;

        for entity in self.scene_graph.iter().map(|id| &self.entities[*id]) {
            let mesh = &resources.meshes[entity.mesh];
            let material = &resources.materials[entity.material];
            let shader = &resources.shaders[material.shader];

            let entity_bind_group = &shader.entity_bind_group.bind_group;

            if currently_bound_material_id != Some(entity.material) {
                currently_bound_material_id = Some(entity.material);

                if currently_bound_shader_id != Some(material.shader) {
                    currently_bound_shader_id = Some(material.shader);
                    render_pass.set_pipeline(&shader.render_pipeline);
                    render_pass.set_bind_group(0, &shader.camera_bind_group.bind_group, &[]);
                }

                render_pass.set_bind_group(2, &material.diffuse_bind_group, &[]);
                // We're presumably going to share the layout for textures across shaders
                // therefore we can and should share texture bind groups across materials
                // only rebind when appropriate, rather than rebinding per material
                // however should only do this is we're bothering to order the scene graph
                // to group materials with the same textures
            }

            if currently_bound_mesh_id != Some(entity.mesh) {
                currently_bound_mesh_id = Some(entity.mesh);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            }

            // using uniform with offset approach of
            // https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples/shadow
            render_pass.set_bind_group(
                1,
                entity_bind_group,
                &[entity.uniform_offset as wgpu::DynamicOffset],
            );
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
