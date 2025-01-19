use std::collections::HashMap;

use glam::*;
use helia::{
    material::MaterialId,
    mesh::MeshId, Color,
};

use crate::{grid::*, sprite::Sprite};

pub struct Character {
    pub position: IVec2,
    pub last_position: IVec2,
    pub movement: u16,
    pub sprite: Sprite,
    distance_map: HashMap<IVec2, u16>,
}

impl Character {
    pub fn create_on_grid(
        position: IVec2,
        mesh_id: MeshId,
        material_id: MaterialId,
        grid: &mut Grid,
    ) -> Self {
        let position = position.clamp(IVec2::ZERO, grid.size);
        let sprite = Sprite { 
            mesh_id,
            material_id,
            position: grid.get_translation_for_position(position),
            uv_offset: Vec2::ZERO,
            uv_scale: Vec2::ONE,
            color: Color::WHITE,
        };
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

    pub fn perform_move(&mut self, delta: IVec2, grid: &Grid) {
        self.position += delta;
        self.sprite.position = grid.get_translation_for_position(self.position);
    }

    pub fn flip_visual(&mut self) {
        let uv_scale = self.sprite.uv_scale; 
        self.sprite.uv_scale = Vec2::new(
            -1.0 * uv_scale.x,
            uv_scale.y,
        );
        self.sprite.uv_offset = if self.sprite.uv_scale.x.is_sign_negative() {
            Vec2::new(1.0, 0.0)
        } else {
            Vec2::ZERO
        };
    }
}
