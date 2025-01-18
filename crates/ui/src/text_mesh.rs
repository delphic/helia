use core::transform::Transform;
use core::{entity::*, DrawCommand};
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

    pub fn build(&self) -> TextMesh {
        TextMesh::new(
            self.text.clone(),
            self.position,
            self.font.clone(),
            self.scale,
            self.alignment,
            self.vertical_alignment,
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

pub struct TextMeshElement {
    transform: Transform,
    entity: Entity,
    offset: Vec3
}

pub struct TextMesh {
    pub text: String,
    position: Vec3,
    font: FontAtlas,
    elements: Vec<TextMeshElement>,
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
    ) -> Self {
        let mut text_mesh = Self {
            text: String::from(""),
            elements: Vec::new(),
            font,
            position,
            scale,
            alignment,
            vertical_alignment,
        };
        text_mesh.set_text(text);
        text_mesh
    }

    // Could take a world transform if we wanted
    pub fn render(&self, draw_commands: &mut Vec<DrawCommand>) {
        for element in self.elements.iter() {
            draw_commands.push(DrawCommand::DrawEntity(element.entity));
        }
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

    fn get_char_width(char: char, font: &FontAtlas, scale: f32) -> f32 {
        if let Some(custom_widths) = &font.custom_char_widths {
            if let Some(width) = custom_widths.get(&char) {
                return *width as f32 * scale;
            }
        }
        font.atlas.tile_width as f32 * scale
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

    pub fn set_text(&mut self, text: String) {
        if !self.elements.is_empty() && self.elements.len() > text.len() {
            self.elements.truncate(text.len());
        }

        self.text = text;

        let mut position = self.position + self.calculate_alignment_offset();
        let chars = self.text.chars();
        // this is probably terrible practice for anything other than ascii
        for (i, char) in chars.enumerate() {
            if let Some(index) = self.font.char_map.find(char) {
                if i < self.elements.len() {
                    let element = &mut self.elements.get_mut(i).unwrap();
                    element.transform.position = position;
                    element.entity.properties.uv_offset = self.font.atlas.uv_offset_scale(index).0;
                    element.entity.properties.world_matrix = element.transform.to_local_matrix();
                    element.offset = Vec3::ZERO; // reset offset
                } else {
                    let (transform, props) = self.font
                        .atlas
                        .instance_properties(index, position, self.scale);
                    let entity = Entity::new(
                        self.font.atlas.mesh_id,
                        self.font.atlas.material_id,
                        props
                    );
                    self.elements.push(TextMeshElement { transform, entity, offset: Vec3::ZERO });
                }
                position += Self::get_char_width(char, &self.font, self.scale) * Vec3::X
            }
        }
    }


    #[allow(dead_code)]
    pub fn translate(&mut self, position: Vec3) {
        self.position = position;
        if self.text.len() != self.elements.len() {
            self.set_text(self.text.clone());
            log::warn!("Tried to translate text mesh, but text did not match entity length, use set_text fn to alter text value");
        } else {
            let mut position = self.position + self.calculate_alignment_offset();
            for (i, element) in self.elements.iter_mut().enumerate() {
                if let Some(char) = self.text.chars().nth(i) {
                    element.transform.position = position + element.offset;
                    element.entity.properties.world_matrix = element.transform.to_local_matrix();
                    position += Self::get_char_width(char, &self.font, self.scale) * Vec3::X;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn offset_char(&mut self, index: usize, target_offset: Vec3) {
        if index < self.elements.len() {
            let entry = &mut self.elements[index];
            let delta = target_offset - entry.offset;
            entry.transform.position += delta;
            entry.entity.properties.world_matrix = entry.transform.to_local_matrix();
            entry.offset = target_offset;
        }
    }
}
