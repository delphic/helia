use glam::*;

use crate::entity::EntityId;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub parent: Option<EntityId>,
    /// for internal use only
    pub world_matrix: Mat4,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            parent: None,
            world_matrix: Mat4::IDENTITY,
        }
    }
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3, parent: Option<EntityId>) -> Self {
        Self {
            position,
            rotation,
            scale,
            parent,
            world_matrix: Mat4::IDENTITY,
        }
    }

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            parent: None,
            world_matrix: Mat4::IDENTITY,
        }
    }

    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            scale: Vec3::ONE,
            parent: None,
            world_matrix: Mat4::IDENTITY,
        }
    }

    pub fn from_position_scale(position: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale,
            parent: None,
            world_matrix: Mat4::IDENTITY,
        }
    }

    pub fn from_position_rotation_scale(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            parent: None,
            world_matrix: Mat4::IDENTITY,
        }
    }

    pub fn to_local_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}