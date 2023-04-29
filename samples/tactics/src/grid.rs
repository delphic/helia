use glam::*;
use helia::{
    entity::{EntityId, InstanceProperties},
    prefab::PrefabId,
    Color, State, transform::Transform,
};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::character::Character;

pub struct Grid {
    pub size: IVec2,
    base_offset: Vec3,
    highlights: Vec<(EntityId, IVec2)>,
    pub occupancy: HashSet<IVec2>,
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

    pub fn init(&mut self, prefab_id: PrefabId, state: &mut State) {
        let n = (self.size.x * self.size.y) as i32;
        for i in 0..n {
            let position = IVec2::new(i % self.size.x, i / self.size.x);
            let id = state.scene.add_instance(
                prefab_id,
                InstanceProperties::builder()
                    .with_transform(
                        Transform::from_position(
                        self.get_translation_for_position(position)
                            - 16.0 * Vec3::Y
                            - 32.0 * Vec3::Z,
                    )) // could sort this y offset with better anchoring and base offset
                    .with_color(Color::TRANSPARENT) // Visibility rather than transparent would be nice
                    .build(),
            );
            self.highlights.push((id, position));
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

    pub fn set_movement_highlights(&self, character: &Character, state: &mut State) {
        let reachable_positions = character.get_reachable_positions();
        for (id, highlight_pos) in self.highlights.iter() {
            let entity = state.scene.get_entity_mut(*id);
            if reachable_positions.contains_key(&highlight_pos) {
                entity.properties.color = Color {
                    r: 0.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.5,
                };
                entity.visible = true;
            } else {
                entity.visible = false;
            };
        }
    }

    pub fn set_highlight(&self, pos: IVec2, color: Color, state: &mut State) {
        if self.is_in_bounds(pos) {
            let index = (pos.x + pos.y * self.size.x) as usize;
            let id = self.highlights[index].0;
            let entity = state.scene.get_entity_mut(id);
            entity.properties.color = color;
            entity.visible = true;
        }
    }

    pub fn clear_highlights(&self, state: &mut State) {
        for (id, _) in self.highlights.iter() {
            let entity = state.scene.get_entity_mut(*id);
            entity.visible = false;
        }
    }
}
