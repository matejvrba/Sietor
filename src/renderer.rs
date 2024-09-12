use crate::buffer::TextBuffer;
use crate::window;
use glium::*;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rusttype::gpu_cache::Cache;
use rusttype::{point, vector, Font, PositionedGlyph, Scale};
use std::borrow::Cow;
use syntect::highlighting::{ThemeSet, Highlighter};
use syntect::parsing::SyntaxSet;

/// Intented to represent size on screen, can be either normalized (e.g. 0.0 is
/// center, 1.0 is right and -1.0 is left of screen)
#[derive(Clone, Copy, Debug)]
pub enum ScreenSize {
    Normalized(f32),
    Px(i32),
}
/// Struct representing on screen rectangle
/// Has `x` and `y` position, `width` and `height`
/// all values are of type [ScreenSize](ScreenSize) which can be either normalized or absolute
#[derive(Debug)]
pub struct Rect {
    x: ScreenSize,
    y: ScreenSize,
    width: ScreenSize,
    height: ScreenSize,
}

impl Rect {
    /// convert coordinate to normalized
    fn normalize(val: ScreenSize, size: u32) -> f32 {
        let size = size as i32;
        match val {
            ScreenSize::Normalized(v) => v,
            ScreenSize::Px(v) => ((v - size / 2) as f32 / (size / 2) as f32) as f32,
        }
    }
    /// convert coordinate to absoluite
    fn pxize(val: ScreenSize, size: u32) -> f32 {
        match val {
            ScreenSize::Normalized(v) => (v * size as f32),
            ScreenSize::Px(v) => v as f32,
        }
    }

    /// returns tuple containing normalized coordinates (x, y, width, height)
    pub fn to_noramalized(&self, screen_width: u32, screen_height: u32) -> (f32, f32, f32, f32) {
        (
            Self::normalize(self.x, screen_width),
            Self::normalize(self.y, screen_height) * -1.0,
            Self::normalize(self.width, screen_width),
            Self::normalize(self.height, screen_height) * -1.0,
        )
    }
    /// returns tuple containing coordinates converted to pixels (x, y, width, height)
    pub fn to_px(&self, screen_width: u32, screen_height: u32) -> (f32, f32, f32, f32) {
        (
            Self::pxize(self.x, screen_width),
            Self::pxize(self.y, screen_height),
            Self::pxize(self.width, screen_width),
            Self::pxize(self.height, screen_height),
        )
    }

    /// normalize all coordinates
    pub fn self_to_noramalized(&mut self, screen_width: u32, screen_height: u32) {
        self.x = ScreenSize::Normalized(Self::normalize(self.x, screen_width));
        self.y = ScreenSize::Normalized(Self::normalize(self.y, screen_height));
        self.width = ScreenSize::Normalized(Self::normalize(self.width, screen_width));
        self.height = ScreenSize::Normalized(Self::normalize(self.height, screen_height));
    }
    /// convert all values to ScreenSize::Px
    pub fn self_to_px(&mut self, screen_width: u32, screen_height: u32) {
        self.x = ScreenSize::Px(Self::pxize(self.x, screen_width) as i32);
        self.y = ScreenSize::Px(Self::pxize(self.y, screen_height) as i32);
        self.width = ScreenSize::Px(Self::pxize(self.width, screen_width) as i32);
        self.height = ScreenSize::Px(Self::pxize(self.height, screen_height) as i32);
    }
}

/// Struct for representing RGBA color with normalized values
pub struct ColorRGBA {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl ColorRGBA {
    /// returns copy of colors in array `[r, g, b, a]`
    pub fn as_arr(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
    /// returns new black color
    fn new() -> Self {
        ColorRGBA {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
    fn from_8bit(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.r = r as f32 / 255.0;
        self.g = g as f32 / 255.0;
        self.b = b as f32 / 255.0;
        self.a = a as f32 / 255.0;
        self
    }
}

/// struct resposible for rendering text and decoration
pub struct Renderer<'a> {
    cache: Cache<'a>,
    cache_tex: Texture2d,
    /// OpenGL texture used for caching of font
    text_program: Program,
    ///shader for drwaing text
    decor_program: Program,
    ///sjader for drawing solid rectangles
    //highlighter: Highlighter<'a>,
    theme: syntect::highlighting::Theme,
    ps: SyntaxSet,
}

impl<'a> Renderer<'a> {
    pub fn new(win: &window::WindowContext) -> Result<Self, failure::Error> {
        trace!("Initializing syntect");
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        //let _syntax = ps.find_syntax_by_extension("rs").unwrap();

        let theme = ts.themes["base16-ocean.dark"].clone();
        trace!("Initializing gpu font cache");
        let scale = win.display.gl_window().window().scale_factor();
        let (cache_width, cache_height) = ((512.0 * scale) as u32, (512.0 * scale) as u32);
        let cache = Cache::builder()
            .dimensions(cache_width, cache_height)
            .build();

        let cache_tex = glium::texture::Texture2d::with_format(
            &win.display,
            glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
                width: cache_width,
                height: cache_height,
                format: glium::texture::ClientFormat::U8,
            },
            glium::texture::UncompressedFloatFormat::U8,
            glium::texture::MipmapsOption::NoMipmap,
        )?;

        trace!("Compiling font shader");
        let text_program = program!(
        &win.display,
        140 => {
            vertex: "
#version 150

in vec2 position;
in vec2 tex_coords;
in vec4 color;

out VS_OUTPUT{
    vec2 tex_coords;
    vec4 color;
}OUT;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    OUT.tex_coords = tex_coords;
    OUT.color = color;
}
            ",
            fragment: "
#version 150
uniform sampler2D tex;
                
in VS_OUTPUT{
    vec2 tex_coords;
    vec4 color;
} IN;

out vec4 color;

void main() {
    color = vec4(IN.color.rgb, texture(tex, IN.tex_coords).r);
}
                        "
        })?;

        let decor_program = program!(
        &win.display,
        140 => {
            vertex: "
#version 140

in vec2 position;
in vec4 color;

out vec4 v_color;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_color = color;
}
            ",
            fragment: "
#version 140
in vec4 v_color;
out vec4 f_color;

void main() {
    f_color = v_color;
}
                        "
        })?;

