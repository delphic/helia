use crate::character::*;
use crate::grid::*;
use crate::player::*;
use crate::GameResources;

use glam::*;
use helia::input::VirtualKeyCode;
use helia::material::MaterialId;
use helia::mesh::MeshId;
use helia::{entity::*, *};

#[derive(Clone)]
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
    font: FontAtlas,
    entities: Vec<(EntityId, Vec3)>,
    position: Vec3,
    scale: f32,
}

impl TextMesh {
    fn new(text: String, position: Vec3, font: FontAtlas, scale: f32, state: &mut State) -> Self {
        let tile_width = font.tile_width as f32;
        let tile_height = font.tile_height as f32;
        let character_width = (font.columns as f32).recip(); // in uv coords
        let character_height = (font.rows as f32).recip(); // in uv coords

        let mut entities = Vec::new();
        let chars = text.chars();
        let chars_len = text.len() as f32;
        let offset = -tile_width * chars_len * scale / 2.0;
        // this is probably terrible practice for anything aother than ascii
        for (i, char) in chars.into_iter().enumerate() {
            if let Some(index) = font.char_map.find(char) {
                let x = (index % 22) as f32;
                let y = (index / 22) as f32;
                let entity_position =
                    position + Vec3::new(offset + i as f32 * tile_width * scale, 0.0, 0.0);
                let id = state.scene.add_entity(
                    font.mesh_id,
                    font.material_id,
                    InstanceProperties::builder()
                        .with_translation(entity_position)
                        .with_uv_offset_scale(
                            Vec2::new(x * character_width, y * character_height),
                            Vec2::new(character_width, character_height),
                        )
                        .with_scale(scale * Vec3::new(tile_width, tile_height, 1.0))
                        .build(),
                );
                entities.push((id, Vec3::ZERO));
            }
        }

        Self {
            text,
            entities,
            font,
            position: Vec3::new(0.0, 16.0, 0.0),
            scale,
        }
    }

    fn calculate_entity_position(&self, index: usize) -> Vec3 {
        let character_width = self.font.tile_width as f32 * self.scale;
        let offset = -character_width * self.text.len() as f32 / 2.0;
        self.position + Vec3::new(offset + index as f32 * character_width, 0.0, 0.0)
    }

    #[allow(dead_code)]
    pub fn translate(&mut self, position: Vec3, state: &mut State) {
        self.position = position;
        for (i, (entity_id, offset)) in self.entities.iter().enumerate() {
            let entity = state.scene.get_entity_mut(*entity_id);
            let (scale, rotation, _) = entity.properties.transform.to_scale_rotation_translation();

            entity.properties.transform = Mat4::from_scale_rotation_translation(
                scale,
                rotation,
                self.calculate_entity_position(i) + *offset,
            )
        }
    }

    #[allow(dead_code)]
    pub fn offset_char(&mut self, index: usize, offset: Vec3, state: &mut State) {
        // set individual character offset from default position
        if index < self.entities.len() {
            let (id, prev_offset) = self.entities[index];
            let entity = state.scene.get_entity_mut(id);
            let (scale, rotation, translation) =
                entity.properties.transform.to_scale_rotation_translation();
            let delta = offset - prev_offset;
            entity.properties.transform =
                Mat4::from_scale_rotation_translation(scale, rotation, translation + delta);
            self.entities[index] = (id, offset);
        }
    }
}

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
    text_mesh: TextMesh,
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
        let text_mesh = TextMesh::new(text, Vec3::new(0.0, 16.0, 0.0), atlas, 2.0, state);

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
            text_mesh,
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
                    self.grid
                        .set_highlight(player.character.position, Color::RED, state);
                    self.text_mesh.translate(Vec3::new(-16.0, 16.0, 0.0), state);
                    for i in 0..self.text_mesh.text.len() {
                        self.text_mesh.offset_char(
                            i,
                            Vec3::new(0.0, (i as f32 / 2.0).sin() * 4.0, 0.0),
                            state,
                        );
                    }
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
                self.text_mesh.translate(Vec3::new(0.0, 16.0, 0.0), state);
                for i in 0..self.text_mesh.text.len() {
                    self.text_mesh
                        .offset_char(i, Vec3::new(0.0, 0.0, 0.0), state);
                }
            }
        }
    }
}
