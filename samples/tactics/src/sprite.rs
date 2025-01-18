use glam::{Vec2, Vec3};
use helia::{entity::{Entity, InstanceProperties}, material::MaterialId, mesh::MeshId, transform::Transform, Color, DrawCommand};

pub struct Sprite {
	pub mesh_id: MeshId,
	pub material_id: MaterialId,
	pub position: Vec3,
	pub scale: Vec3,
	pub uv_scale: Vec2,
	pub uv_offset: Vec2,
	pub color: Color,
}

// TODO: Builder please!

impl Sprite {
	pub fn to_draw_command(&self) -> DrawCommand {
		DrawCommand::DrawEntity(Entity::new(
			self.mesh_id,
			self.material_id,
			InstanceProperties::builder()
				.with_uv_offset_scale(self.uv_offset, self.uv_scale)
				.with_color(self.color)
				.with_matrix(Transform::from_position_scale(self.position, self.scale).into())
				.build()
			)
		)
	}
}

