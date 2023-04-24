use crate::character::*;
use crate::grid::*;
use crate::player::*;
use crate::GameResources;

use glam::*;
use helia::input::VirtualKeyCode;
use helia::material::MaterialId;
use helia::mesh::MeshId;
use helia::{entity::*, *};

pub enum BattleStage {
    PlayerMove,
    PlayerAction,
    EnemyTurn,
}

pub struct BattleState {
    players: Vec<Player>,
    dummys: Vec<Character>,
    grid: Grid,
    stage: BattleStage,
    active_player_index: usize,
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


        struct FontAtlas {
            mesh_id: MeshId,
            material_id: MaterialId,
            char_map: String,
            tile_width: u16,
            tile_height: u16,
            columns: u16,
            rows: u16,
        }

        struct TextMesh {
            text: String,
            entities: Vec<EntityId>,
        }

        impl TextMesh {
            fn new(text: String, atlas: FontAtlas, scale: f32, state: &mut State) -> Self {
                let tile_width = atlas.tile_width as f32;
                let tile_height = atlas.tile_height as f32;
                let character_width = (atlas.columns as f32).recip(); // in uv coords
                let character_height = (atlas.rows as f32).recip(); // in uv coords          

                let mut entities = Vec::new();
                let chars = text.chars();
                let chars_len = text.len() as f32;
                let offset = -tile_width * chars_len * scale / 2.0;
                // this is probably terrible practice for anything aother than ascii
                for (i, char) in chars.into_iter().enumerate() {
                    if let Some(index) = atlas.char_map.find(char) {
                        let x = (index % 22) as f32;
                        let y = (index / 22) as f32;
                        let position = Vec3::new(offset + i as f32 * tile_width * scale , 16.0, 0.0);
                        let id = state.scene.add_entity(
                            atlas.mesh_id,
                            atlas.material_id,
                            InstanceProperties::builder()
                            .with_translation(position)
                            .with_uv_offset_scale(
                                Vec2::new(x * character_width, y * character_height),
                                Vec2::new(character_width, character_height))
                                .with_scale(scale * Vec3::new(tile_width, tile_height, 1.0))
                            .build(),
                        );
                        entities.push(id);
                    }
                }

                Self {
                    text,
                    entities,
                }
            }
        }

        // Font test
        let quad_mesh = crate::utils::build_quad_mesh(1.0, 1.0, Vec2::ZERO, state);
        let mesh_id = state.resources.meshes.insert(quad_mesh);
        let material_id = resources[&"micro_font".to_string()].1;
        let atlas = FontAtlas {
            mesh_id,
            material_id,
            char_map: "ABCDEFGHIJKLMNOPQRSTUVabcdefghijklmnopqrstuvWXYZ0123456789_.,!?:; wxyz()[]{}'\"/\\|=-+*<>%".to_string(),
            tile_width: 4,
            tile_height: 6,
            columns: 22,
            rows: 4,
        };

        let text = "Hello World!".to_string();
        TextMesh::new(text, atlas, 2.0, state);

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
            stage: BattleStage::PlayerMove,
            active_player_index: 0,
        }
    }

    pub fn enter(&mut self, state: &mut State) {
        let player = &mut self.players[self.active_player_index];
        player.character.start_turn(&self.grid);
        self.grid.set_movement_highlights(&player.character, state);
    }

    pub fn update(&mut self, state: &mut State, _elapsed: f32) {
        match self.stage {
            BattleStage::PlayerMove => {
                let player = &mut self.players[self.active_player_index];
                if let Some(character_move) = player.run_move_stage(&self.grid, state) {
                    self.grid.occupancy.remove(&character_move.0);
                    self.grid.occupancy.insert(character_move.1);

                    self.grid.clear_highlights(state);
                    self.stage = BattleStage::PlayerAction;
                    // this is essentially selecting the "skip turn" ability
                    self.grid.set_highlight(player.character.position, Color::RED, state);
                }
            }
            BattleStage::PlayerAction => {
                if state.input.key_down(VirtualKeyCode::X) {
                    self.stage = BattleStage::PlayerMove;
                    let player_character = &self.players[self.active_player_index].character;
                    self.grid.set_movement_highlights(player_character, state);
                }
                if state.input.key_down(VirtualKeyCode::Z) {
                    // todo: select and perform player ability
                    // need another stage really `PlayerActionMenu`
                    // and then `PlayerAbilityTargeting { ability }`
                    // followed by a stage which executes the ability
                    // before finally moving onto the enemy turn
                    self.stage = BattleStage::EnemyTurn;
                }
            }
            BattleStage::EnemyTurn => {
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
                // todo: animate, coroutines would be nice

                // back to the players turn
                let player = &mut self.players[self.active_player_index];
                player.character.start_turn(&self.grid);
                self.grid.set_movement_highlights(&player.character, state);
                self.stage = BattleStage::PlayerMove;
            }
        }
    }
}
