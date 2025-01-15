use crate::{entity::InstanceProperties, material::MaterialId, mesh::MeshId, transform::Transform};
use glam::{Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Atlas {
    pub mesh_id: MeshId, // assumed center anchored 1x1 quad
    pub material_id: MaterialId,
    pub tile_width: u16,
    pub tile_height: u16,
    pub columns: u16,
    pub rows: u16,
}

impl Atlas {
    pub fn uv_offset_scale(&self, index: usize) -> (Vec2, Vec2) {
        let x = (index % self.columns as usize) as f32;
        let y = (index / self.columns as usize) as f32;
        let tile_uv_width = (self.columns as f32).recip();
        let tile_uv_height = (self.rows as f32).recip();
        (
            Vec2::new(x * tile_uv_width, y * tile_uv_height),
            Vec2::new(tile_uv_width, tile_uv_height),
        )
    }

    pub fn tile_size(&self) -> Vec2 {
        Vec2::new(self.tile_width as f32, self.tile_height as f32)
    }

    pub fn instance_properties(
        &self,
        index: usize,
        position: Vec3,
        scale: f32,
    ) -> (Transform, InstanceProperties) {
        let (uv_offset, uv_scale) = self.uv_offset_scale(index);
        let transform = Transform::from_position_scale(
            position,
            scale * self.tile_size().extend(1.0),
        );
        let props = InstanceProperties::builder()
            .with_matrix(transform.to_local_matrix())
            .with_uv_offset_scale(uv_offset, uv_scale)
            .build();
        (transform , props)
    }
}
