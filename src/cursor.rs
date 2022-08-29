use rusttype::{point, vector, Font, PositionedGlyph, Scale};
pub enum Position<T> {
    Absolute(T),
    Relative(T),
}
/// Struct representing cursor
pub struct Cursor {
    /// position in text buffer
    pub text_pos: (usize, usize),
    /// normalized position on screen
    pub screen_pos: (f32, f32),
    pub screen_width: f32,
    /// number of first line on a screen
    top_line: u32,
    /// font size
    size: u32,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            text_pos: (0, 0),
            screen_pos: (0.0, 0.0),
            screen_width: 12.0,
            size: 24,
            top_line: 0,
        }
    }

    pub fn move_to<'a>(
        &mut self,
        row: Position<usize>,
        col: Position<usize>,
        buff: &Vec<String>,
        font: &Font<'a>,
    ) {
        let row = match row {
            Position::Absolute(p) => p,
            Position::Relative(p) => self.text_pos.0 + p,
        };
        let col = match col {
            Position::Absolute(p) => p,
            Position::Relative(p) => self.text_pos.1 + p,
        };
        self.text_pos.0 = row;
        self.text_pos.1 = col;

        let line = &buff[row];
        let mut caret_x: f32 = 0.0;

        let mut last_glyph_id = None;
        for i in [0..col] {
            for c in line.chars() {
                let base_glyph = font.glyph(c);
                if let Some(id) = last_glyph_id.take() {
                    caret_x +=
                        font.pair_kerning(Scale::uniform(self.size as f32), id, base_glyph.id());
                }
                last_glyph_id = Some(base_glyph.id());
            }
        }
    }
}
