use glam::*;
use helia::{
    material::MaterialId, mesh::MeshId, Color, DrawCommand
};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{character::Character, sprite::Sprite};

pub struct Grid {
    pub size: IVec2,
    base_offset: Vec3,
    highlights: Vec<GridHighlight>,
    pub occupancy: HashSet<IVec2>,
}

pub struct GridHighlight {
    sprite: Sprite,
    grid_position: IVec2,
    visible: bool,
}

impl Grid {
    pub fn new() -> Self {
        let size = IVec2::new(12, 3);
        let base_offset = Vec3::new(-400.0, -32.0, 32.0); // dependent on bg sprite currently

        Self {
            size,
            base_offset,
            highlights: Vec::new(),
            occupancy: HashSet::new(),
        }
    }

    pub fn init(&mut self, mesh_id: MeshId, material_id: MaterialId) {
        let n = (self.size.x * self.size.y) as i32;
        for i in 0..n {
            let grid_position = IVec2::new(i % self.size.x, i / self.size.x);
            let position = self.get_translation_for_position(grid_position)
            - 16.0 * Vec3::Y
            - 32.0 * Vec3::Z;
            let sprite = Sprite {
                mesh_id,
                material_id,
                position,
                uv_scale: Vec2::ONE,
                uv_offset: Vec2::ZERO,
                color: Color::TRANSPARENT,
            };
            self.highlights.push(GridHighlight { sprite, grid_position, visible: true });
        }
    }

    pub fn is_in_bounds(&self, grid_position: IVec2) -> bool {
        grid_position.x >= 0
            && grid_position.x < self.size.x
            && grid_position.y >= 0
            && grid_position.y < self.size.y
    }

    pub fn get_translation_for_position(&self, grid_position: IVec2) -> Vec3 {
        let x = grid_position.x as f32;
        let y = grid_position.y as f32;
        self.base_offset + Vec3::new(64.0 * x + 32.0 * y, -32.0 * y, 16.0 * y)
    }

    #[allow(dead_code)]
    pub fn distance(a: IVec2, b: IVec2) -> i32 {
        (a.x - b.x).abs() + (a.y - b.y).abs()
    }

    pub fn generate_distance_map(&self, character: &Character) -> HashMap<IVec2, u16> {
        // When we want to support flying units should simply iterate over
        // grid by manhatten distance, checking occupancy
        let mut position_queue = VecDeque::new();
        let mut position_set: HashSet<IVec2> = HashSet::new();
        let mut reachable_positions: HashMap<IVec2, u16> = HashMap::new();
        position_queue.push_back((character.position, 0));
        position_set.insert(character.position);

        while let Some((position, distance)) = position_queue.pop_front() {
            if !self.occupancy.contains(&position) || distance == 0 {
                reachable_positions.insert(position, distance);

                if distance < character.movement {
                    let directions = [
                        position + IVec2::X,
                        position + IVec2::NEG_X,
                        position + IVec2::Y,
                        position + IVec2::NEG_Y,
                    ];
                    for position in directions {
                        if !position_set.contains(&position) && self.is_in_bounds(position) {
                            // ^^ note if tiles had distances of more than 1 we'd need to still
                            // consider the position despite it haven't already been previous
                            // added to the queue in case the distance was less
                            position_queue.push_back((position, distance + 1));
                            position_set.insert(position);
                        }
                    }
                }
            }
        }

        reachable_positions
    }

    pub fn set_movement_highlights(&mut self, character: &Character) {
        let reachable_positions = character.get_reachable_positions();
        for highlight in self.highlights.iter_mut() {
            if reachable_positions.contains_key(&highlight.grid_position) {
                highlight.sprite.color = Color {
                    r: 0.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.5,
                };
                highlight.visible = true;
            } else {
                highlight.sprite.color = Color::TRANSPARENT;
                highlight.visible = false;
            };
        }
    }

    pub fn set_highlight(&mut self, pos: IVec2, color: Color) {
        if self.is_in_bounds(pos) {
            let index = (pos.x + pos.y * self.size.x) as usize;
            self.highlights[index].sprite.color = color;
            self.highlights[index].visible = true;
        }
    }

    pub fn clear_highlights(&mut self) {
        for highlight in self.highlights.iter_mut() {
            highlight.visible = false;
        }
    }

    pub fn render(&self, commands: &mut Vec<DrawCommand>) {
        for highlight in self.highlights.iter() {
            if highlight.visible {
                commands.push(highlight.sprite.to_draw_command());
            }
        }
    }
}
