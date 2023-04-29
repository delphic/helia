use glam::*;
use helia::{
    camera::{Camera, OrthographicSize},
    entity::*,
    mesh::Mesh,
    *,
};

// todo: move to helia::aseprite module
// ideally should be optional module
mod aseprite {
    #[derive(Debug, serde::Deserialize)]
    pub struct AsepriteAnimation {
        pub meta: Meta,
        pub frames: Vec<AnimationFrameData>,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Meta {
        pub size: Size,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Size {
        pub w: u64,
        pub h: u64,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct AnimationFrameData {
        pub frame: Frame,
        pub duration: u64,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Frame {
        pub x: u64,
        pub y: u64,
        pub w: u64,
        pub h: u64,
    }
}

use self::aseprite::*;

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
pub struct GameState {
    sprite_data: AsepriteAnimation,
    time_in_frame: f32,
    current_frame: usize,
    lena: Option<EntityId>,
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let sprite_bytes = include_bytes!("../assets/lena_shoot.png");

        let texture =
            helia::texture::Texture::from_bytes(&state.device, &state.queue, sprite_bytes).unwrap();
        let texture_id = state.resources.textures.insert(texture);
        let lena_material =
            helia::material::Material::new(state.shaders.sprite, texture_id, &state);
        let lena_material_id = state.resources.materials.insert(lena_material);

        let quad_mesh = Mesh::from_arrays(QUAD_POSITIONS, QUAD_UVS, QUAD_INDICES, &state.device);
        let mesh_id = state.resources.meshes.insert(quad_mesh);

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
            size: OrthographicSize::from_ratio_height(ratio, 1.0),
            pixel_ratio: 1.0,
        };

        state.scene.camera = camera;

        let (scale, offset) = self.calculate_scale_offset(self.current_frame);
        self.lena = Some(
            state.scene.add_entity(
                mesh_id,
                lena_material_id,
                InstanceProperties::builder()
                    .with_uv_offset_scale(offset, scale)
                    .build(),
            ),
        );
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        self.time_in_frame += elapsed * 1000.0;
        let frame_duration = self.sprite_data.frames[self.current_frame].duration as f32;
        if self.time_in_frame > frame_duration {
            self.time_in_frame -= frame_duration;
            self.current_frame = (self.current_frame + 1) % self.sprite_data.frames.len();
            if let Some(entity_id) = self.lena {
                let (scale, offset) = self.calculate_scale_offset(self.current_frame);
                let lena = state.scene.get_entity_mut(entity_id);
                lena.properties.uv_scale = scale;
                lena.properties.uv_offset = offset;
            }
        }
    }

    fn resize(&mut self, state: &mut State) {
        let ratio = state.size.width as f32 / state.size.height as f32;
        state.scene.camera.size = OrthographicSize::from_ratio_height(ratio, 1.0);
    }
}

impl GameState {
    fn calculate_scale_offset(&mut self, index: usize) -> (Vec2, Vec2) {
        let frame = &self.sprite_data.frames[index].frame;
        let w = self.sprite_data.meta.size.w as f32;
        let h = self.sprite_data.meta.size.h as f32;
        let scale = Vec2::new(frame.w as f32 / w, frame.h as f32 / h);
        let offset = Vec2::new(frame.x as f32 / w, frame.y as f32 / h);
        (scale, offset)
    }
}

pub async fn run() {
    let game_state = GameState {
        current_frame: 0,
        time_in_frame: 0.0,
        lena: None,
        sprite_data: serde_json::from_str::<AsepriteAnimation>(include_str!(
            "../assets/lena_shoot.json"
        ))
        .unwrap(),
    };
    Helia::new().run(Box::new(game_state)).await;
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
