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
    pub distance_map: Option<HashMap<IVec2, u16>>,
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
            distance_map: None,
        }
    }

    pub fn update_distance_map(&mut self, grid: &Grid) {
        self.distance_map = Some(grid.generate_distance_map(self));
    }

    pub fn is_move_valid(&self, grid: &Grid, delta: IVec2) -> bool {
        let target_position = self.position + delta;
        if let Some(map) = &self.distance_map {
            return map.contains_key(&target_position);
        }

        grid.is_in_bounds(target_position)
            && (target_position == self.last_position || !grid.occupancy.contains(&target_position))
            && Grid::distance(target_position, self.last_position) <= self.movement as i32
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
