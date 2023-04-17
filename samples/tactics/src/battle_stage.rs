use crate::character::*;
use crate::grid::*;
use crate::player::*;
use crate::GameResources;

use glam::*;
use helia::{entity::*, *};

pub struct BattleState {
    players: Vec<Player>,
    dummys: Vec<Character>,
    grid: Grid,
}

impl BattleState {
    pub fn new(resources: &GameResources, state: &mut State) -> Self {
        let helia_sprite_ids = resources[&"helia".to_string()];
        let bg_sprite_ids = resources[&"bg".to_string()];
        let highlight_ids = resources[&"highlight".to_string()];
        let dummy_ids = resources[&"dummy".to_string()];

        let mut grid = Grid::new();
        let mut players = Vec::new();
        let mut dummys = Vec::new();

        let helia_character = Character::create_on_grid(
            IVec2::new(8, 1),
            helia_sprite_ids.0,
            helia_sprite_ids.1,
            &mut grid,
            state,
        );
        players.push(Player {
            character: helia_character,
            facing: IVec2::new(-1, 0),
        });

        state.scene.add_entity(
            bg_sprite_ids.0,
            bg_sprite_ids.1,
            InstanceProperties::builder()
                .with_translation(Vec3::new(0.0, 0.0, -100.0))
                .build(),
        );

        for i in 0..3 {
            let dummy_character = Character::create_on_grid(
                IVec2::new(4 + i % 2, i),
                dummy_ids.0,
                dummy_ids.1,
                &mut grid,
                state,
            );
            dummys.push(dummy_character);
        }

        let highlight_prefab = state.scene.create_prefab(highlight_ids.0, highlight_ids.1);

        grid.init(highlight_prefab, state);

        Self {
            grid,
            players,
            dummys,
        }
    }

    pub fn enter(&mut self, state: &mut State) {
        let player = &mut self.players[0]; // todo: active player
        player.character.start_turn(&self.grid);
        self.grid.update_hightlights(&player.character, state);
    }

    pub fn update(&mut self, state: &mut State, elapsed: f32) {
        let player = &mut self.players[0]; // todo: active player
        if let Some(character_move) = player.update(&self.grid, state, elapsed) {
            self.grid.occupancy.remove(&character_move.0);
            self.grid.occupancy.insert(character_move.1);

            for dummy in &mut self.dummys {
                dummy.start_turn(&self.grid);
                let delta = IVec2::new(1, 0);
                if dummy.is_move_valid(delta) {
                    dummy.perform_move(delta, &self.grid, state);
                    self.grid.occupancy.remove(&dummy.last_position);
                    self.grid.occupancy.insert(dummy.position);
                    dummy.last_position = dummy.position;

                    // flip, for fun
                    dummy.flip_visual(state);
                }
            }

            // back to the players turn
            player.character.start_turn(&self.grid);
            self.grid.update_hightlights(&player.character, state);
        }
    }
}
