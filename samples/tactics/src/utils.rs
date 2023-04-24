use glam::*;
use helia::{
    material::{Material, MaterialId},
    mesh::{Mesh, MeshId},
    texture::Texture,
    *,
};

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

fn sized_quad_positions(width: f32, height: f32, offset: Vec2) -> Vec<Vec3> {
    QUAD_POSITIONS
        .iter()
        .map(|v| Vec3::new(width * v.x + offset.x, height * v.y + offset.y, v.z))
        .collect::<Vec<Vec3>>()
}
// TODO: Should we perhaps just have a single global quad mesh in Helia and use scale instead?

pub fn build_quad_mesh(width: f32, height: f32, offset: Vec2, state: &mut State) -> Mesh {
    Mesh::from_arrays(
        &sized_quad_positions(width, height, offset).as_slice(),
        QUAD_UVS,
        QUAD_INDICES,
        &state.device,
    )
}

pub fn build_sprite_resources(
    label: &str,
    width: f32,
    height: f32,
    offset: Vec2,
    sprite_bytes: &[u8],
    state: &mut State,
) -> (MeshId, MaterialId) {
    let texture = Texture::from_bytes(&state.device, &state.queue, sprite_bytes, label).unwrap();
    let material = Material::new(state.shaders.sprite, texture, &state);
    let material_id = state.resources.materials.insert(material);

    let quad_mesh = build_quad_mesh(width, height, offset, state);
    let mesh_id = state.resources.meshes.insert(quad_mesh);
    (mesh_id, material_id)
}
