use glam::*;
use helia::{
    camera::{Camera, OrthographicSize},
    entity::*,
    input::VirtualKeyCode,
    material::{Material, MaterialId},
    mesh::{Mesh, MeshId},
    prefab::PrefabId,
    texture::Texture,
    *,
};

const QUAD_POSITIONS: &[Vec3] = &[
    Vec3::new(-0.5, -0.5, 0.0),
    Vec3::new(0.5, -0.5, 0.0),
    Vec3::new(0.5, 0.5, 0.0),
    Vec3::new(-0.5, 0.5, 0.0),
];
const QUAD_UVS: &[Vec2] = &[
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
];
const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

fn sized_quad_positions(width: f32, height: f32) -> Vec<Vec3> {
    QUAD_POSITIONS
        .iter()
        .map(|v| Vec3::new(width * v.x, height * v.y, v.z))
        .collect::<Vec<Vec3>>()
}

pub struct Grid {
    size: IVec2,
    base_offset: Vec3,
    highlights: Vec<(EntityId, IVec2)>,
}

impl Grid {
    fn new() -> Self {
        let size = IVec2::new(12, 3);
        let base_offset = Vec3::new(-400.0, 16.0, 32.0); // dependent on bg sprite currently

        Self {
            size,
            base_offset,
            highlights: Vec::new(),
        }
    }

    fn init(&mut self, prefab_id: PrefabId, state: &mut State) {
        let n = (self.size.x * self.size.y) as i32;
        for i in 0..n {
            let position = IVec2::new(i % self.size.x, i / self.size.x);
            let id = state.scene.add_instance(
                prefab_id,
                InstanceProperties::builder()
                    .with_translation(
                        self.get_translation_for_position(position)
                            - 48.0 * Vec3::Y
                            - 32.0 * Vec3::Z,
                    ) // could sort this y offset with better anchoring and base offset
                    .with_color(Color::TRANSPARENT) // Visibility rather than transparent would be nice
                    .build(),
            );
            self.highlights.push((id, position));
        }
    }

    fn is_in_bounds(&self, grid_position: IVec2) -> bool {
        grid_position.x >= 0
            && grid_position.x < self.size.x
            && grid_position.y >= 0
            && grid_position.y < self.size.y
    }

    fn get_translation_for_position(&self, grid_position: IVec2) -> Vec3 {
        let x = grid_position.x as f32;
        let y = grid_position.y as f32;
        self.base_offset + Vec3::new(64.0 * x + 32.0 * y, -32.0 * y, 16.0 * y)
    }

    fn distance(a: IVec2, b: IVec2) -> i32 {
        (a.x - b.x).abs() + (a.y - b.y).abs()
    }

    fn update_hightlights(&self, position: IVec2, distance: i32, state: &mut State) {
        for (id, highlight_pos) in self.highlights.iter() {
            let entity = state.scene.get_entity_mut(*id);
            entity.properties.color = if Grid::distance(position, *highlight_pos) <= distance {
                Color {
                    r: 0.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.5,
                }
            } else {
                Color::TRANSPARENT
            };
        }
    }
}

pub struct Character {
    position: IVec2,
    last_position: IVec2,
    sprite: EntityId,
    movement: u16,
}

impl Character {
    pub fn create_on_grid(
        position: IVec2,
        mesh_id: MeshId,
        material_id: MaterialId,
        grid: &Grid,
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
        Self {
            position,
            last_position: position,
            sprite,
            movement: 3,
        }
    }

    pub fn is_move_valid(&self, grid: &Grid, delta: IVec2) -> bool {
        let target_position = self.position + delta;
        grid.is_in_bounds(target_position)
            && Grid::distance(target_position, self.last_position) <= self.movement as i32
    }
}

pub struct Player {
    character: Character,
}

