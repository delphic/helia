use crate::{character::*, grid::Grid};
use glam::*;
use helia::{input::KeyCode, State};

pub struct Player {
    pub character: Character,
    pub facing: IVec2,
}

impl Player {
    pub fn run_move_stage(&mut self, grid: &Grid, state: &mut State) -> Option<(IVec2, IVec2)> {
        let character = &mut self.character;
        let mut delta = IVec2::ZERO;
        let mut requested_delta = IVec2::ZERO;

        if state.input.key_down(KeyCode::ArrowLeft) {
            if character.is_move_valid(IVec2::NEG_X) {
                delta += IVec2::NEG_X;
            }
            requested_delta += IVec2::NEG_X;
        }
        if state.input.key_down(KeyCode::ArrowRight) {
            if character.is_move_valid(IVec2::X) {
                delta += IVec2::X;
            }
            requested_delta += IVec2::X;
        }

        // no diagonal movement allowed
        if delta == IVec2::ZERO {
            if state.input.key_down(KeyCode::ArrowUp) {
                if character.is_move_valid(IVec2::NEG_Y) {
                    delta += IVec2::NEG_Y;
                }
                requested_delta += IVec2::NEG_Y;
            }
            if state.input.key_down(KeyCode::ArrowDown) {
                if character.is_move_valid(IVec2::Y) {
                    delta += IVec2::Y;
                }
                requested_delta += IVec2::Y;
            }
        }

        if requested_delta.x != 0 && requested_delta.x.signum() != self.facing.x {
            character.flip_visual(state);
            self.facing.x = requested_delta.x.signum();
        } else if delta != IVec2::ZERO {
            character.perform_move(delta, grid, state);
        }

        if state.input.key_down(KeyCode::KeyZ) {
            // this would change battle state if we had any other states
            let character_update = (character.last_position, character.position);
            character.last_position = character.position;
            return Some(character_update);
        }
        None
    }
}
