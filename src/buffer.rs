use rusttype::Font;
use std::cmp::{max, min};
use std::fs;
use std::io::Read;
use crate::cursor::{Cursor, Position};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[cfg(test)]
mod tests {

    use super::BufferOrigin;
    use super::BufferType;
    use super::TextBuffer;

    #[test]
    fn test_new_buffer() {
        let buff = TextBuffer::new(BufferOrigin::Empty, None, None);
        assert_eq!(buff.buffer, Vec::<String>::new());
        assert_eq!(buff.cursor.text_pos, (0, 0));
        assert_eq!(buff.view_pos, (0, 0));
        assert!(buff.file.is_none());
        assert_eq!(buff.buffer_type, BufferType::Clear);
        assert!(buff.file.is_none());
        let buff = TextBuffer::new(BufferOrigin::Empty, Some((1, 2)), Some((3, 4)));
        assert_eq!(buff.cursor.text_pos, (1, 2));
        assert_eq!(buff.view_pos, (3, 4));
        let buff = TextBuffer::new(BufferOrigin::Buffer("test".to_string()), None, None);
        assert_eq!(buff.buffer, vec!["test".to_string()]);
        match std::fs::File::open("./Cargo.toml") {
            Ok(file) => {
                let buff = TextBuffer::new(BufferOrigin::File(file), None, None);
                assert!(buff.file.is_some());
                assert_eq!(
                    buff.buffer[0],
                    "[package]".to_string(),
                    "if this test fails, \
 check that Cargo.toml starts with \"[package]\" on first line"
                );
            }
            Err(_) => {
                assert!(
                    false,
                    "Could not find file \"./Cargo.toml\" to finish testing"
                );
            }
        }
    }
    #[test]
    fn test_moving_cursor() {
        let mut buff = TextBuffer::new(
            BufferOrigin::Buffer(
                "test tes
asdf asdl fasd
fas
fa"
                .to_string(),
            ),
            None,
            None,
        );
        assert_eq!(buff.cursor, (0, 0));
        buff.move_cursor_absolute(1, 2);
        assert_eq!(buff.cursor, (1, 2));
        buff.move_cursor_absolute(2, 1);
        assert_eq!(buff.cursor, (2, 1));
        buff.move_cursor_absolute(1000, 100);
        assert_eq!(buff.cursor, (3, 2));

        buff.move_cursor_relative(-1000, -10000);
        assert_eq!(buff.cursor, (0, 0));
        buff.move_cursor_relative(1, 2);
        assert_eq!(buff.cursor, (1, 2));
        buff.move_cursor_relative(1, 1);
        assert_eq!(buff.cursor, (2, 3));
        buff.move_cursor_relative(0, 1);
        assert_eq!(buff.cursor, (3, 0));
        buff.move_cursor_relative(-1, 0);
        assert_eq!(buff.cursor, (2, 0));
        buff.move_cursor_relative(0, -1);
        assert_eq!(buff.cursor, (1, 14));
    }
}

/// Indicates if bUffer contains a source code
#[derive(Debug, PartialEq)]
pub enum BufferType {
    Lang(Lang),
    Clear,
}

/// Indicates where is content of buffer from
#[derive(Debug)]
pub enum BufferOrigin {
    /// Content originates from file
    File(fs::File),
    /// Content generated/from memory...
    /// There's no file tied to this buffer, but it contains something
    Buffer(String),
    /// empty buffer
    Empty,
}

/// List of programming languages
#[derive(Debug, PartialEq)]
pub enum Lang {
    Rust,
}

// File buffer, optionaly tied to file
pub struct TextBuffer<'a> {
    pub buffer: Vec<String>,
    pub buffer_type: BufferType,
    ///row and column where curosr is located
    pub cursor: Cursor,
    pub view_pos: (usize, usize),
    pub file: Option<fs::File>,
    font: Font<'a>,
}

