use std::collections::HashMap;

use crate::camera::Camera;
use crate::entity::*;
use crate::material::*;
use crate::mesh::*;
use crate::prefab::*;
use crate::transform::Transform;
use crate::transform_hierarchy::HierarchyId;
use crate::transform_hierarchy::TransformHierarchy;
use crate::DrawCommand;
use crate::Resources;
// ^^ should probably consider a prelude, although I do prefer this to throwing everything in the prelude
use slotmap::{DenseSlotMap, SlotMap};

pub struct Scene {
    pub camera: Camera,
    pub prefabs: DenseSlotMap<PrefabId, Prefab>,
    pub hierarchy: TransformHierarchy,
    render_objects: Vec<EntityId>,
    entities: SlotMap<EntityId, (HierarchyId, Entity)>,
    scene_graph: Vec<EntityId>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            camera: Camera::default(),
            prefabs: DenseSlotMap::with_key(),
            render_objects: Vec::new(),
            hierarchy: TransformHierarchy::new(),
            entities: SlotMap::with_key(),
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

    // Prefabs are our way of using game code to explicitly state "these all have the same mesh"
    // and hence group them together minimising the need for rebinds, however I'm not sure
    // how much it's gaining us, so tempted to remove for simplicity

    pub fn add_instance(
        &mut self,
        prefab_id: PrefabId,
        transform: Transform,
        properties: InstanceProperties,
    ) -> (EntityId, HierarchyId) {
        let prefab = self.prefabs.get_mut(prefab_id).unwrap();
        let hierarchy_id = self
            .hierarchy
            .insert(transform, None);
        let entity_id = self.entities.insert((hierarchy_id, Entity::new(prefab.mesh, prefab.material, properties)));
        prefab.instances.push(entity_id);
        (entity_id, hierarchy_id)
    }

    pub fn add_entity(
        &mut self,
        mesh: MeshId,
        material: MaterialId,
        transform: Transform,
        properties: InstanceProperties,
    ) -> (EntityId, HierarchyId) {
        let hierarchy_id = self
            .hierarchy
            .insert(transform, None);
        let entity_id = self.entities.insert((hierarchy_id, Entity::new(mesh, material, properties)));
        self.render_objects.push(entity_id);
        (entity_id, hierarchy_id)
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(index) = self.render_objects.iter().position(|x| *x == entity_id) {
            self.render_objects.remove(index);
            if let Some((hierarchy_id, _)) = self.entities.remove(entity_id) {
                self.hierarchy.remove(hierarchy_id);
            }
        }
    }

    pub fn remove_instance(&mut self, prefab_id: PrefabId, entity_id: EntityId) {
        if let Some(prefab) = self.prefabs.get_mut(prefab_id) {
            if let Some(index) = prefab.instances.iter().position(|x| *x == entity_id) {
                prefab.instances.remove(index);
                if let Some((hierarchy_id, _)) = self.entities.remove(entity_id) {
                    self.hierarchy.remove(hierarchy_id);
                }
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

    pub fn get_entity(&self, id: EntityId) -> &Entity {
        &self.entities[id].1
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> &mut Entity {
        &mut self.entities[id].1
    }

    /// Updates entity world matrices from hierarchy
    /// Builds ordered scene graph, including ordering based on camera depth for alpha blended objects
    pub fn update(
        &mut self,
        resources: &Resources
    ) {
        // Update Entity World Matrix From Hierarchy
        for (_, (hierarchy_id, entity)) in self.entities.iter_mut() {
            entity.properties.world_matrix = self.hierarchy.get_world_matrix(*hierarchy_id).unwrap();
        }

        // Build list of entities by shader so we can know how many entities will need to rendered per shader
        // also allows us to add to the scene graph grouped by shader, to minimise rebinds during render pass
        let mut entities_by_shader = HashMap::new();

        for (id, (_, entity)) in self
            .render_objects
            .iter()
            .map(|id| (id, &self.entities[*id]))
            .filter(|(_, (_, entity))| entity.visible)
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
            
            let entities = &mut entities_by_shader.get_mut(&material.shader).unwrap();
            for id in prefab
                .instances
                .iter()
                .filter(|id| self.entities[**id].1.visible)
            {
                entities.push(*id);
            }
        }
        // todo: remove the straight get_mut unwraps?

        // Enumerate over shader to entity map to build ordered scene graph
        let mut alpha_entities = Vec::new();
        self.scene_graph.clear();

        for (shader_id, entities) in entities_by_shader.iter_mut() {
            let shader = &resources.shaders[*shader_id];
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
            let world_pos_a = self.entities[*a].1
                .properties
                .world_matrix
                .transform_point3(glam::Vec3::ZERO);
            let world_pos_b = self.entities[*b].1
                .properties
                .world_matrix
                .transform_point3(glam::Vec3::ZERO);
            let a_z = camera_transform.transform_point3(world_pos_a).z;
            let b_z = camera_transform.transform_point3(world_pos_b).z;
            a_z.total_cmp(&b_z)
        });
        self.scene_graph.append(&mut alpha_entities);
    }

    pub fn render(&mut self, draw_commands: &mut Vec<DrawCommand>) {
        for entity in self.scene_graph.iter().map(|id| &self.entities[*id]) {
            draw_commands.push(DrawCommand::DrawEntity(entity.1));
        }
    }
}
