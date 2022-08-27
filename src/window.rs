use glium::*;
use glutin::event_loop:: EventLoop;
extern crate gl;

pub struct WindowContext {
    pub event_loop: EventLoop<()>,
    pub display: Display,
    pub gl: gl::Gl,
}


impl WindowContext {
    pub fn new(title: &str) -> Result<Self, failure::Error> {

        let window = glium::glutin::window::WindowBuilder::new()
            .with_inner_size(glium::glutin::dpi::PhysicalSize::new(512u32, 512u32))
            .with_title(title);
        let context = glium::glutin::ContextBuilder::new().with_vsync(true);
        let event_loop = glium::glutin::event_loop::EventLoop::new();

        let display = glium::Display::new(window, context, &event_loop)?;


        let gl = gl::Gl::load_with(|ptr| display.gl_window().get_proc_address(ptr) as *const _);

        let win = WindowContext {
            display,
            event_loop,
            gl: gl.clone(),
        };
        Ok(win)
    }
/*    pub fn run(self) -> Result<(), String> {
        self.event_loop.run(move |event, _, control_flow| {
            //println!("{:?}", event);
            *control_flow = ControlFlow::Wait;

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        self.window_context.resize(physical_size);
                        unsafe{self.gl.Viewport(0,0,physical_size.width as i32, physical_size.height as i32);}
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                },
                Event::RedrawRequested(_) => {
                    self.renderer.draw();
                    self.window_context.swap_buffers().unwrap();
                }
                _ => (),
            }
        });
        // Ok(())
    }*/
}