impl TextBuffer<'_> {
    pub fn new(
        buffer: BufferOrigin,
        cursor: Option<(usize, usize)>,
        view_pos: Option<(usize, usize)>,
    ) -> Self {

        trace!("Loading font \"/usr/share/fonts/TTF/Hack-Regular.ttf\"");
        let font_path = std::env::current_dir()
            .unwrap()
            .join("/usr/share/fonts/TTF/Hack-Regular.ttf");
        let data = std::fs::read(&font_path).unwrap();
        let font = Font::try_from_vec(data).unwrap();
        let mut buf = TextBuffer {
            buffer: Vec::<String>::new(),
            buffer_type: BufferType::Clear,
            cursor: Cursor::new(),
            view_pos: (0, 0),
            file: None,
            font,
        };

        if let Some((row, col)) = cursor {
            buf.cursor.move_to(Position::Absolute(row), Position::Absolute(col), &buf.buffer, &buf.font);
        }
        if let Some((row, col)) = view_pos {
            buf.view_pos.0 = row;
            buf.view_pos.1 = col;
        }

        match buffer {
            BufferOrigin::File(f) => {
                buf.file = Some(f);
                let mut buff = String::new();
                if let Some(ref mut file) = buf.file {
                    file.read_to_string(&mut buff);
                    for line in buff.lines() {
                        buf.buffer.push(line.to_string());
                    }
                }
            }
            BufferOrigin::Buffer(b) => {
                for line in b.lines() {
                    buf.buffer.push(line.to_string());
                }
            }
            BufferOrigin::Empty => {}
        }
        return buf;
    }

    pub fn insert(&mut self, ch: char, pos: Option<(usize, usize)>) {
        let (mut row, mut col) =  self.cursor.text_pos;

        if let Some((r, c)) = pos {
            row = r;
            col = c;
        }

        match ch {
            '\r' => {
                let mut tmp = self.buffer[row].clone();
                let (left, right) = tmp.split_at_mut(col);
                self.buffer[row] = right.to_string();
                self.buffer.insert(row, left.to_string());
                if let None = pos {
                    self.cursor.move_to(Position::Relative(1), Position::Absolute(0), &self.buffer, &self.font);
                }
            }
            _ => {
                self.buffer[row].insert(col, ch);
                if let None = pos {
                    self.cursor.move_to(Position::Absolute(0), Position::Relative(1), &self.buffer, &self.font)
                }
            }
        }
    }

    /// sets cursor to absolute x and y or to the end of line/buffer
    /// cursor is first moved vertically and then horizontaly, if horizontal
    /// move is past the end of line, it continues on the next line(except for
    /// the end of file, where is stops)
    pub fn move_cursor_absolute(&mut self, vertical: usize, mut horizontal: usize) {
        let mut row = min(vertical, self.buffer.len() - 1);
        let mut col: usize = 0;
        let mut line_len = self.buffer[row].len();
        while line_len < horizontal {
            if self.buffer.len() - 1 == row {
                col = line_len;
                return;
            }
            horizontal -= line_len + 1;
            row += 1;
            line_len = self.buffer[row].len();
        }
        col = horizontal;
        self.cursor.move_to(Position::Absolute(row), Position::Absolute(col), &self.buffer, &self.font)
    }

    /// sets cursor to relative x and y to current position or to the beginning/end of line/buffer
    pub fn move_cursor_relative(&mut self, vertical: i32, mut horizontal: i32) {
        //could cause problems, where file is longer than i32::max/2
        let mut vertical = max(vertical + self.cursor.text_pos.0 as i32, 0) as usize;
        if horizontal < 0 {
            if vertical == 0 {
                horizontal = 0;
            } else {
                vertical -= 1;
                horizontal = self.buffer[vertical].len() as i32;
            }
        } else {
            horizontal = horizontal + self.cursor.text_pos.1 as i32;
        }
        self.move_cursor_absolute(vertical, horizontal as usize);
    }

    pub fn delete(&mut self, _start: Option<(usize, usize)>, _end: Option<(usize, usize)>) {
        todo!();
    }
}
