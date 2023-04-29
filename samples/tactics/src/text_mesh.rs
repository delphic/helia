use glam::*;
use helia::entity::*;
use helia::material::MaterialId;
use helia::mesh::MeshId;
use helia::State;
use std::collections::HashMap;

#[derive(Clone)]
pub struct FontAtlas {
    pub mesh_id: MeshId, // assumed center anchored 1x1 quad
    pub material_id: MaterialId,
    pub char_map: String,
    pub tile_width: u16,
    pub tile_height: u16,
    pub columns: u16,
    pub rows: u16,
    pub custom_char_widths: Option<HashMap<char, u16>>,
}
// ^^ this could almost be any atlas if you replaced the char_map with id -> index map
// though we're likely to add individual char meta data so maybe hold off on that
// though maybe that means we want struct Font that contains an Atlas struct

impl FontAtlas {
    pub fn build_char_widths(width_to_chars: HashMap<u16, String>) -> HashMap<char, u16> {
        let mut result = HashMap::new();
        for (width, str) in width_to_chars {
            for char in str.chars() {
                result.insert(char, width);
            }
        }
        result
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

pub struct TextMeshBuilder {
    text: String,
    font: FontAtlas,
    position: Vec3,
    scale: f32,
    alignment: TextAlignment,
    vertical_alignment: VerticalAlignment,
}

impl TextMeshBuilder {
    pub fn new(text: String, position: Vec3, font: FontAtlas) -> Self {
        Self {
            text,
            font,
            position,
            scale: 1.0,
            alignment: TextAlignment::Left,
            vertical_alignment: VerticalAlignment::Bottom,
        }
    }

    pub fn build(&self, state: &mut State) -> TextMesh {
        TextMesh::new(
            self.text.clone(),
            self.position,
            self.font.clone(),
            self.scale,
            self.alignment,
            self.vertical_alignment,
            state,
        )
    }

    #[allow(dead_code)]
    pub fn with_scale(&mut self, scale: f32) -> &mut Self {
        self.scale = scale;
        self
    }

    pub fn with_alignment(&mut self, alignment: TextAlignment) -> &mut Self {
        self.alignment = alignment;
        self
    }

    pub fn with_vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = vertical_alignment;
        self
    }
}

pub struct TextMesh {
    pub text: String,
    position: Vec3,
    font: FontAtlas,
    entities: Vec<(EntityId, Vec3)>,
    scale: f32,
    alignment: TextAlignment,
    vertical_alignment: VerticalAlignment,
}

impl TextMesh {
    pub fn new(
        text: String,
        position: Vec3,
        font: FontAtlas,
        scale: f32,
        alignment: TextAlignment,
        vertical_alignment: VerticalAlignment,
        state: &mut State,
    ) -> Self {
        let mut text_mesh = Self {
            text: String::from(""),
            entities: Vec::new(),
            font,
            position,
            scale,
            alignment,
            vertical_alignment,
        };
        text_mesh.set_text(text, state);
        text_mesh
    }
    // todo: builder pattern, scale and alignment as options

    pub fn builder(text: String, position: Vec3, font: FontAtlas) -> TextMeshBuilder {
        TextMeshBuilder::new(text, position, font)
    }

    fn calculate_alignemnt_offset(&self) -> Vec3 {
        let character_width = self.font.tile_width as f32 * self.scale;
        let x_offset = match self.alignment {
            TextAlignment::Left => character_width / 2.0,
            TextAlignment::Center => -self.measure_text(&self.text) / 2.0,
            TextAlignment::Right => character_width / 2.0 - self.measure_text(&self.text),
        };
        let character_height = self.font.tile_height as f32 * self.scale;
        let y_offset = match self.vertical_alignment {
            VerticalAlignment::Top => -character_height,
            VerticalAlignment::Center => 0.0,
            VerticalAlignment::Bottom => character_height,
        };

        Vec3::new(x_offset, y_offset, 0.0)
    }

    fn get_char_width(&self, char: char) -> f32 {
        if let Some(custom_widths) = &self.font.custom_char_widths {
            if let Some(width) = custom_widths.get(&char) {
                return *width as f32 * self.scale;
            }
        }
        self.font.tile_width as f32 * self.scale
    }

    #[allow(dead_code)]
    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn measure_text(&self, text: &String) -> f32 {
        if let Some(custom_widths) = &self.font.custom_char_widths {
            text.chars()
                .map(|char| custom_widths.get(&char).unwrap_or(&self.font.tile_width))
                .map(|w| *w as f32 * self.scale)
                .sum()
        } else {
            self.font.tile_width as f32 * self.scale * text.len() as f32
        }
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

        let mut position = self.position + self.calculate_alignemnt_offset();
        let chars = self.text.chars();
        // this is probably terrible practice for anything aother than ascii
        for char in chars {
            if let Some(index) = self.font.char_map.find(char) {
                let x = (index % 22) as f32;
                let y = (index / 22) as f32;
                let id = state.scene.add_entity(
                    self.font.mesh_id,
                    self.font.material_id,
                    InstanceProperties::builder()
                        .with_translation(position)
                        .with_uv_offset_scale(
                            Vec2::new(x * character_width, y * character_height),
                            Vec2::new(character_width, character_height),
                        )
                        .with_scale(self.scale * Vec3::new(tile_width, tile_height, 1.0))
                        .build(),
                );
                self.entities.push((id, Vec3::ZERO));
                position += self.get_char_width(char) * Vec3::X
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
        if self.text.len() != self.entities.len() {
            self.set_text(self.text.clone(), state);
            log::warn!("Tried to translate text mesh, but text did not match entity length, use set_text fn to alter text value");
        } else {
            let mut position = self.position + self.calculate_alignemnt_offset();
            for (i, (entity_id, offset)) in self.entities.iter().enumerate() {
                if let Some(char) = self.text.chars().nth(i) {
                    let entity = state.scene.get_entity_mut(*entity_id);
                    let (scale, rotation, _) =
                        entity.properties.transform.to_scale_rotation_translation();

                    entity.properties.transform =
                        Mat4::from_scale_rotation_translation(scale, rotation, position + *offset);
                    position += self.get_char_width(char) * Vec3::X;
                }
            }
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
