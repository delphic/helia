mod battle_state;
mod character;
mod grid;
mod player;
mod sprite;
mod utils;

use battle_state::*;
use glam::*;
use helia::{
    atlas::Atlas, camera::*, material::MaterialId, mesh::MeshId,
    primitives::*, ui::font::FontAtlas, ui::*, *,
};
use std::collections::HashMap;

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

        let quad_mesh = quad::centered_mesh(state);
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

const PIXEL_RATIO : u32 = 1;

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
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
            size: OrthographicSize::from_size_scale(state.size, PIXEL_RATIO),
        };

        self.load_resources(state);

        state.camera = camera;

        let mut battle_state = BattleState::new(&self.resources, state);

        battle_state.enter();
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

    fn render(&mut self, commands: &mut Vec<DrawCommand>) {
        if let Stage::Battle { state } = &self.stage {
            state.render(commands);
        }
    }

    fn resize(&mut self, state: &mut State) {
        state.camera.size = OrthographicSize::from_size_scale(state.size, PIXEL_RATIO);
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
