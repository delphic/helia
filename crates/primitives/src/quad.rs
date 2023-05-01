use core::{mesh::Mesh, *};
use glam::*;

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

pub fn centered_mesh(state: &mut State) -> Mesh {
    Mesh::from_arrays(QUAD_POSITIONS, QUAD_UVS, QUAD_INDICES, &state.device)
}

pub fn centred_mesh_with_offset_scale(
    width: f32,
    height: f32,
    offset: Vec2,
    state: &mut State,
) -> Mesh {
    Mesh::from_arrays(
        &positions_with_offset_scale(width, height, offset).as_slice(),
        QUAD_UVS,
        QUAD_INDICES,
        &state.device,
    )
}

pub fn positions_with_offset_scale(width: f32, height: f32, offset: Vec2) -> Vec<Vec3> {
    QUAD_POSITIONS
        .iter()
        .map(|v| Vec3::new(width * v.x + offset.x, height * v.y + offset.y, v.z))
        .collect::<Vec<Vec3>>()
}

pub fn uvs_with_offset_scale(offset: Vec2, scale: Vec2) -> Vec<Vec2> {
    QUAD_UVS
        .iter()
        .map(|v| Vec2::new(offset.x + scale.x * v.x, offset.y + scale.y * v.y))
        .collect::<Vec<Vec2>>()
}

pub fn extend_indices(indices: &mut Vec<u16>, offset: u16) {
    indices.append(
        &mut QUAD_INDICES
            .iter()
            .map(|i| i + offset)
            .collect::<Vec<u16>>(),
    );
}
