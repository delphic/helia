use std::collections::{HashSet, HashMap};
use instant::Instant;
use winit::event::{WindowEvent, ElementState, KeyboardInput};

pub type VirtualKeyCode = winit::event::VirtualKeyCode;

pub struct InputState {
    pub mouse_position: winit::dpi::PhysicalPosition<f64>,
    pressed: HashSet<VirtualKeyCode>,
    down: HashSet<VirtualKeyCode>,
    up: HashSet<VirtualKeyCode>,
    down_times: HashMap<VirtualKeyCode, Instant>,
    up_times: HashMap<VirtualKeyCode, Instant>,
}

impl InputState {
    pub fn process_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => self.mouse_position = *position,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                match *state {
                    ElementState::Pressed => {
                        self.pressed.insert(*keycode);
                        self.down.insert(*keycode);
                        self.down_times.insert(*keycode, Instant::now());
                    },
                    ElementState::Released => {
                        self.pressed.remove(keycode);
                        self.up.insert(*keycode);
                        self.up_times.insert(*keycode, Instant::now());
                    }
                }
            }
            _ => {},
        }
    }

    pub fn frame_finished(&mut self) {
        self.up.clear();
        self.down.clear();
    }

    /// If a key is currently pressed
    pub fn key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.pressed.contains(&keycode)
    }

    /// If a key was pressed this frame
    pub fn key_down(&self, keycode: VirtualKeyCode) -> bool {
        self.down.contains(&keycode)
    }

    /// If a key was released this frame
    pub fn key_up(&self, keycode: VirtualKeyCode) -> bool {
        self.up.contains(&keycode)
    }

    /// Get the amount of real time since the the last key up event
    pub fn key_up_elapsed(&self, keycode: VirtualKeyCode) -> Option<f32> {
        self.up_times.get(&keycode).map(|x| x.elapsed().as_secs_f32())
    }

    /// Get the amount of real time since the last key down event
    pub fn key_down_elapsed(&self, keycode: VirtualKeyCode) -> Option<f32> {
        self.down_times.get(&keycode).map(|x| x.elapsed().as_secs_f32())
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mouse_position: winit::dpi::PhysicalPosition { x: 0.0, y: 0.0 },
            pressed: HashSet::new(),
            up: HashSet::new(),
            up_times: HashMap::new(),
            down: HashSet::new(),
            down_times: HashMap::new(),
        }
    }
} 