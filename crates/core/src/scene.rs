use std::collections::HashMap;
use std::collections::HashSet;

use crate::camera::Camera;
use crate::entity::*;
use crate::material::*;
use crate::mesh::*;
use crate::prefab::*;
use crate::shader::ShaderId;
use crate::transform::Transform;
use crate::Resources;
use glam::Mat4;
// ^^ should probably consider a prelude, although I do prefer this to throwing everything in the prelude
use slotmap::{DenseSlotMap, SecondaryMap, SlotMap};

pub struct Scene {
    pub camera: Camera,
    pub prefabs: DenseSlotMap<PrefabId, Prefab>,
    pub render_objects: Vec<EntityId>,
    hierarchy: SlotMap<EntityId, Transform>,
    entities: SecondaryMap<EntityId, Entity>,
    scene_graph: Vec<EntityId>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::default(),
            prefabs: DenseSlotMap::with_key(),
            render_objects: Vec::new(),
            hierarchy: SlotMap::with_key(),
            entities: SecondaryMap::new(),
            scene_graph: Vec::new(),
        }
    }

    pub fn create_prefab(&mut self, mesh: MeshId, material: MaterialId) -> PrefabId {
        self.prefabs.insert(Prefab::new(mesh, material))
    }

    // the fact we have the path of prefab instances and individual entities, is what
    // requires the nesting of properties, ideally this would be unnecessary, and the
    // scene graph would take care of the grouping, however until we have figured out
    // how to support custom properties going to keep it this way.

    pub fn add_instance(
        &mut self,
        prefab_id: PrefabId,
        transform: Transform,
        properties: InstanceProperties,
    ) -> EntityId {
        let prefab = self.prefabs.get_mut(prefab_id).unwrap();
        let entity_id = self
            .hierarchy
            .insert(transform);
        self.entities.insert(entity_id, Entity::new(prefab.mesh, prefab.material, properties));
        prefab.instances.push(entity_id);
        entity_id
    }

    pub fn add_entity(
        &mut self,
        mesh: MeshId,
        material: MaterialId,
        transform: Transform,
        properties: InstanceProperties,
    ) -> EntityId {
        let entity_id = self
            .hierarchy
            .insert(transform);
        self.entities.insert(entity_id, Entity::new(mesh, material, properties));
        self.render_objects.push(entity_id);
        entity_id
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(index) = self.render_objects.iter().position(|x| *x == entity_id) {
            self.render_objects.remove(index);
            self.hierarchy.remove(entity_id);
            self.entities.remove(entity_id);
        }
    }

    pub fn remove_instance(&mut self, prefab_id: PrefabId, entity_id: EntityId) {
        if let Some(prefab) = self.prefabs.get_mut(prefab_id) {
            if let Some(index) = prefab.instances.iter().position(|x| *x == entity_id) {
                prefab.instances.remove(index);
                self.hierarchy.remove(entity_id);
                self.entities.remove(entity_id);
            }
        }
    }

    pub fn clear(&mut self) {
        self.hierarchy.clear();
        self.entities.clear();
        self.prefabs.clear();
        self.render_objects.clear();
        self.scene_graph.clear();
    }

    pub fn get_entity_transform(&self, id: EntityId) -> &Transform {
        &self.hierarchy[id]
    }

    pub fn get_entity_transform_mut(&mut self, id: EntityId) -> &mut Transform {
        &mut self.hierarchy[id]
    }
    // ^^ TODO: move to heirarchy data structure

    pub fn get_entity(&self, id: EntityId) -> &Entity {
        &self.entities[id]
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> &mut Entity {
        &mut self.entities[id]
    }
    // ^^ TODO: I think what I want to do remove this get_entity / get_entity_mut 
    // and instead expose methods to submit instance data for an entity i.e. copy in 
    // instance properties, and transform data, the only question is where to manage 
    // the transform heirarchies, if transforms always updated themselves then we could
    // just calculate the world matrix when submitting.

    pub fn update(
        &mut self,
        entity_count_by_shader: &mut HashMap::<ShaderId, u64>,
        resources: &mut Resources
    ) {
        // todo: check if any prefab instance has had material or mesh not matching prefab defn
        // if so, move to entities and remove from instances
        // although could argue the game code should explicitly break the prefab connection rather
        // than checking every frame for something that's going to be quite rare

        // Build list of entities by shader so we can know how many entities will need to rendered per shader
        // also allows us to add to the scene graph grouped by shader, to minimise rebinds during render pass
        let mut entities_by_shader = HashMap::new();

        // Calculate world matrices
        let mut recalculated = HashSet::new();
        let mut pending = Vec::new();
        let mut entities = self.entities.keys().collect::<Vec<_>>();
        if let Some(id) = entities.pop() {
            pending.push(id);
        }
        while let Some(id) = pending.last() {
            let mut matrix = Mat4::IDENTITY;
            if let Some(parent) = self.hierarchy[*id].parent {
                if !recalculated.contains(&parent) {
                    pending.push(parent);
                    continue;
                } else {
                    matrix = self.entities[parent].properties.world_matrix;
                }
            }

            while let Some(id) = pending.pop() {
                recalculated.insert(id);
                matrix = self.hierarchy[id].to_local_matrix() * matrix;
                self.entities[id].properties.world_matrix = matrix;
            }

            loop {
                if let Some(id) = entities.pop() {
                    if !recalculated.contains(&id) {
                        pending.push(id);
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        for (id, entity) in self
            .render_objects
            .iter()
            .map(|id| (id, &self.entities[*id]))
            .filter(|(_, entity)| entity.visible)
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
            for id in prefab
                .instances
                .iter()
                .filter(|id| self.entities[**id].visible)
            {
                entities.push(*id);
            }
        }
        // todo: remove the straight get_mut unwraps?

        // Enumerate over shader to entity map to build ordered scene graph
        let mut alpha_entities = Vec::new();
        self.scene_graph.clear();

        for (shader_id, entities) in entities_by_shader.iter_mut() {
            let shader = &mut resources.shaders[*shader_id];
            if let Some(count) = entity_count_by_shader.get(shader_id) {
                entity_count_by_shader.insert(*shader_id, count + entities.len() as u64);
            } else {
                entity_count_by_shader.insert(*shader_id, entities.len() as u64);
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
                .properties
                .world_matrix
                .transform_point3(glam::Vec3::ZERO);
            let world_pos_b = self.entities[*b]
                .properties
                .world_matrix
                .transform_point3(glam::Vec3::ZERO);
            let a_z = camera_transform.transform_point3(world_pos_a).z;
            let b_z = camera_transform.transform_point3(world_pos_b).z;
            a_z.total_cmp(&b_z)
        });
        self.scene_graph.append(&mut alpha_entities);
    }

    pub fn append_scene_entities(&mut self, entities: &mut Vec<Entity>) {
        for entity in self.scene_graph.iter().map(|id| &self.entities[*id]) {
            entities.push(*entity);
        }
    }
}
