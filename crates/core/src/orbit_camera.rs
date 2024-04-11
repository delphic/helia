use crate::camera::*;
use crate::input::*;

pub struct OrbitCamera {
    speed: f32,
}

impl OrbitCamera {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }

    pub fn update_camera(&self, camera: &mut Camera, input: &InputState, elapsed: f32) {
        let is_forward_pressed =
            input.key_pressed(KeyCode::KeyW) || input.key_pressed(KeyCode::ArrowUp);
        let is_left_pressed =
            input.key_pressed(KeyCode::KeyA) || input.key_pressed(KeyCode::ArrowLeft);
        let is_backward_pressed =
            input.key_pressed(KeyCode::KeyS) || input.key_pressed(KeyCode::ArrowDown);
        let is_right_pressed =
            input.key_pressed(KeyCode::KeyD) || input.key_pressed(KeyCode::ArrowRight);

        let to_target = camera.target - camera.eye;
        let forward = to_target.normalize();
        let distance_to_target = to_target.length();
        let delta = self.speed * elapsed;

        if is_forward_pressed && distance_to_target > delta {
            camera.eye += forward * delta;
        }
        if is_backward_pressed {
            camera.eye -= forward * delta;
        }

        // Rotate which is probably fine cause small angle approx.
        let right = forward.cross(camera.up);
        let to_target = camera.target - camera.eye;
        let distance_to_target = to_target.length();

        if is_right_pressed {
            camera.eye = camera.target - (forward - right * delta).normalize() * distance_to_target;
        }
        if is_left_pressed {
            camera.eye = camera.target - (forward + right * delta).normalize() * distance_to_target;
        }
    }
}
