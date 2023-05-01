use core::{
    mesh::{Mesh, MeshId},
    *,
};
use glam::*;
use primitives::quad::*;

#[derive(Clone, Copy, Debug)]
pub struct SliceConfig {
    pub width: f32,
    pub height: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SliceSpriteMesh {
    pub mesh: MeshId,
    pub size: Vec2,
    pub config: SliceConfig,
}

impl SliceSpriteMesh {
    pub fn new(size: Vec2, config: SliceConfig, state: &mut State) -> Self {
        let mesh = Self::build_mesh(size, config, state);
        let mesh_id = state.resources.meshes.insert(mesh);
        Self {
            mesh: mesh_id,
            size,
            config,
        }
    }

    #[allow(dead_code)]
    /// Generates a new instance with an independent mesh resource
    pub fn duplicate(&self, state: &mut State) -> Self {
        Self::new(self.size, self.config, state)
    }

    fn build_mesh(size: Vec2, config: SliceConfig, state: &mut State) -> Mesh {
        build_mesh(
            size.x,
            size.y,
            config.width,
            config.height,
            config.top,
            config.right,
            config.bottom,
            config.left,
            state,
        )
    }

    /// Updates the corresponding mesh id's mesh representation with new size
    /// Note: will affect all sprites using this SliceSpriteMesh / MeshId instance
    pub fn resize(&mut self, size: Vec2, state: &mut State) {
        self.size = size;
        let mesh = Self::build_mesh(size, self.config, state);
        state.resources.meshes[self.mesh] = mesh;
    }
}

pub fn build_mesh(
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
    positions.append(&mut positions_with_offset_scale(
        left,
        top,
        Vec2::new(left_offset, top_offset),
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::ZERO,
        Vec2::new(left / image_width, top / image_height),
    ));
    // top
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        inner_width,
        top,
        Vec2::new(0.0, top_offset),
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::new(left / image_width, 0.0),
        Vec2::new(inner_image_width / image_width, top / image_height),
    ));
    // top right
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        right,
        top,
        Vec2::new(right_offset, top_offset),
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::new((image_width - right) / image_width, 0.0),
        Vec2::new(right / image_width, top / image_height),
    ));
    // left
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        left,
        inner_height,
        Vec2::new(left_offset, 0.0),
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::new(0.0, top / image_height),
        Vec2::new(left / image_width, inner_image_height / image_height),
    ));
    // middle
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        inner_width,
        inner_height,
        Vec2::ZERO,
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::new(left / image_width, top / image_height),
        Vec2::new(
            inner_image_width / image_width,
            inner_image_height / image_height,
        ),
    ));
    // right
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        right,
        inner_height,
        Vec2::new(right_offset, 0.0),
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::new((image_width - right) / image_width, top / image_height),
        Vec2::new(right / image_width, inner_image_height / image_height),
    ));
    // bottom left
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        left,
        bottom,
        Vec2::new(left_offset, bottom_offset),
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::new(0.0, (image_height - bottom) / image_height),
        Vec2::new(left / image_width, bottom / image_height),
    ));
    // bottom
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        inner_width,
        bottom,
        Vec2::new(0.0, bottom_offset),
    ));
    uvs.append(&mut uvs_with_offset_scale(
        Vec2::new(left / image_width, (image_height - bottom) / image_height),
        Vec2::new(inner_image_width / image_width, bottom / image_height),
    ));
    // bottom right
    extend_indices(&mut indices, positions.len() as u16);
    positions.append(&mut positions_with_offset_scale(
        right,
        bottom,
        Vec2::new(right_offset, bottom_offset),
    ));
    uvs.append(&mut uvs_with_offset_scale(
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
