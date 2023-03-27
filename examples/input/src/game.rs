use glam::*;
use helia::{camera::Camera, *};
use winit::event::WindowEvent;

pub struct GameState {}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let camera = Camera {
            eye: (-0.5, 1.0, 2.0).into(),
            target: (-0.5, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: state.size.width as f32 / state.size.height as f32,
            fov: 60.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 1000.0,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        };

        state.scene.camera = camera;
    }

    fn update(&mut self, _state: &mut State, _elapsed: f32) {}

    fn input(&mut self, state: &mut State, event: &winit::event::WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                state.scene.camera.clear_color = wgpu::Color {
                    r: position.x / state.size.width as f64,
                    g: 0.2,
                    b: position.y / state.size.height as f64,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }

    fn resize(&mut self, state: &mut State) {
        state.scene.camera.aspect_ratio = state.size.width as f32 / state.size.height as f32;
    }
}

pub async fn run() {
    let game_state = GameState {};
    helia::run(Box::new(game_state)).await;
}
