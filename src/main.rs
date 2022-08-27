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
mod debug;
mod rend;
mod window;

enum State {
    Normal,
    Insert,
    Visual,
    Command,
}

fn main() {
    env_logger::init();

    let app = App::new();

    if let Err(e) = app.sietor() {
        error!("{}", failure_to_string(e));
    }
}

struct App {
    buffers: Vec<TextBuffer>,
    active_buffer: usize,
    state: State,
}

impl App {
    fn sietor(mut self) -> Result<(), failure::Error> {
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

        let buf = TextBuffer::new(buffer::BufferOrigin::Buffer(text), None, None);
        win.event_loop.run(move |event, _, control_flow| {
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
                            Some(VirtualKeyCode::Left) => {
                                self.buffers[self.active_buffer].move_cursor_relative(0, -1);
                            }
                            Some(VirtualKeyCode::Right) => {
                                self.buffers[self.active_buffer].move_cursor_relative(0, 1);
                            }
                            Some(VirtualKeyCode::Up) => {
                                self.buffers[self.active_buffer].move_cursor_relative(-1, 0);
                            }
                            Some(VirtualKeyCode::Down) => {
                                self.buffers[self.active_buffer].move_cursor_relative(1, 0);
                            }
                            None => {}
                            _ => {}
                        }
                        win.display.gl_window().window().request_redraw();
                    }
                }

                Event::WindowEvent {
                    event: WindowEvent::ReceivedCharacter(c),
                    ..
                } => {
                    self.process_input(c);
                    match c {
                        '\u{8}' => {
                            //         text.pop();
                        }
                        _ if c != '\u{7f}' => {
                            //text.push(c)
                        }
                        _ => {}
                    }
                    win.display.gl_window().window().request_redraw();
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
                    rend.draw(
                        0.0,
                        0.0,
                        0.5,
                        1.0,
                        24.0,
                        &disp,
                        &self.buffers[self.active_buffer],
                    );
                }
                _ => (),
            }
        });
    }

    fn process_input(&mut self, ch: char) {
        match self.state {
            State::Normal => match ch {
                'i' => {
                    self.state = State::Insert;
                    trace!("Switched to Insert mode");
                }
                'v' => {
                    self.state = State::Visual;
                    trace!("Switched to Visual mode");
                }
                ':' => {
                    self.state = State::Command;
                    trace!("Switched to Command mode");
                }
                _ => {
                    trace!("Unprocessed state result. Input {:?} in Normal mode", ch);
                }
            },
            State::Insert => match ch {
                '\u{1b}' => {
                    self.state = State::Normal;
                    trace!("Switched to Normal mode");
                }
                _ => {
                    self.buffers[self.active_buffer].insert(ch, None);
                }
            },
            State::Visual => match ch {
                '\u{1b}' => {
                    self.state = State::Normal;
                    trace!("Switched to Normal mode");
                }
                _ => {
                    trace!("Unprocessed state result. Input {:?} in Visual mode", ch);
                }
            },
            State::Command => match ch {
                '\u{1b}' => {
                    self.state = State::Normal;
                    trace!("Switched to Normal mode");
                }
                _ => {
                    trace!("Unprocessed state result. Input {:?} in Command mode", ch);
                }
            },
        }
    }

    fn new() -> Self {
        let buff = TextBuffer::new(
            buffer::BufferOrigin::Buffer(
                "Welcome to Sietor
idk, welcome text"
                    .to_string(),
            ),
            None,
            None,
        );
        App {
            active_buffer: 0,
            buffers: vec![buff],
            state: State::Normal,
        }
    }
}
