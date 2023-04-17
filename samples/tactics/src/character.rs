use std::collections::HashMap;

use glam::*;
use helia::{
    entity::{EntityId, InstanceProperties},
    material::MaterialId,
    mesh::MeshId,
    *,
};

use crate::grid::*;

pub struct Character {
    pub position: IVec2,
    pub last_position: IVec2,
    pub movement: u16,
    sprite: EntityId,
    distance_map: HashMap<IVec2, u16>,
}

impl Character {
    pub fn create_on_grid(
        position: IVec2,
        mesh_id: MeshId,
        material_id: MaterialId,
        grid: &mut Grid,
        state: &mut State,
    ) -> Self {
        let position = position.clamp(IVec2::ZERO, grid.size);
        let sprite = state.scene.add_entity(
            mesh_id,
            material_id,
            InstanceProperties::builder()
                .with_translation(grid.get_translation_for_position(position))
                .build(),
        );
        grid.occupancy.insert(position);
        Self {
            position,
            last_position: position,
            sprite,
            movement: 3,
            distance_map: HashMap::new(),
        }
    }

    pub fn start_turn(&mut self, grid: &Grid) {
        self.distance_map = grid.generate_distance_map(self);
    }

    pub fn get_reachable_positions(&self) -> &HashMap<IVec2, u16> {
        &self.distance_map
    }

    pub fn is_move_valid(&self, delta: IVec2) -> bool {
        self.distance_map.contains_key(&(self.position + delta))
    }

    pub fn perform_move(&mut self, delta: IVec2, grid: &Grid, state: &mut State) {
        self.position += delta;
        let entity = state.scene.get_entity_mut(self.sprite);
        entity.properties.transform =
            Mat4::from_translation(grid.get_translation_for_position(self.position));
    }

    pub fn flip_visual(&self, state: &mut State) {
        let entity = state.scene.get_entity_mut(self.sprite);
        entity.properties.uv_scale = Vec2::new(
            -1.0 * entity.properties.uv_scale.x,
            entity.properties.uv_scale.y,
        );
        entity.properties.uv_offset = if entity.properties.uv_scale.x.is_sign_negative() {
            Vec2::new(1.0, 0.0)
        } else {
            Vec2::ZERO
        };
    }
}
