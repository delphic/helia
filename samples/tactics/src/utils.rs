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

fn quad_uvs(offset: Vec2, scale: Vec2) -> Vec<Vec2> {
    QUAD_UVS
        .iter()
        .map(|v| Vec2::new(offset.x + scale.x * v.x, offset.y + scale.y * v.y))
        .collect::<Vec<Vec2>>()
}

fn extend_indices(indices: &mut Vec<u16>, offset: u16) {
    indices.append(
        &mut QUAD_INDICES
            .iter()
            .map(|i| i + offset)
            .collect::<Vec<u16>>(),
    );
}

pub fn build_quad_mesh(width: f32, height: f32, offset: Vec2, state: &mut State) -> Mesh {
    Mesh::from_arrays(
        &sized_quad_positions(width, height, offset).as_slice(),
        QUAD_UVS,
        QUAD_INDICES,
        &state.device,
    )
}

pub fn build_9_slice_mesh(
    width: f32,
    height: f32,
    image_width: f32,
    image_height: f32,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
    state: &mut State,
) -> Mesh {
    let half_width = width / 2.0;
    let half_height = height / 2.0;
    let inner_width = width - left - right;
    let inner_height = height - top - bottom;
    let inner_image_width = image_width - left - right;
    let inner_image_height = image_height - top - bottom;
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let left_offset = -(half_width - 0.5 * left);
    let top_offset = half_height - 0.5 * top;
    let right_offset = half_width - 0.5 * right;
    let bottom_offset = -(half_height - 0.5 * bottom);

    // top left
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        left,
        top,
        Vec2::new(left_offset, top_offset),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::ZERO,
        Vec2::new(left / image_width, top / image_height),
    ));
    // top
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        inner_width,
        top,
        Vec2::new(0.0, top_offset),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new(left / image_width, 0.0),
        Vec2::new(inner_image_width / image_width, top / image_height),
    ));
    // top right
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        right,
        top,
        Vec2::new(right_offset, top_offset),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new((image_width - right) / image_width, 0.0),
        Vec2::new(right / image_width, top / image_height),
    ));
    // left
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        left,
        inner_height,
        Vec2::new(left_offset, 0.0),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new(0.0, top / image_height),
        Vec2::new(left / image_width, inner_image_height / image_height),
    ));
    // middle
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        inner_width,
        inner_height,
        Vec2::ZERO,
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new(left / image_width, top / image_height),
        Vec2::new(
            inner_image_width / image_width,
            inner_image_height / image_height,
        ),
    ));
    // right
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        right,
        inner_height,
        Vec2::new(right_offset, 0.0),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new((image_width - right) / image_width, top / image_height),
        Vec2::new(right / image_width, inner_image_height / image_height),
    ));
    // bottom left
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        left,
        bottom,
        Vec2::new(left_offset, bottom_offset),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new(0.0, (image_height - bottom) / image_height),
        Vec2::new(left / image_width, bottom / image_height),
    ));
    // bottom
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        inner_width,
        bottom,
        Vec2::new(0.0, bottom_offset),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new(left / image_width, (image_height - bottom) / image_height),
        Vec2::new(inner_image_width / image_width, bottom / image_height),
    ));
    // bottom right
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut sized_quad_positions(
        right,
        bottom,
        Vec2::new(right_offset, bottom_offset),
    ));
    uvs.append(&mut quad_uvs(
        Vec2::new(
            (image_width - right) / image_width,
            (image_height - bottom) / image_height,
        ),
        Vec2::new(right / image_width, bottom / image_height),
    ));

    Mesh::from_arrays(
        &positions.as_slice(),
        &uvs.as_slice(),
        &indices.as_slice(),
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
