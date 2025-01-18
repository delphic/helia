use crate::character::*;
use crate::grid::*;
use crate::player::*;
use crate::text_mesh::*;
use crate::GameResources;
use crate::sprite::Sprite;

use glam::*;
use helia::input::KeyCode;
use helia::*;

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
    background: Sprite,
}

impl BattleState {
    pub fn new(resources: &GameResources, state: &mut State) -> Self {
        let helia_sprite_ids = resources.get_pair(&"helia".to_string()).unwrap();
        let bg_sprite_ids = resources.get_pair(&"bg".to_string()).unwrap();
        let highlight_ids = resources.get_pair(&"highlight".to_string()).unwrap();
        let dummy_ids = resources.get_pair(&"dummy".to_string()).unwrap();

        let mut grid = Grid::new();
        let mut players = Vec::new();
        let mut dummys = Vec::new();

        let helia_character = Character::create_on_grid(
            IVec2::new(8, 1),
            helia_sprite_ids.0,
            helia_sprite_ids.1,
            &mut grid,
        );
        players.push(Player {
            character: helia_character,
            facing: IVec2::new(-1, 0),
        });

        let background = Sprite {
            mesh_id: bg_sprite_ids.0,
            material_id: bg_sprite_ids.1,
            position: Vec3::new(0.0, 0.0, -100.0),
            scale: Vec3::ONE,
            uv_offset: Vec2::ZERO,
            uv_scale: Vec2::ONE,
            color: Color::WHITE,
        };

        // Font test
        let mini_atlas = resources.fonts[&"mini".to_string()].clone();

        let text = "Helia Tactics".to_string();

        let text_mesh = TextMesh::builder(
            text.clone(),
            Vec3::new(0.0, state.camera.size.top, 0.0),
            mini_atlas,
        )
        .with_alignment(TextAlignment::Center)
        .with_vertical_alignment(VerticalAlignment::Top)
        .build();

        for i in 0..3 {
            let dummy_character = Character::create_on_grid(
                IVec2::new(4 + i % 2, i),
                dummy_ids.0,
                dummy_ids.1,
                &mut grid,
            );
            dummys.push(dummy_character);
        }

        grid.init(highlight_ids.0, highlight_ids.1);

        Self {
            grid,
            players,
            dummys,
            stage: BattleStage::PlayerMove,
            active_player_index: 0,
            text_mesh,
            background,
        }
    }

    pub fn enter(&mut self) {
        let player = &mut self.players[self.active_player_index];
        player.character.start_turn(&self.grid);
        self.grid.set_movement_highlights(&player.character);
    }

    pub fn update(&mut self, state: &mut State, _elapsed: f32) {
        match self.stage {
            BattleStage::PlayerMove => {
                let player = &mut self.players[self.active_player_index];
                if let Some(character_move) = player.run_move_stage(&self.grid, &state.input) {
                    self.grid.occupancy.remove(&character_move.0);
                    self.grid.occupancy.insert(character_move.1);

                    self.grid.clear_highlights();
                    self.stage = BattleStage::PlayerAction;
                    // this is essentially selecting the "skip turn" ability
                    self.grid
                        .set_highlight(player.character.position, Color::RED);
                    self.text_mesh.set_text("Testing Testing".to_string());
                    self.text_mesh.translate(Vec3::new(-16.0, 16.0, 0.0));
                    for i in 0..self.text_mesh.text.len() {
                        self.text_mesh.offset_char(
                            i,
                            Vec3::new(0.0, (i as f32 / 2.0).sin() * 4.0, 0.0),
                        );
                    }
                }
            }
            BattleStage::PlayerAction => {
                if state.input.key_down(KeyCode::KeyX) {
                    self.stage = BattleStage::PlayerMove;
                    let player_character = &self.players[self.active_player_index].character;
                    self.grid.set_movement_highlights(player_character);
                }
                if state.input.key_down(KeyCode::KeyZ) {
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
                        dummy.perform_move(delta, &self.grid);
                        self.grid.occupancy.remove(&dummy.last_position);
                        self.grid.occupancy.insert(dummy.position);
                        dummy.last_position = dummy.position;

                        // flip, for fun
                        dummy.flip_visual();
                    }
                }
                // todo: animate, coroutines would be nice

                // back to the players turn
                let player = &mut self.players[self.active_player_index];
                player.character.start_turn(&self.grid);
                self.grid.set_movement_highlights(&player.character);
                self.stage = BattleStage::PlayerMove;
                self.text_mesh.set_text("Helia Tactics!".to_string());
                self.text_mesh.translate(Vec3::new(0.0, 16.0, 0.0));
                for i in 0..self.text_mesh.text.len() {
                    self.text_mesh
                        .offset_char(i, Vec3::new(0.0, 0.0, 0.0));
                }
            }
        }
    }
    
    pub fn render(&self, commands: &mut Vec<DrawCommand>) {
        // Currently 
        commands.push(self.background.to_draw_command());
        self.grid.render(commands);
        for player in self.players.iter() {
            commands.push(player.character.sprite.to_draw_command());
        }
        for dummy in self.dummys.iter() {
            commands.push(dummy.sprite.to_draw_command());
        }
        self.text_mesh.render(commands);
    }
}
