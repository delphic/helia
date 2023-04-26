use glam::*;
use helia::State;
use helia::entity::*;
use helia::material::MaterialId;
use helia::mesh::MeshId;

#[derive(Clone)]
pub struct FontAtlas {
    pub mesh_id: MeshId,
    pub material_id: MaterialId,
    pub char_map: String,
    pub tile_width: u16,
    pub tile_height: u16,
    pub columns: u16,
    pub rows: u16,
}
// ^^ this could be any atlas if you replaced the char_map with id -> index map
// though we're likely to add individual char meta data so maybe hold off on that
// though maybe that means we want struct Font that contains an Atlas struct

pub struct TextMesh {
    pub text: String,
    font: FontAtlas,
    entities: Vec<(EntityId, Vec3)>,
    position: Vec3,
    scale: f32,
}

impl TextMesh {
    pub fn new(text: String, position: Vec3, font: FontAtlas, scale: f32, state: &mut State) -> Self {
        let mut text_mesh = Self {
            text: String::from(""),
            entities: Vec::new(),
            font,
            position,
            scale,
        };
        text_mesh.set_text(text, state);
        text_mesh
    }

    fn calculate_entity_position(&self, index: usize) -> Vec3 {
        let character_width = self.font.tile_width as f32 * self.scale;
        let alignment_offset = -character_width * self.text.len() as f32 / 2.0;
        // ^^ todo: more alignments!
        self.position + Vec3::new(alignment_offset + index as f32 * character_width, 0.0, 0.0)
    }

    pub fn set_text(&mut self, text: String, state: &mut State) {
        if !self.entities.is_empty() {
            self.clear_entities(state);
        }

        self.text = text;

        let tile_width = self.font.tile_width as f32;
        let tile_height = self.font.tile_height as f32;
        let character_width = (self.font.columns as f32).recip(); // in uv coords
        let character_height = (self.font.rows as f32).recip(); // in uv coords

        let chars = self.text.chars();
        // this is probably terrible practice for anything aother than ascii
        for (i, char) in chars.into_iter().enumerate() {
            if let Some(index) = self.font.char_map.find(char) {
                let x = (index % 22) as f32;
                let y = (index / 22) as f32;
                let id = state.scene.add_entity(
                    self.font.mesh_id,
                    self.font.material_id,
                    InstanceProperties::builder()
                        .with_translation(self.calculate_entity_position(i))
                        .with_uv_offset_scale(
                            Vec2::new(x * character_width, y * character_height),
                            Vec2::new(character_width, character_height),
                        )
                        .with_scale(self.scale * Vec3::new(tile_width, tile_height, 1.0))
                        .build(),
                );
                self.entities.push((id, Vec3::ZERO));
            }
        }
    }

    #[allow(dead_code)]
    pub fn clear_entities(&mut self, state: &mut State) {
        for (id, _) in &self.entities {
            state.scene.remove_entity(*id);
        }
    }

    #[allow(dead_code)]
    pub fn translate(&mut self, position: Vec3, state: &mut State) {
        self.position = position;
        for (i, (entity_id, offset)) in self.entities.iter().enumerate() {
            let entity = state.scene.get_entity_mut(*entity_id);
            let (scale, rotation, _) = entity.properties.transform.to_scale_rotation_translation();

            entity.properties.transform = Mat4::from_scale_rotation_translation(
                scale,
                rotation,
                self.calculate_entity_position(i) + *offset,
            )
        }
    }

    #[allow(dead_code)]
    pub fn offset_char(&mut self, index: usize, offset: Vec3, state: &mut State) {
        if index < self.entities.len() {
            let (id, prev_offset) = self.entities[index];
            let entity = state.scene.get_entity_mut(id);
            let (scale, rotation, translation) =
                entity.properties.transform.to_scale_rotation_translation();
            let delta = offset - prev_offset;
            entity.properties.transform =
                Mat4::from_scale_rotation_translation(scale, rotation, translation + delta);
            self.entities[index] = (id, offset);
        }
    }
}