        Ok(Self {
            ps,
            cache,
            cache_tex,
            text_program,
            decor_program,
            theme,
        })
    }

    fn layout_paragraph(
        &self,
        font: &Font<'a>,
        scale: Scale,
        width: u32,
        text: &Vec<String>,
    ) -> Vec<(PositionedGlyph<'a>, syntect::highlighting::Style)> {
        let syntax = self.ps.find_syntax_by_extension("rs").unwrap();
        let ps = SyntaxSet::load_defaults_newlines();
        let mut highlight =
            syntect::easy::HighlightLines::new(syntax, &self.theme);

        let mut result = Vec::new();
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(0.0, v_metrics.ascent);
        let mut last_glyph_id = None;
        for l in text {
            let line = highlight.highlight_line(l, &self.ps).unwrap();
            for word in line {
                let style = word.0;
                for c in word.1.chars() {
                    if c.is_control() {
                        //check if line contains \n - should not contain, [text] should be vector of inidvidual lines
                        match c {
                            '\n' => {
                                caret = point(0.0, caret.y + advance_height);
                                error!("Line \"{}\" is not separated properly, should be splitted into two", l);
                            }
                            '\r' => {
                                caret = point(0.0, caret.y + advance_height);
                                error!("Line \"{}\" is not separated properly, should be splitted into two", l);
                            }
                            _ => {}
                        }
                        continue;
                    }
                    let base_glyph = font.glyph(c);
                    if let Some(id) = last_glyph_id.take() {
                        caret.x += font.pair_kerning(scale, id, base_glyph.id());
                    }
                    last_glyph_id = Some(base_glyph.id());
                    let mut glyph = base_glyph.scaled(scale).positioned(caret);
                    if let Some(bb) = glyph.pixel_bounding_box() {
                        if bb.max.x > width as i32 {
                            caret = point(0.0, caret.y + advance_height);
                            glyph.set_position(caret);
                            last_glyph_id = None;
                        }
                    }
                    caret.x += glyph.unpositioned().h_metrics().advance_width;
                    result.push((glyph, style));
                }
            }
            caret = point(0.0, caret.y + advance_height);
        }
        result
    }
    pub fn draw(
        &mut self,
        x: f32,
        y: f32,
        width_factor: f32,
        scale: f32,
        disp: &Display,
        buff: &mut TextBuffer<'a>,
    ) {
        let mut target = disp.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        let rect = Rect {
            x: ScreenSize::Px(10),
            y: ScreenSize::Px(10),
            width: ScreenSize::Px(20),
            height: ScreenSize::Px(20),
        };
        let col = ColorRGBA::new().from_8bit(255, 0, 0, 255);
        self.draw_rect(&rect, &col, disp, &mut target);
        self.draw_cursor(buff, disp, &mut target);
        self.draw_text(
            x,
            y,
            width_factor,
            scale,
            disp,
            buff,
            &mut target,
        );
        target.finish().unwrap();
    }

