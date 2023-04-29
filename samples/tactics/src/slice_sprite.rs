use glam::*;
use helia::mesh::*;
use helia::*;

use crate::utils;

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
        utils::build_9_slice_mesh(
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
