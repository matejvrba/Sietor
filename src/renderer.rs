use crate::buffer::TextBuffer;
use crate::window;
use glium::*;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rusttype::gpu_cache::Cache;
use rusttype::{point, vector, Font, PositionedGlyph, Rect, Scale};
use std::borrow::Cow;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// struct resposible for rendering text and decoration
pub struct Renderer<'a> {
    font: Font<'a>,
    cache: Cache<'a>,
    cache_tex: Texture2d,
    /// OpenGL texture used for caching of font
    text_program: Program,
    ///shader for drwaing text
    decor_program: Program,
    ///sjader for drawing solid rectangles
    ts: ThemeSet,
    ps: SyntaxSet,
}

impl<'a> Renderer<'_> {
    pub fn new(win: &window::WindowContext) -> Result<Self, failure::Error> {
        trace!("Initializing syntect");
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        //let _syntax = ps.find_syntax_by_extension("rs").unwrap();

        trace!("Loading font \"/usr/share/fonts/TTF/Hack-Regular.ttf\"");
        let font_path = std::env::current_dir()
            .unwrap()
            .join("/usr/share/fonts/TTF/Hack-Regular.ttf");
        let data = std::fs::read(&font_path).unwrap();
        let font = Font::try_from_vec(data).unwrap();

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
in vec4 colour;

out VS_OUTPUT{
    vec2 tex_coords;
    vec4 colour;
}OUT;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    OUT.tex_coords = tex_coords;
    OUT.colour = colour;
}
            ",
            fragment: "
#version 150
uniform sampler2D tex;
                
in VS_OUTPUT{
    vec2 tex_coords;
    vec4 colour;
} IN;

out vec4 colour;

void main() {
    colour = vec4(IN.colour.rgb, texture(tex, IN.tex_coords).r);
}
                        "
        })?;

        let decor_program = program!(
        &win.display,
        140 => {
            vertex: "
#version 140

in vec2 position;
in vec4 colour;

out vec4 v_colour;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_colour = colour;
}
            ",
            fragment: "
#version 140
in vec4 v_colour;
out vec4 f_colour;

void main() {
    f_colour = v_colour;
}
                        "
        })?;

        Ok(Self {
            ps,
            font,
            cache,
            cache_tex,
            text_program,
            decor_program,
            ts,
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
        let mut highlight =
            syntect::easy::HighlightLines::new(syntax, &self.ts.themes["InspiredGitHub"]);

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
        height_factor: f32,
        scale: f32,
        disp: &Display,
        buff: &TextBuffer,
    ) {
        self.draw_text(x, y, width_factor, height_factor, scale, disp, buff);
    }

    ///draws text, `x` and `y` is normalized position of top left corner
    /// `width_factor` and `height_factor` is normalized width and height
    /// `scale is size of font`
    fn draw_text(
        &mut self,
        x: f32,
        y: f32,
        width_factor: f32,
        height_factor: f32,
        scale: f32,
        disp: &Display,
        buff: &TextBuffer,
    ) {
        //get size of window
        let (mut width, mut height): (f32, f32) = disp.gl_window().window().inner_size().into();
        height = height_factor * height;
        let height = height.ceil() as u32;
        width = width_factor * width;
        let width = width.ceil() as u32;

        let scale_dis = disp.gl_window().window().scale_factor() as f32;

        let glyphs = self.layout_paragraph(
            &self.font,
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
                colour: [f32; 4],
            }

            implement_vertex!(Vertex, position, tex_coords, colour);
            let mut colour = [0.0, 0.0, 0.0, 1.0];
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
                    let gl_rect = Rect {
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
                    colour[0] = style.foreground.r as f32 / 255.0;
                    colour[1] = style.foreground.g as f32 / 255.0;
                    colour[2] = style.foreground.b as f32 / 255.0;
                    colour[3] = style.foreground.a as f32 / 255.0;
                    vec![
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.max.y],
                            tex_coords: [uv_rect.min.x, uv_rect.max.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.min.y],
                            tex_coords: [uv_rect.min.x, uv_rect.min.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.min.y],
                            tex_coords: [uv_rect.max.x, uv_rect.min.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.min.y],
                            tex_coords: [uv_rect.max.x, uv_rect.min.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.max.x, gl_rect.max.y],
                            tex_coords: [uv_rect.max.x, uv_rect.max.y],
                            colour,
                        },
                        Vertex {
                            position: [gl_rect.min.x, gl_rect.max.y],
                            tex_coords: [uv_rect.min.x, uv_rect.max.y],
                            colour,
                        },
                    ]
                })
                .collect();

            glium::VertexBuffer::new(disp, &vertices).unwrap()
        };

        let mut target = disp.draw();
        target.clear_color(1.0, 1.0, 1.0, 0.0);
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

        target.finish().unwrap();
    }
    /// draw cursor,...
    pub fn draw_decoration() {}
}
