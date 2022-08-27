#[macro_use]
extern crate render_derive;
#[macro_use]
extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate vec_2_10_10_10;
extern crate crossfont;
use glutin::event::{ElementState, VirtualKeyCode};
use nalgebra as na;

use debug::failure_to_string;
use failure::err_msg;
use glutin::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use renderer::Camera;
use resources::Resources;
use std::path::Path;

use crate::crossfont::Rasterize;

mod debug;
mod macros;
mod renderer;
mod resources;
mod triangle;
mod window;

fn main() {
    env_logger::init();
    if let Err(e) = sietor() {
        error!("{}", failure_to_string(e));
    }
}

fn sietor() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();
    let win = window::WindowContext::new().map_err(err_msg)?;
    let mut cam = Camera::new();
    cam.set_size(1920, 1080);
    cam.pos(0, 0);

    let shader_program = renderer::GlProgram::from_res(&win.gl, &res, "shaders/triangle")?;
    let mut viewport = renderer::Viewport::for_window(900, 700);
    shader_program.set_used();
    let triangle = triangle::Triangle::new(&res, &win.gl)?;

    let color_buffer = renderer::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.3));
    color_buffer.set_used(&win.gl);

    let mut rast : crossfont::Rasterizer = crossfont::Rasterize::new(1.0, false)?;
    let desc = crossfont::FontDesc::new("./TerminusTTF.ttf", crossfont::Style::Description{slant: crossfont::Slant::Normal,weight: crossfont::Weight::Normal});
    let key  = rast.load_font(&desc, crossfont::Size::new(10.0))?;

    let gly = rast.get_glyph(crossfont::GlyphKey{character: 'c', font_key: key, size: crossfont::Size::new(10.0)})?;
    println!("{} {}", gly.width, gly.height);

    //viewport.set_used(&win.gl);
    win.event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        None => {}
                        Some(VirtualKeyCode::F11) => {
                            static mut WIREFRAME: bool = false;
                            unsafe {
                                if WIREFRAME {
                                    win.gl.PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                                    WIREFRAME = false;
                                } else {
                                    win.gl.PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                                    WIREFRAME = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                viewport.update_size(size.width as i32, size.height as i32);
                viewport.set_used(&win.gl);
                cam.set_size(size.width, size.height);
            }
            Event::MainEventsCleared => {
                // Application update code.
                shader_program.set_mat4("view", *cam.view_mat());
                color_buffer.clear(&win.gl);
                triangle.draw(&win.gl);
                win.window_context.swap_buffers().unwrap();
            }
            Event::RedrawRequested(_) => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.
                //
                // Could be usefull for gui
            }
            _ => (),
        }
    });
    //    Ok(())
}