    ///draws text, `x` and `y` is normalized position of top left corner
    /// `width_factor` and `height_factor` is normalized width and height
    /// `scale is size of font`
    fn draw_text(
        &mut self,
        x: f32,
        y: f32,
        width_factor: f32,
        scale: f32,
        disp: &Display,
        buff: &TextBuffer<'a>,
        target: &mut Frame,
    ) {
        //get size of window
        let (mut width, _): (f32, f32) = disp.gl_window().window().inner_size().into();
        width = width_factor * width;
        let width = width.ceil() as u32;

        let scale_dis = disp.gl_window().window().scale_factor() as f32;

        let glyphs = self.layout_paragraph(
            &buff.font,
            Scale::uniform(scale * scale_dis),
            width,
            &buff.buffer,
        );

        for glyph in &glyphs {
            self.cache.queue_glyph(0, glyph.0.clone());
        }

        self.cache
            .cache_queued(|rect, data| {
                self.cache_tex.main_level().write(
                    glium::Rect {
                        left: rect.min.x,
                        bottom: rect.min.y,
                        width: rect.width(),
                        height: rect.height(),
                    },
                    glium::texture::RawImage2d {
                        data: Cow::Borrowed(data),
                        width: rect.width(),
                        height: rect.height(),
                        format: glium::texture::ClientFormat::U8,
                    },
                );
            })
            .unwrap();

        let uniforms = uniform! {
            tex: self.cache_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
                tex_coords: [f32; 2],
                color: [f32; 4],
            }

            implement_vertex!(Vertex, position, tex_coords, color);
            let mut color = [0.0, 0.0, 0.0, 1.0];
            let (screen_width, screen_height) = {
                let (w, h) = disp.get_framebuffer_dimensions();
                (w as f32, h as f32)
            };
            let origin = point(x, y);
            let vertices: Vec<Vertex> = glyphs
                .iter()
                .filter_map(|g| match self.cache.rect_for(0, &g.0).ok().flatten() {
                    Some(rect) => Some((rect, &g.1)),
                    None => None,
                })
                .flat_map(|(rect, style)| {
                    let (uv_rect, screen_rect) = rect;
                    let gl_rect = rusttype::Rect {
                        min: origin
                            + (vector(
                                screen_rect.min.x as f32 / screen_width - 0.5,
                                1.0 - screen_rect.min.y as f32 / screen_height - 0.5,
                            )) * 2.0,
                        max: origin
                            + (vector(
                                screen_rect.max.x as f32 / screen_width - 0.5,
                                1.0 - screen_rect.max.y as f32 / screen_height - 0.5,
                            )) * 2.0,
                    };
                    color[0] = style.foreground.r as f32 / 255.0;
                    color[1] = style.foreground.g as f32 / 255.0;
                    color[2] = style.foreground.b as f32 / 255.0;
                    color[3] = style.foreground.a as f32 / 255.0;
                    vec![
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.max.y],
                            tex_coords: [uv_rect.min.x, uv_rect.max.y],
                            color,
                        },
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.min.y],
                            tex_coords: [uv_rect.min.x, uv_rect.min.y],
                            color,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.min.y],
                            tex_coords: [uv_rect.max.x, uv_rect.min.y],
                            color,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.min.y],
                            tex_coords: [uv_rect.max.x, uv_rect.min.y],
                            color,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.max.y],
                            tex_coords: [uv_rect.max.x, uv_rect.max.y],
                            color,
                        },
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.max.y],
                            tex_coords: [uv_rect.min.x, uv_rect.max.y],
                            color,
                        },
                    ]
                })
                .collect();

            glium::VertexBuffer::new(disp, &vertices).unwrap()
        };

        target
            .draw(
                &vertex_buffer,
                glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.text_program,
                &uniforms,
                &glium::DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    /// draw cursor,...
    pub fn draw_rect(
        &mut self,
        rect: &Rect,
        color: &ColorRGBA,
        disp: &Display,
        target: &mut Frame,
    ) {
        //get size of window
        let (width, height): (f32, f32) = disp.gl_window().window().inner_size().into();
        let (x, y, width, height) = rect.to_noramalized(width.ceil() as u32, height.ceil() as u32);

        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
                color: [f32; 4],
            }

            implement_vertex!(Vertex, position, color);
            let verts: Vec<Vertex> = vec![
                Vertex {
                    position: [x, y],
                    color: color.as_arr(),
                },
                Vertex {
                    position: [x, height],
                    color: color.as_arr(),
                },
                Vertex {
                    position: [width, y],
                    color: color.as_arr(),
                },
                Vertex {
                    position: [x, height],
                    color: color.as_arr(),
                },
                Vertex {
                    position: [width, y],
                    color: color.as_arr(),
                },
                Vertex {
                    position: [width, height],
                    color: color.as_arr(),
                },
            ];

            glium::VertexBuffer::new(disp, &verts).unwrap()
        };

        let uniforms = uniform! {
            tex: self.cache_tex.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };
        target
            .draw(
                &vertex_buffer,
                glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &self.decor_program,
                &uniforms,
                &glium::DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
    }

    fn draw_cursor(
        &mut self,
        buff: &mut TextBuffer,
        disp: &Display,
        target: &mut Frame,
    ) {
        let (width, height): (f32, f32) = disp.gl_window().window().inner_size().into();

        buff.cursor
            .calc_screen_pos(&buff.font, &buff.buffer, width as i32, height as i32);
        let x = (buff.cursor.screen_pos.0 + 1.0) * (width / 2.0);
        let y = (1.0 - buff.cursor.screen_pos.1) * (height / 2.0);
        let width = x + buff.cursor.width;
        let height = y + buff.cursor.height;

        let rect = Rect {
            x: ScreenSize::Px(x as i32),
            y: ScreenSize::Px(y as i32),
            width: ScreenSize::Px(width as i32),
            height: ScreenSize::Px(height as i32),
        };

        let col = ColorRGBA::new();
        self.draw_rect(&rect, &col, disp, target);
    }
}
