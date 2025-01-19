use std::collections::HashMap;

use glam::*;
use helia::{
    atlas::*,
    camera::{Camera, OrthographicSize},
    entity::*,
    material::*,
    mesh::*,
    texture::*,
    ui::{ *, font::* },
    primitives::*,
    *,
};

pub struct Sprite {
    pub mesh_id: MeshId,
    pub material_id: MaterialId,
    pub position: Vec3,
    pub scale: Vec3,
    pub uv_scale: Vec2,
    pub uv_offset: Vec2,
    pub color: Color,
}

impl Sprite {
    pub fn to_draw_command(&self) -> DrawCommand {
        DrawCommand::Draw(
            self.mesh_id,
            self.material_id,
            RenderProperties::builder()
                .with_uv_offset_scale(self.uv_offset, self.uv_scale)
                .with_color(self.color)
                .with_matrix(Transform::from_position_scale(self.position, self.scale).into())
                .build()
            )
    }
}

pub struct GameState {
    text_mesh: Option<TextMesh>,
    slice_mesh: Option<SliceSpriteMesh>,
    sprites: Vec<Sprite>,
}

const PIXEL_RATIO: u32 = 2;

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let ratio = state.size.width as f32 / state.size.height as f32;
        let camera = Camera {
            eye: (0.0, 0.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: ratio,
            fov: 60.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 1000.0,
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            projection: camera::Projection::Orthographic,
            size: OrthographicSize::from_size_scale(state.size, PIXEL_RATIO),
        };
        state.camera = camera;

        let quad_mesh = quad::centered_mesh(state);
        let char_map = "ABCDEFGHIJKLMNOPQRSTUVabcdefghijklmnopqrstuvWXYZ0123456789_.,!?:; wxyz()[]{}'\"/\\|=-+*<>%".to_string();

        let mesh_id = state.resources.meshes.insert(quad_mesh);
        let material_id = build_sprite_material(include_bytes!("../assets/mini-font.png"), state);

        let mut custom_widths = HashMap::new();
        custom_widths.insert(5, "abcdeghknopqstuvxyz.,!?:;=".to_string());
        custom_widths.insert(4, "fr0123456789 {}'\"/\\|-+*<>".to_string());
        custom_widths.insert(3, "jl()[]".to_string());
        custom_widths.insert(2, "i".to_string());

        let mini_atlas = FontAtlas {
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

        let text_mesh = TextMesh::builder(
                "The Quick Brown Fox Jumped Over the Lazy Dog!".to_string(),
                Vec3::new(0.0, 0.0, 0.0),
                mini_atlas, // The fact this just takes the atlas and it needs cloning is ... not good
            )
            .with_alignment(TextAlignment::Center)
            .with_vertical_alignment(VerticalAlignment::Center)
            .build();
        self.text_mesh = Some(text_mesh);

        // 9 slice test
        let slice_mesh = SliceSpriteMesh::new(
            Vec2::new(32.0, 16.0),
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
        let material_id = build_sprite_material(include_bytes!("../assets/slice.png"), state);

        let sliced_sprite = Sprite { 
            mesh_id: slice_mesh.mesh,
            material_id,
            position: Vec3::new(0.0, state.camera.size.top - 16.0, 0.0),
            scale: Vec3::ONE,
            uv_offset: Vec2::ZERO,
            uv_scale: Vec2::ONE,
            color: Color::WHITE,
        };
        self.slice_mesh = Some(slice_mesh);
        self.sprites.push(sliced_sprite);
    }

    fn update(&mut self, state: &mut State, _elapsed: f32) {
        if let Some(text_mesh) = &mut self.text_mesh {
            if state.input.key_down(KeyCode::KeyZ) {
                text_mesh.set_text("The Quick Brown Fox Jumped Over the Lazy Dog!".to_string());
                text_mesh.translate(Vec3::new(0.0, 0.0, 0.0));
                for i in 0..text_mesh.text.len() {
                    text_mesh
                        .offset_char(i, Vec3::new(0.0, 0.0, 0.0));
                }
                if let Some(slice_mesh) = &mut self.slice_mesh {
                    slice_mesh.resize(Vec2::new(32.0, 16.0), state);
                }
            }
            if state.input.key_down(KeyCode::KeyX) {
                text_mesh.set_text("Testing Testing".to_string());
                text_mesh.translate(Vec3::new(0.0, 16.0, 0.0));
                for i in 0..text_mesh.text.len() {
                    text_mesh.offset_char(
                        i,
                        Vec3::new(0.0, (i as f32 / 2.0).sin() * 4.0, 0.0),
                    );
                }
                if let Some(slice_mesh) = &mut self.slice_mesh {
                    slice_mesh.resize(Vec2::new(16.0, 16.0), state);
                }
            }
        }
    }

    fn render(&mut self, commands: &mut Vec<DrawCommand>) {
        for sprite in self.sprites.iter() {
            commands.push(sprite.to_draw_command());
        }
        if let Some(text_mesh) = &self.text_mesh {
            text_mesh.render(commands);
        }
    }

    fn resize(&mut self, state: &mut State) {
        state.camera.size = OrthographicSize::from_size_scale(state.size, PIXEL_RATIO);
    }
}

pub fn build_sprite_material(sprite_bytes: &[u8], state: &mut State) -> MaterialId {
    let texture = Texture::from_bytes(&state.device, &state.queue, sprite_bytes).unwrap();
    let texture_id = state.resources.textures.insert(texture);
    let material = Material::new(state.shaders.sprite, texture_id, &state);
    state.resources.materials.insert(material)
}

pub async fn run() {
    let game_state = GameState {
        text_mesh: None,
        slice_mesh: None,
        sprites: Vec::new(),
    };
    Helia::new().with_resizable(true).run(Box::new(game_state)).await;
}

use input::KeyCode;
use transform::Transform;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    run().await;
}

fn main() {
    pollster::block_on(run());
}
