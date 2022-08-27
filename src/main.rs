extern crate render_derive;
#[macro_use]
extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate vec_2_10_10_10;
use std::env;

use failure::err_msg;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use glium::*;
use glutin::{
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};


use crate::buffer::TextBuffer;
use crate::debug::failure_to_string;

mod buffer;
mod window;
mod debug;
mod rend;

fn main() {
    env_logger::init();
    if let Err(e) = sietor() {
        error!("{}", failure_to_string(e));
    }
}

fn sietor() -> Result<(), failure::Error> {
    if cfg!(target_os = "linux") && env::var("WINIT_UNIX_BACKEND").is_err() {
        env::set_var("WINIT_UNIX_BACKEND", "x11");
    }

    trace!("Opening a window");
    let win = window::WindowContext::new("Sietor").map_err(err_msg)?;
    let disp = win.display.clone();
    let mut rend = rend::Rend::new(&win)?;


    let text: String = "
mod window;
mod rend;

fn main() {
    env_logger::init();
    if let Err(e) = sietor() {
        error!(\"{}\", failure_to_string(e));
    }
}
"
        .into();

    let buf = TextBuffer::new(
        buffer::BufferOrigin::Buffer(text),
        None,
        None,
    );
    win.event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
            *control_flow = ControlFlow::Exit
            },

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.state == ElementState::Pressed {
                    let mut redraw = false;
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
                                redraw = true;
                            }
                        }
                        Some(VirtualKeyCode::Q) | Some(VirtualKeyCode::Escape) => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    }
                    if redraw {
                        win.display.gl_window().window().request_redraw();
                    }
                }
            }

            /*
                Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                viewport.update_size(size.width as i32, size.height as i32);
                viewport.set_used(&win.gl);
                cam.set_size(size.width, size.height);
            }
                 */
            Event::MainEventsCleared => {
                // Application update code.
            }
            Event::RedrawRequested(_) => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in MainEventsCleared, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.
                //
                // Could be usefull for gui
//                buf.draw(0.0, 0.0, 0.0, 0.0);
                rend.draw(0.0,0.0,0.5,1.0, 24.0, &disp, &buf);
            }
            _ => (),
        }
    });
}
