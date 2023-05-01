mod battle_state;
mod character;
mod grid;
mod player;
mod slice_sprite;
mod text_mesh;
mod utils;

use std::collections::HashMap;

use battle_state::*;
use slice_sprite::*;

use glam::*;
use helia::{camera::*, entity::InstanceProperties, material::MaterialId, mesh::MeshId, *};

pub struct GameTexture<'a> {
    pub name: String,
    pub bytes: &'a [u8], // should take texture Id for sanity here
    pub dimensions: Vec2,
    pub offset: Vec2,
}

impl<'a> GameTexture<'a> {
    pub fn build_texture(&self, state: &mut State) -> texture::Texture {
        texture::Texture::from_bytes(&state.device, &state.queue, self.bytes).unwrap()
    }
}

pub struct GameResources {
    pub meshes: HashMap<String, MeshId>,
    pub materials: HashMap<String, MaterialId>,
    pub fonts: HashMap<String, FontAtlas>,
}

impl GameResources {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            materials: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, pair: (MeshId, MaterialId)) {
        self.meshes.insert(key.clone(), pair.0);
        self.materials.insert(key, pair.1);
    }

    pub fn get_pair(&self, key: &String) -> Option<(MeshId, MaterialId)> {
        if let (Some(mesh_id), Some(material_id)) = (self.meshes.get(key), self.materials.get(key))
        {
            Some((*mesh_id, *material_id))
        } else {
            None
        }
    }
}

enum Stage {
    Init,
    Battle { state: BattleState },
}

pub struct GameState {
    stage: Stage,
    resources: GameResources,
}

impl GameState {
    fn new() -> Self {
        Self {
            stage: Stage::Init,
            resources: GameResources::new(),
        }
    }

    fn load_resources(&mut self, state: &mut State) {
        // just keeping all resources in memory for now
        // will probably want a way to clear and reset
        // resources in a larger game though
        self.resources.insert(
            "helia".to_string(),
            utils::build_sprite_resources(
                96.0,
                96.0,
                Vec2::new(0.0, 48.0),
                include_bytes!("../assets/helia.png"),
                state,
            ),
        );
        self.resources.insert(
            "bg".to_string(),
            utils::build_sprite_resources(
                960.0,
                480.0,
                Vec2::ZERO,
                include_bytes!("../assets/placeholder-bg.png"),
                state,
            ),
        );
        self.resources.insert(
            "highlight".to_string(),
            utils::build_sprite_resources(
                96.0,
                32.0,
                Vec2::new(0.0, 16.0),
                include_bytes!("../assets/grid_sq.png"),
                state,
            ),
        );
        self.resources.insert(
            "dummy".to_string(),
            utils::build_sprite_resources(
                64.0,
                64.0,
                Vec2::new(0.0, 32.0),
                include_bytes!("../assets/dummy.png"),
                state,
            ),
        );

        let quad_mesh = crate::utils::build_quad_mesh(1.0, 1.0, Vec2::ZERO, state);
        let char_map = "ABCDEFGHIJKLMNOPQRSTUVabcdefghijklmnopqrstuvWXYZ0123456789_.,!?:; wxyz()[]{}'\"/\\|=-+*<>%".to_string();

        let mesh_id = state.resources.meshes.insert(quad_mesh);
        let material_id = utils::build_material(include_bytes!("../assets/micro-font.png"), state);

        let micro_font = FontAtlas {
            atlas: Atlas {
                mesh_id,
                material_id,
                tile_width: 4,
                tile_height: 6,
                columns: 22,
                rows: 4,
            },
            char_map: char_map.clone(),
            custom_char_widths: None,
        };
        self.resources.fonts.insert("micro".to_string(), micro_font);

        let material_id = utils::build_material(include_bytes!("../assets/mini-font.png"), state);

        let mut custom_widths = HashMap::new();
        custom_widths.insert(5, "abcdeghknopqstuvxyz.,!?:;=".to_string());
        custom_widths.insert(4, "fr0123456789 {}'\"/\\|-+*<>".to_string());
        custom_widths.insert(3, "jl()[]".to_string());
        custom_widths.insert(2, "i".to_string());

        let mini_font = FontAtlas {
            atlas: Atlas {
                mesh_id,
                material_id,
                tile_width: 6,
                tile_height: 8,
                columns: 22,
                rows: 4,
            },
            char_map: char_map.clone(),
            custom_char_widths: Some(FontAtlas::build_char_widths(custom_widths)),
        };
        self.resources.fonts.insert("mini".to_string(), mini_font);

        self.resources.materials.insert(
            "white-sq".to_string(),
            utils::build_material(include_bytes!("../assets/white-sq.png"), state),
        );
        self.resources.materials.insert(
            "border".to_string(),
            utils::build_material(include_bytes!("../assets/border.png"), state),
        );
    }
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let pixel_ratio = 1;
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
            size: OrthographicSize::from_size_scale(state.size, pixel_ratio),
            pixel_ratio: pixel_ratio as f32,
        };

        self.load_resources(state);

        state.scene.camera = camera;

        let mut battle_state = BattleState::new(&self.resources, state);

        // 9 slice test
        let mut slice_mesh = SliceSpriteMesh::new(
            Vec2::new(16.0, 16.0),
            SliceConfig {
                width: 8.0,
                height: 8.0,
                top: 2.0,
                right: 2.0,
                bottom: 2.0,
                left: 2.0,
            },
            state,
        );
        let texture = helia::texture::Texture::from_bytes(
            &state.device,
            &state.queue,
            include_bytes!("../assets/slice.png"),
        )
        .unwrap();
        let texture_id = state.resources.textures.insert(texture);
        let material = helia::material::Material::new(state.shaders.sprite, texture_id, &state);
        let material_id = state.resources.materials.insert(material);
        state.scene.add_entity(
            slice_mesh.mesh,
            material_id,
            InstanceProperties::builder()
                .with_transform(transform::Transform::from_position_scale(
                    Vec3::new(0.0, 64.0, 0.0),
                    4.0 * Vec3::ONE,
                ))
                .build(),
        );

        slice_mesh.resize(Vec2::new(32.0, 16.0), state);

        battle_state.enter(state);
        self.stage = Stage::Battle {
            state: battle_state,
        };
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        match &mut self.stage {
            Stage::Init => {}
            Stage::Battle {
                state: battle_state,
            } => battle_state.update(state, elapsed),
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

use text_mesh::{Atlas, FontAtlas};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    run().await;
}

fn main() {
    pollster::block_on(run());
}
