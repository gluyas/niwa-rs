extern crate gl;
extern crate glutin;

fn main() {
    use glutin::GlContext;

    // set up gl context/window
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("NIWA")
        .with_dimensions(1280, 720);
    let context = glutin::ContextBuilder::new();
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().unwrap() };

    // load fn ptrs into gl from context
    gl::load_with(|addr| gl_window.get_proc_address(addr) as *const _);

    let mut exit = false;
    while !exit {
        use glutin::{Event, WindowEvent};

        events_loop.poll_events(|event| match event {
            Event::WindowEvent{window_id: _, event} => match event {
                WindowEvent::CloseRequested => exit = true,
                _ => (),
            },
            _ => (),
        });
    }
}
