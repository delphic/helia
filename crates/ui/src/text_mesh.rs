use core::entity::*;
use core::transform_hierarchy::HierarchyId;
use core::State;
use glam::*;

use crate::font::*;

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
    entities: Vec<(EntityId, HierarchyId, Vec3)>,
    scale: f32,
    alignment: TextAlignment,
    vertical_alignment: VerticalAlignment,
}

impl TextMesh {
    // TODO: Remove state and putting in scene
    // instead have a something to generate the relevant draw commands 

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

    pub fn builder(text: String, position: Vec3, font: FontAtlas) -> TextMeshBuilder {
        TextMeshBuilder::new(text, position, font)
    }

    fn calculate_alignment_offset(&self) -> Vec3 {
        let character_width = self.font.atlas.tile_width as f32 * self.scale;
        let x_offset = match self.alignment {
            TextAlignment::Left => character_width / 2.0,
            TextAlignment::Center => -self.measure_text(&self.text) / 2.0,
            TextAlignment::Right => character_width / 2.0 - self.measure_text(&self.text),
        };
        let character_height = self.font.atlas.tile_height as f32 * self.scale;
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
        self.font.atlas.tile_width as f32 * self.scale
    }

    #[allow(dead_code)]
    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn measure_text(&self, text: &String) -> f32 {
        if let Some(custom_widths) = &self.font.custom_char_widths {
            text.chars()
                .map(|char| {
                    custom_widths
                        .get(&char)
                        .unwrap_or(&self.font.atlas.tile_width)
                })
                .map(|w| *w as f32 * self.scale)
                .sum()
        } else {
            self.font.atlas.tile_width as f32 * self.scale * text.len() as f32
        }
    }

    pub fn set_text(&mut self, text: String, state: &mut State) {
        if !self.entities.is_empty() && self.entities.len() > text.len() {
            let from = text.len();
            let to = self.entities.len();
            for i in from..to {
                state.scene.remove_entity(self.entities[i].0);
            }
            self.entities.truncate(text.len());
        }

        self.text = text;

        let mut position = self.position + self.calculate_alignment_offset();
        let chars = self.text.chars();
        // this is probably terrible practice for anything other than ascii
        for (i, char) in chars.enumerate() {
            if let Some(index) = self.font.char_map.find(char) {
                if i < self.entities.len() {
                    let mut transform = state.scene.hierarchy.get_transform(self.entities[i].1).unwrap();
                    transform.position = position;
                    state.scene.hierarchy.set_transform(self.entities[i].1, transform);
                    let entity = state.scene.get_entity_mut(self.entities[i].0);
                    entity.properties.uv_offset = self.font.atlas.uv_offset_scale(index).0;
                    self.entities[i].2 = Vec3::ZERO; // reset offset
                } else {
                    let (transform, props) = self.font
                        .atlas
                        .instance_properties(index, position, self.scale);
                    let (id, hierarchy_id) = state.scene.add_entity(
                        self.font.atlas.mesh_id,
                        self.font.atlas.material_id,
                        transform,
                        props
                    );
                    self.entities.push((id, hierarchy_id, Vec3::ZERO));
                }
                position += self.get_char_width(char) * Vec3::X
            }
        }
    }

    #[allow(dead_code)]
    pub fn clear_entities(&mut self, state: &mut State) {
        for (id, _, _) in &self.entities {
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
            let mut position = self.position + self.calculate_alignment_offset();
            for (i, (_, hierarchy_id, offset)) in self.entities.iter().enumerate() {
                if let Some(char) = self.text.chars().nth(i) {
                    let mut transform = state.scene.hierarchy.get_transform(self.entities[i].1).unwrap();
                    transform.position = position + offset;
                    state.scene.hierarchy.set_transform(*hierarchy_id, transform);
                    position += self.get_char_width(char) * Vec3::X;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn offset_char(&mut self, index: usize, offset: Vec3, state: &mut State) {
        if index < self.entities.len() {
            let (id, hierarchy_id, prev_offset) = self.entities[index];
            let delta = offset - prev_offset;
            let mut transform = state.scene.hierarchy.get_transform(hierarchy_id).unwrap();
            transform.position += delta;
            state.scene.hierarchy.set_transform(hierarchy_id, transform);
            self.entities[index] = (id, hierarchy_id, offset);
        }
    }
}
