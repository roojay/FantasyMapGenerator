use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct TextExtents {
    pub offx: f64,
    pub offy: f64,
    pub width: f64,
    pub height: f64,
    pub dx: f64,
    pub dy: f64,
}

pub struct FontFace {
    data: Value,
    font_face: String,
    font_size: String,
    default_font: String,
    default_font_size: String,
}

impl FontFace {
    pub fn new(json_str: &str) -> Self {
        let data: Value = serde_json::from_str(json_str).expect("valid fontdata JSON");
        let obj = data.as_object().expect("fontdata is object");

        let default_font = if obj.contains_key("Arial") {
            "Arial".to_string()
        } else {
            obj.keys().next().cloned().unwrap_or_default()
        };

        let default_font_size = data[&default_font]
            .as_object()
            .and_then(|o| o.keys().next().cloned())
            .unwrap_or_default();

        FontFace {
            data,
            font_face: default_font.clone(),
            font_size: default_font_size.clone(),
            default_font,
            default_font_size,
        }
    }

    pub fn get_font_face(&self) -> &str {
        &self.font_face
    }

    pub fn get_font_size(&self) -> i32 {
        self.font_size.parse().unwrap_or(0)
    }

    pub fn set_font_face_size(&mut self, name: &str, size: i32) -> bool {
        if self.data.get(name).is_none() { return false; }
        let size_str = size.to_string();
        if self.data[name].get(&size_str).is_none() { return false; }
        self.font_face = name.to_string();
        self.font_size = size_str;
        true
    }

    pub fn get_text_extents(&self, text: &str) -> TextExtents {
        if text.is_empty() {
            return TextExtents::default();
        }
        if text.len() == 1 {
            return self.get_char_extents(text.chars().next().unwrap());
        }

        let first = self.get_char_extents(text.chars().next().unwrap());
        let mut extents = TextExtents::default();
        extents.offx = first.offx;

        let mut ymin = 0.0f64;
        let mut ymax = 0.0f64;
        let mut dx = 0.0f64;
        for c in text.chars() {
            let ce = self.get_char_extents(c);
            ymin = ymin.min(ce.offy);
            ymax = ymax.max(ce.offy + ce.height);
            dx += ce.dx;
        }
        extents.offy = ymin;

        let last = self.get_char_extents(text.chars().last().unwrap());
        extents.width = dx + extents.offx - (last.dx - last.width);
        extents.height = ymax - ymin;
        extents.dx = dx;
        extents.dy = 0.0;
        extents
    }

    pub fn get_character_extents(&self, text: &str) -> Vec<TextExtents> {
        let mut extents = Vec::new();
        let mut x = 0.0f64;
        for c in text.chars() {
            let mut ce = self.get_char_extents(c);
            ce.offx += x;
            x += ce.dx;
            extents.push(ce);
        }
        extents
    }

    fn get_char_extents(&self, c: char) -> TextExtents {
        let key = c.to_string();
        let char_data = &self.data[&self.font_face][&self.font_size][&key];
        let arr = char_data.as_array();
        let mut data = [0.0f64; 6];
        if let Some(arr) = arr {
            for (i, v) in arr.iter().enumerate().take(6) {
                data[i] = v.as_f64().unwrap_or(0.0);
            }
        }
        TextExtents {
            offx: data[0],
            offy: data[1],
            width: data[2],
            height: data[3],
            dx: data[4],
            dy: data[5],
        }
    }
}
