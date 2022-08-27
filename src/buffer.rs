use std::fs;
use nalgebra as na;

#[cfg(test)]
mod tests {
    

    

    #[test]
    fn test_new_buffer() {}
}

/// Indicates if buffer contains a source code
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
pub struct TextBuffer {
    pub buffer: Vec<String>,
    pub buffer_type: BufferType,
    pub cursor: (u32, u32),
    pub view_pos: (u32, u32),
    pub file: Option<fs::File>,
}

impl TextBuffer  {
    pub fn new(
        buffer: BufferOrigin,
        cursor: Option<(u32, u32)>,
        view_pos: Option<(u32, u32)>,
    ) -> Self {
        let mut buf = TextBuffer {
            buffer: Vec::<String>::new(),
            buffer_type: BufferType::Clear,
            cursor: (0, 0),
            view_pos: (0, 0),
            file: None,
        };

        if let Some((row, col)) = cursor {
            buf.cursor.0 = row;
            buf.cursor.1 = col;
        }
        if let Some((row, col)) = view_pos {
            buf.view_pos.0 = row;
            buf.view_pos.1 = col;
        }

        match buffer {
            BufferOrigin::File(f) => {
                buf.file = Some(f);
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

}

struct Word {
    color: ColorRGBA,
    bg_color: ColorRGBA,
}

pub struct ColorRGBA{
	pub color: na::Vector4<f32>,
}
impl ColorRGBA{
	fn new() -> Self{
		ColorRGBA{
			color: na::Vector4::new(1.0,1.0,1.0,1.0),
		}
	}
}
