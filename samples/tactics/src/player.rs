use crate::{character::*, grid::Grid};
use glam::*;
use helia::{input::VirtualKeyCode, State};

pub struct Player {
    pub character: Character,
    pub facing: IVec2,
}

impl Player {
    pub fn update(
        &mut self,
        grid: &Grid,
        state: &mut State,
        _elapsed: f32,
    ) -> Option<(IVec2, IVec2)> {
        let character = &mut self.character;
        let mut delta = IVec2::ZERO;
        let mut requested_delta = IVec2::ZERO;
        if state.input.key_down(VirtualKeyCode::Left) {
            if character.is_move_valid(grid, IVec2::NEG_X) {
                delta += IVec2::NEG_X;
            }
            requested_delta += IVec2::NEG_X;
        }
        if state.input.key_down(VirtualKeyCode::Right) {
            if character.is_move_valid(grid, IVec2::X) {
                delta += IVec2::X;
            }
            requested_delta += IVec2::X;
        }

        if state.input.key_down(VirtualKeyCode::Up) {
            if character.is_move_valid(grid, IVec2::NEG_Y) {
                delta += IVec2::NEG_Y;
            }
            requested_delta += IVec2::NEG_Y;
        }
        if state.input.key_down(VirtualKeyCode::Down) {
            if character.is_move_valid(grid, IVec2::Y) {
                delta += IVec2::Y;
            }
            requested_delta += IVec2::Y;
        }

        if requested_delta.x != 0 && requested_delta.x.signum() != self.facing.x {
            character.flip_visual(state);
            self.facing.x = requested_delta.x.signum();
        }

        if delta != IVec2::ZERO {
            character.perform_move(delta, grid, state);
        }

        if state.input.key_down(VirtualKeyCode::Z) {
            // this would change battle state if we had any other states
            let character_update = (character.last_position, character.position);
            character.last_position = character.position;
            return Some(character_update);
        }
        None
    }
}