impl Player {
    fn update(&mut self, grid: &Grid, state: &mut State, _elapsed: f32) {
        let character = &mut self.character;
        let mut delta = IVec2::ZERO;
        if state.input.key_down(VirtualKeyCode::Left) {
            if character.is_move_valid(grid, IVec2::NEG_X) {
                delta += IVec2::NEG_X;
            }
        }
        if state.input.key_down(VirtualKeyCode::Right) {
            if character.is_move_valid(grid, IVec2::X) {
                delta += IVec2::X;
            }
        }

        if state.input.key_down(VirtualKeyCode::Up) {
            if character.is_move_valid(grid, IVec2::NEG_Y) {
                delta += IVec2::NEG_Y;
            }
        }
        if state.input.key_down(VirtualKeyCode::Down) {
            if character.is_move_valid(grid, IVec2::Y) {
                delta += IVec2::Y;
            }
        }

        if delta != IVec2::ZERO {
            character.position += delta;
            let entity = state.scene.get_entity_mut(character.sprite);
            entity.properties.transform =
                Mat4::from_translation(grid.get_translation_for_position(character.position));
        }

        if state.input.key_down(VirtualKeyCode::Z) {
            // this would change battle state if we had any other states
            character.last_position = character.position;
            grid.update_hightlights(character.position, character.movement as i32, state);
        }
    }
}

pub struct GameState {
    player: Option<Player>,
    grid: Grid,
}

impl GameState {
    fn new() -> Self {
        Self {
            player: None,
            grid: Grid::new(),
        }
    }
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        fn build_sprite_resources(
            label: &str,
            width: f32,
            height: f32,
            sprite_bytes: &[u8],
            state: &mut State,
        ) -> (MeshId, MaterialId) {
            let texture =
                Texture::from_bytes(&state.device, &state.queue, sprite_bytes, label).unwrap();
            let material = Material::new(state.shaders.sprite, texture, &state);
            let material_id = state.resources.materials.insert(material);

            let quad_mesh = Mesh::from_arrays(
                &sized_quad_positions(width, height).as_slice(),
                QUAD_UVS,
                QUAD_INDICES,
                &state.device,
            );
            let mesh_id = state.resources.meshes.insert(quad_mesh);
            (mesh_id, material_id)
        }

        let helia_sprite_ids = build_sprite_resources(
            "helia",
            96.0,
            96.0,
            include_bytes!("../assets/helia.png"),
            state,
        );
        let bg_sprite_ids = build_sprite_resources(
            "bg",
            960.0,
            480.0,
            include_bytes!("../assets/placeholder-bg.png"),
            state,
        );
        let highlight_ids = build_sprite_resources(
            "sq",
            96.0,
            32.0,
            include_bytes!("../assets/grid_sq.png"),
            state,
        );

        let camera = Camera {
            eye: (0.0, 0.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: state.size.width as f32 / state.size.height as f32,
            fov: 60.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 1000.0,
            clear_color: Color::BLACK,
            projection: camera::Projection::Orthographic,
            size: OrthographicSize::from_size(state.size),
        };

        state.scene.camera = camera;

        let player_character = Character::create_on_grid(
            IVec2::new(8, 1),
            helia_sprite_ids.0,
            helia_sprite_ids.1,
            &self.grid,
            state,
        );
        let (player_position, player_movement) =
            (player_character.position, player_character.movement);
        self.player = Some(Player {
            character: player_character,
        });

        state.scene.add_entity(
            bg_sprite_ids.0,
            bg_sprite_ids.1,
            InstanceProperties::builder()
                .with_translation(Vec3::new(0.0, 0.0, -100.0))
                .build(),
        );

        let highlight_prefab = state.scene.create_prefab(highlight_ids.0, highlight_ids.1);

        self.grid.init(highlight_prefab, state);
        self.grid
            .update_hightlights(player_position, player_movement as i32, state);
    }

    fn update(&mut self, state: &mut State, _elapsed: f32) {
        if let Some(player) = &mut self.player {
            player.update(&self.grid, state, _elapsed);
        }
    }

    fn resize(&mut self, state: &mut State) {
        state.scene.camera.size = OrthographicSize::from_size(state.size);
    }
}

pub async fn run() {
    Helia::new()
        .with_title("Helia Tactics")
        .with_size(960, 640)
        .with_resizable(false)
        .run(Box::new(GameState::new()))
        .await;
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    run().await;
}

fn main() {
    pollster::block_on(run());
}
