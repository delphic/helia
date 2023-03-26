use crate::camera::Camera;
use crate::entity::*;
use crate::material::*;
use crate::mesh::*;
use crate::prefab::*;
use crate::shader::ShaderId;
use crate::shader::EntityUniforms;
use crate::EntityBindGroup;
use crate::Resources;
// ^^ should probably consider a prelude, although I do prefer this to throwing everything in the prelude
use slotmap::{DenseSlotMap, SlotMap};

pub struct Scene {
    // this feels like renderer / context internal state
    entity_bind_group: EntityBindGroup,
    // todo: this is specific per shader, so we need different buffers for each shader not one for the whole scene
    // ^^ if we move methods from lib into scene impl might be able to make these fields private again
    pub camera: Camera,
    pub prefabs: DenseSlotMap<PrefabId, Prefab>,
    pub render_objects: Vec<EntityId>,
    entities: SlotMap<EntityId, Entity>,
}

impl Scene {
    pub fn new(
        entity_bind_group: EntityBindGroup,
    ) -> Self {
        Self {
            entity_bind_group,
            camera: Camera::default(),
            prefabs: DenseSlotMap::with_key(),
            render_objects: Vec::new(),
            entities: SlotMap::with_key(),
        }
    }

    pub fn create_prefab(&mut self, mesh: MeshId, material: MaterialId) -> PrefabId {
        self.prefabs.insert(Prefab::new(mesh, material))
    }

    pub fn add_instance(
        &mut self,
        prefab_id: PrefabId,
        transform: glam::Mat4,
        color: wgpu::Color,
    ) -> EntityId {
        let entity_id = self.entities.insert(Entity {
            transform,
            color,
            mesh: None,
            material: None,
        });
        self.prefabs[prefab_id].instances.push(entity_id);
        entity_id
    }

    pub fn add_entity(
        &mut self,
        transform: glam::Mat4,
        color: wgpu::Color,
        mesh: MeshId,
        material: MaterialId,
    ) -> EntityId {
        let entity_id = self.entities.insert(Entity {
            transform,
            color,
            mesh: Some(mesh),
            material: Some(material),
        });
        self.render_objects.push(entity_id);
        entity_id
    }

    pub fn get_entity(&self, id: EntityId) -> &Entity {
        &self.entities[id]
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> &mut Entity {
        &mut self.entities[id]
    }

    pub fn update(&mut self, resources: &mut Resources, queue: &wgpu::Queue, device: &wgpu::Device) {
        // This fills the camera and entity buffers with the appropriate information
        // all shaders which will be used are updated
        // todo: should make test cases and profile updating all shaders compared to updating currently used
        // shaders

        // NOTE: This currently has dependnecy on scene.camera so it's explicitly a pre-render step for a 
        // specific camera and as such, todo: we should probably call it then instead, alternatively we could 
        // create a set of shaders which will be used from this update, and then loop through and run an update for
        // the camera prior to the render step for the specific cameras instead

        let mut shaders_updated = std::collections::HashSet::new();

        // todo: check if any prefab instance has had material or mesh set
        // if so, assign the other (if required) and move to entities and remove from instances
        // although could argue the game code should explicitly break the prefab connection rather
        // than checking every frame for something that's going to be quite rare

        let capacity = self.entity_bind_group.entity_capacity;
        let entity_count = self.entities.len() as u64;
        if capacity < entity_count {
            let mut target_capacity = 2 * entity_count;
            while target_capacity < entity_count {
                target_capacity *= 2;
            }
            self.entity_bind_group
                .recreate_entity_buffer(target_capacity, device);
        }

        let entity_aligment = self.entity_bind_group.alignment;

        for (i, entity) in self
            .render_objects
            .iter()
            .map(|id| &self.entities[*id])
            .enumerate()
        {
            if let Some(material_id) = entity.material {
                let material = &resources.materials[material_id];
                if !shaders_updated.contains(&material.shader) {
                    shaders_updated.insert(material.shader);
                    resources.shaders[material.shader].camera_bind_group.update(&self.camera, queue);
                }
            }

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

        let mut running_offset: usize = self.render_objects.len();
        for prefab in self.prefabs.values() {
            let material = &resources.materials[prefab.material];
            if !shaders_updated.contains(&material.shader) {
                shaders_updated.insert(material.shader);
                resources.shaders[material.shader].camera_bind_group.update(&self.camera, queue);
            }

            for (i, entity) in prefab
                .instances
                .iter()
                .map(|id| &self.entities[*id])
                .enumerate()
            {
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

        let mut running_offset = 0;
        let entity_aligment = self.entity_bind_group.alignment;
        let entity_bind_group = &self.entity_bind_group.bind_group;

        for (i, entity) in self
            .render_objects
            .iter()
            .map(|id| self.get_entity(*id))
            .enumerate()
        {
            if let (Some(mesh_id), Some(material_id)) = (entity.mesh, entity.material) {
                if currently_bound_material_id != Some(material_id) {
                    currently_bound_material_id = Some(material_id);

                    let material = &resources.materials[material_id];
                    if currently_bound_shader_id != Some(material.shader) {
                        currently_bound_shader_id = Some(material.shader);
                        let shader = &resources.shaders[material.shader];
                        render_pass.set_pipeline(&shader.render_pipeline);
                        render_pass.set_bind_group(0, &shader.camera_bind_group.bind_group, &[]);
                    }

                    render_pass.set_bind_group(1, &material.diffuse_bind_group, &[]);
                    // We're presumably going to share the layout for textures across shaders
                    // therefore we can and should share texture bind groups across materials
                    // only rebind when appropriate, rather than rebinding per material
                    // however should only do this is we're bothering to order the scene graph
                    // to group materials with the same textures
                }

                let mesh = &resources.meshes[mesh_id];
                if currently_bound_mesh_id != Some(mesh_id) {
                    currently_bound_mesh_id = Some(mesh_id);

                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                }

                let offset = (i + running_offset) as u64 * entity_aligment;
                render_pass.set_bind_group(2, entity_bind_group, &[offset as wgpu::DynamicOffset]);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }
        running_offset += self.render_objects.len();

        for prefab in self.prefabs.values() {
            if prefab.instances.is_empty() {
                continue;
            }

            if currently_bound_material_id != Some(prefab.material) {
                currently_bound_material_id = Some(prefab.material);

                let material = &resources.materials[prefab.material];

                if currently_bound_shader_id != Some(material.shader) {
                    currently_bound_shader_id = Some(material.shader);
                    let shader = &resources.shaders[material.shader];
                    render_pass.set_pipeline(&shader.render_pipeline);
                    render_pass.set_bind_group(0, &shader.camera_bind_group.bind_group, &[]);
                }

                render_pass.set_bind_group(1, &material.diffuse_bind_group, &[]);
            }

            let mesh = &resources.meshes[prefab.mesh];
            if currently_bound_mesh_id != Some(prefab.mesh) {
                currently_bound_mesh_id = Some(prefab.mesh);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            }

            // using uniform with offset approach of
            // https://github.com/gfx-rs/wgpu/tree/master/wgpu/examples/shadow
            for i in 0..prefab.instances.len() {
                let offset = (i + running_offset) as u64 * entity_aligment;
                render_pass.set_bind_group(2, entity_bind_group, &[offset as wgpu::DynamicOffset]);
                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
            running_offset += prefab.instances.len();
        }
    }
}
