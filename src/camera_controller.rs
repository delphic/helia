use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

use crate::camera::*;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}
// TODO: Move most of that ^^ to input map

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self { 
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    // Orbit camera
    pub fn update_camera(&self, camera: &mut Camera, elapsed: f32) {
        let to_target = camera.target - camera.eye;
        let forward = to_target.normalize();
        let distance_to_target = to_target.length();
        let delta = self.speed * elapsed;

        if self.is_forward_pressed && distance_to_target > delta {
            camera.eye += forward * delta;
        }
        if self.is_backward_pressed {
            camera.eye -= forward * delta;
        }

        // Rotate which is probably fine cause small angle approx.
        let right = forward.cross(camera.up);
        let to_target = camera.target - camera.eye;
        let distance_to_target = to_target.length();

        if self.is_right_pressed {
            camera.eye = camera.target - (forward + right * delta).normalize() * distance_to_target;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * delta).normalize() * distance_to_target;
        }
    }
}
