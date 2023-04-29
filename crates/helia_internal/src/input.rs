use glam::Vec2;
use instant::Instant;
use std::{
    cmp::Eq,
    collections::{HashMap, HashSet},
    hash::Hash,
};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseScrollDelta, WindowEvent},
};

pub type VirtualKeyCode = winit::event::VirtualKeyCode;
pub type MouseButton = winit::event::MouseButton;

pub struct InputState {
    pub mouse_position: PhysicalPosition<f64>,
    pub mouse_delta: Vec2,
    pub mouse_scroll_delta: Vec2,
    pub pixel_scroll_ratio: f32,
    last_mouse_position: PhysicalPosition<f64>,
    key_map: InputMap<VirtualKeyCode>,
    mouse_button_map: InputMap<MouseButton>,
}

struct InputMap<T: Eq + Hash + Copy> {
    pressed: HashSet<T>,
    down: HashSet<T>,
    up: HashSet<T>,
    down_times: HashMap<T, Instant>,
    up_times: HashMap<T, Instant>,
}

impl<T: Eq + Hash + Copy> InputMap<T> {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            up: HashSet::new(),
            up_times: HashMap::new(),
            down: HashSet::new(),
            down_times: HashMap::new(),
        }
    }

    pub fn pressed(&mut self, key: T) {
        if !self.pressed.contains(&key) {
            self.down.insert(key);
            self.down_times.insert(key, Instant::now());
        }
        self.pressed.insert(key);
    }

    pub fn released(&mut self, key: T) {
        self.pressed.remove(&key);
        self.up.insert(key);
        self.up_times.insert(key, Instant::now());
    }

    pub fn frame_finished(&mut self) {
        self.up.clear();
        self.down.clear();
    }

    pub fn is_pressed(&self, key: T) -> bool {
        self.pressed.contains(&key)
    }

    pub fn down(&self, key: T) -> bool {
        self.down.contains(&key)
    }

    pub fn up(&self, key: T) -> bool {
        self.up.contains(&key)
    }

    pub fn up_elapsed(&self, key: T) -> Option<f32> {
        self.up_times.get(&key).map(|x| x.elapsed().as_secs_f32())
    }

    pub fn down_elapsed(&self, key: T) -> Option<f32> {
        self.down_times.get(&key).map(|x| x.elapsed().as_secs_f32())
    }
}

impl InputState {
    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput { state, button, .. } => match *state {
                ElementState::Pressed => self.mouse_button_map.pressed(*button),
                ElementState::Released => self.mouse_button_map.released(*button),
            },
            WindowEvent::MouseWheel { delta, .. } => match *delta {
                MouseScrollDelta::LineDelta(x, y) => self.mouse_scroll_delta += Vec2::new(x, y),
                MouseScrollDelta::PixelDelta(position) => {
                    self.mouse_scroll_delta +=
                        self.pixel_scroll_ratio * Vec2::new(position.x as f32, position.y as f32)
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_delta = Vec2::new(
                    (position.x - self.last_mouse_position.x) as f32,
                    (position.y - self.last_mouse_position.y) as f32,
                );
                self.mouse_position = *position;
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => match *state {
                ElementState::Pressed => self.key_map.pressed(*keycode),
                ElementState::Released => self.key_map.released(*keycode),
            },
            _ => {}
        }
    }

    pub fn frame_finished(&mut self) {
        self.key_map.frame_finished();
        self.mouse_button_map.frame_finished();
        self.mouse_delta = Vec2::ZERO;
        self.mouse_scroll_delta = Vec2::ZERO;
        self.last_mouse_position = self.mouse_position;
    }

    /// Is key currently pressed
    pub fn key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.key_map.is_pressed(keycode)
    }

    /// Was key pressed this frame
    pub fn key_down(&self, keycode: VirtualKeyCode) -> bool {
        self.key_map.down(keycode)
    }

    /// Was key released this frame
    pub fn key_up(&self, keycode: VirtualKeyCode) -> bool {
        self.key_map.up(keycode)
    }

    /// Get the amount of real time since the the last key up event
    pub fn key_up_elapsed(&self, keycode: VirtualKeyCode) -> Option<f32> {
        self.key_map.up_elapsed(keycode)
    }

    /// Get the amount of real time since the last key down event
    pub fn key_down_elapsed(&self, keycode: VirtualKeyCode) -> Option<f32> {
        self.key_map.down_elapsed(keycode)
    }

    /// Is mouse button currently pressed
    pub fn mouse_button_pressed(&self, mouse_button: MouseButton) -> bool {
        self.mouse_button_map.is_pressed(mouse_button)
    }

    /// Was mouse button pressed this frame
    pub fn mouse_button_down(&self, mouse_button: MouseButton) -> bool {
        self.mouse_button_map.down(mouse_button)
    }

    // Was mouse button released this frame
    pub fn mouse_button_up(&self, mouse_button: MouseButton) -> bool {
        self.mouse_button_map.up(mouse_button)
    }

    /// Get the amount of real time since the the last mouse button up event
    pub fn mouse_button_up_elapsed(&self, mouse_button: MouseButton) -> Option<f32> {
        self.mouse_button_map.up_elapsed(mouse_button)
    }

    /// Get the amount of real time since the last mouse button down event
    pub fn mouse_button_down_elapsed(&self, mouse_button: MouseButton) -> Option<f32> {
        self.mouse_button_map.down_elapsed(mouse_button)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_position: PhysicalPosition { x: 0.0, y: 0.0 },
            last_mouse_position: PhysicalPosition { x: 0.0, y: 0.0 },
            mouse_delta: Vec2::ZERO,
            key_map: InputMap::new(),
            mouse_button_map: InputMap::new(),
            pixel_scroll_ratio: 1.0,
            mouse_scroll_delta: Vec2::ZERO,
        }
    }
}
