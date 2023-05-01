use core::atlas::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FontAtlas {
    pub atlas: Atlas, // text mesh assumes atlas mesh id a center anchored 1x1 quad
    pub char_map: String,
    pub custom_char_widths: Option<HashMap<char, u16>>,
}

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
