use std::collections::HashMap;

use crate::camera::Camera;
use crate::entity::*;
use crate::material::*;
use crate::mesh::*;
use crate::prefab::*;
use crate::transform::Transform;
use crate::transform_hierarchy::TransformId;
use crate::transform_hierarchy::TransformHierarchy;
use crate::DrawCommand;
use crate::Resources;
use slotmap::SecondaryMap;
use slotmap::DenseSlotMap;

// ^^ should probably consider a prelude, although I do prefer this to throwing everything in the prelude

pub struct SceneEntity {
    pub visible: bool,
    pub mesh: MeshId,
    pub material: MaterialId,
    pub properties: InstanceProperties,
}

impl SceneEntity {
    pub fn new(mesh: MeshId, material: MaterialId, properties: InstanceProperties) -> Self {
        Self {
            mesh,
            material,
            visible: true,
            properties,
        }
    }
}

pub struct Scene {
    pub prefabs: DenseSlotMap<PrefabId, Prefab>,
    pub hierarchy: TransformHierarchy,
    entities: SecondaryMap<TransformId, SceneEntity>,
    render_objects: Vec<TransformId>,
    scene_graph: Vec<TransformId>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            prefabs: DenseSlotMap::with_key(),
            render_objects: Vec::new(),
            entities: SecondaryMap::new(),
            hierarchy: TransformHierarchy::new(),
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
    ) -> TransformId {
        let prefab = self.prefabs.get_mut(prefab_id).unwrap();
        let id = self
            .hierarchy
            .insert(transform, None);
        self.entities.insert(id, SceneEntity::new(prefab.mesh, prefab.material, properties));
        prefab.instances.push(id);
        id
    }

    pub fn add(
        &mut self,
        mesh: MeshId,
        material: MaterialId,
        transform: Transform,
        properties: InstanceProperties,
    ) -> TransformId {
        let id = self
            .hierarchy
            .insert(transform, None);
        self.entities.insert(id, SceneEntity::new(mesh, material, properties));
        self.render_objects.push(id);
        id
    }

    pub fn remove(&mut self, id: TransformId) {
        if let Some(index) = self.render_objects.iter().position(|x| *x == id) {
            self.render_objects.remove(index);
            self.hierarchy.remove(id);
            self.entities.remove(id);
        }
    }

    pub fn remove_instance(&mut self, prefab_id: PrefabId, id: TransformId) {
        if let Some(prefab) = self.prefabs.get_mut(prefab_id) {
            if let Some(index) = prefab.instances.iter().position(|x| *x == id) {
                prefab.instances.remove(index);
                self.entities.remove(id);
                self.hierarchy.remove(id);
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

    pub fn get(&self, id: TransformId) -> &SceneEntity {
        &self.entities[id]
    }

    // This is misleading because you could update entity.properties.world_matrix but it would have no effect
    pub fn get_mut(&mut self, id: TransformId) -> &mut SceneEntity {
        &mut self.entities[id]
    }

    /// Updates entity world matrices from hierarchy
    /// Builds ordered scene graph, including ordering based on camera depth for alpha blended objects
    pub fn update(
        &mut self,
        camera: &Camera,
        resources: &Resources
    ) {
        // Update Entity World Matrix From Hierarchy
        for (id, entity) in self.entities.iter_mut() {
            entity.properties.world_matrix = self.hierarchy.get_world_matrix(id).unwrap();
        }

        // Build list of entities by shader so we can know how many entities will need to rendered per shader
        // also allows us to add to the scene graph grouped by shader, to minimise rebinds during render pass
        let mut entities_by_shader = HashMap::new();

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
            let shader = &resources.shaders[*shader_id];
            if shader.requires_ordering {
                alpha_entities.append(entities);
            } else {
                self.scene_graph.append(entities);
            }
        }

        // All the opaque objects are in the 'graph', now add depth ordered alpha objects
        let camera_transform =
            glam::Mat4::look_at_rh(camera.eye, camera.target, glam::Vec3::Y);
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

    pub fn render(&mut self, draw_commands: &mut Vec<DrawCommand>) {
        for entity in self.scene_graph.iter().map(|id| &self.entities[*id]) {
            draw_commands.push(DrawCommand::Draw(entity.mesh, entity.material, entity.properties));
        }
    }
}
