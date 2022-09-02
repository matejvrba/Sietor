use crate::Position;
use rusttype::{point, Font, Scale};
use std::cmp::min;
/// Struct representing cursor
pub struct Cursor {
    /// position in text buffer
    pub text_pos: (usize, usize),
    /// normalized position on screen
    pub screen_pos: (f32, f32),
    pub screen_width: f32,
    pub screen_scale: f32,
    /// number of first line on a screen
    top_line: u32,
    /// font size
    size: u32,
    pub width: f32,
    pub height: f32,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            text_pos: (0, 0),
            screen_pos: (0.0, 0.0),
            screen_width: 12.0,
            screen_scale: 1.0,
            size: 24,
            top_line: 0,
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn move_to<'a>(&mut self, row: Position<usize, i32>, col: Position<usize, i32>) {
        let row = match row {
            Position::Absolute(p) => p,
            Position::Relative(p) => self.text_pos.0 + p as usize,
        };
        let col = match col {
            Position::Absolute(p) => p,
            Position::Relative(p) => self.text_pos.1 + p as usize,
        };
        self.text_pos.0 = row;
        self.text_pos.1 = col;
    }
    pub fn calc_screen_pos<'a>(
        &mut self,
        font: &Font<'a>,
        buff: &Vec<String>,
        width: i32,
        height: i32,
    ) {
        let col = self.text_pos.1;
        let row = self.text_pos.0;

        let line = &buff[row];

        let mut last_glyph_id = None;
        let mut i = 0;
        let scale = Scale::uniform(self.size as f32 * self.screen_scale); //todo scale
        let v_metrics = font.v_metrics(scale);
        let mut caret = point(0.0, v_metrics.ascent);

        while i < min(col, line.len()) {
            let c = line.chars().nth(i).unwrap();
            let base_glyph = font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, base_glyph.id());
            }
            last_glyph_id = Some(base_glyph.id());
            let glyph = base_glyph.scaled(scale).positioned(caret);
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            i += 1;
        }
        // get char under cursor or previous if we're past last char.
        // It'll be ok... I mean. it won't... but still
        let char = match line.chars().nth(col) {
            Some(c) => c,
            None => {
                if col == 0 || line.len() == 0 {
                    ' '
                } else {
                    line.chars().nth(min(col, line.len()) - 1).unwrap()
                }
            }
        };

        let glyph = font.glyph(char).scaled(scale);
        let cursor_width = glyph.h_metrics().advance_width;

        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let cursor_height = advance_height;

        self.screen_pos.0 = (caret.x / width as f32 * 2.0) - 1.0;
        self.screen_pos.1 = 1.0 - row as f32 * advance_height / (height / 2) as f32;
        self.width = cursor_width;
        self.height = cursor_height;
    }
}
