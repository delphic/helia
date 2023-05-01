use glam::*;
use helia::{
    material::{Material, MaterialId},
    mesh::MeshId,
    primitives::quad::*,
    texture::Texture,
    *,
};

pub fn build_material(sprite_bytes: &[u8], state: &mut State) -> MaterialId {
    let texture = Texture::from_bytes(&state.device, &state.queue, sprite_bytes).unwrap();
    let texture_id = state.resources.textures.insert(texture);
    let material = Material::new(state.shaders.sprite, texture_id, &state);
    state.resources.materials.insert(material)
}

pub fn build_sprite_resources(
    width: f32,
    height: f32,
    offset: Vec2,
    sprite_bytes: &[u8],
    state: &mut State,
) -> (MeshId, MaterialId) {
    let quad_mesh = centred_mesh_with_offset_scale(width, height, offset, state);
    let mesh_id = state.resources.meshes.insert(quad_mesh);
    let material_id = build_material(sprite_bytes, state);
    (mesh_id, material_id)
}
