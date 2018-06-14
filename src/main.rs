extern crate gl;
extern crate glutin;

use std::fs::File;
use std::ptr;
use std::ffi::CString;

use glutin::GlContext;

use gl::types::*;

static VERTEX_DATA: [GLfloat; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];

fn main() {
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

    unsafe {
        // create shaders
        let vert = compile_shader(File::open("src/basic2d.vert").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(File::open("src/background.frag").unwrap(), gl::FRAGMENT_SHADER);

        // link shaders
        let program = gl::CreateProgram();
        gl::AttachShader(program, vert);
        gl::AttachShader(program, frag);
        gl::LinkProgram(program);

        {   // check link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            if status != (gl::TRUE as GLint) { panic!("Program linking failed"); }
        }

        gl::UseProgram(program);

        // set uniforms
        {
            let name = CString::new("bg_color_top").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 1f32, 0f32, 0f32);
        }

        {
            let name = CString::new("bg_color_bot").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 0f32, 0f32, 1f32);
        }

        {
            let name = CString::new("bg_resolution").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform2ui(uniform, 1280, 720);
        }

        {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (VERTEX_DATA.len() * std::mem::size_of::<GLfloat>()) as isize,
                std::mem::transmute(&VERTEX_DATA[0]),
                gl::STATIC_DRAW,
            );

            {
                let name = CString::new("position").unwrap();
                let position = gl::GetAttribLocation(program, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(position);
                gl::VertexAttribPointer(position, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
            }
        }

        {   // final check
            gl::ValidateProgram(program);
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::VALIDATE_STATUS, &mut status);
            if status != (gl::TRUE as GLint) { panic!("Program validation failed"); }
        }

        gl::DrawArrays(gl::TRIANGLES, 0, 3);
        gl_window.swap_buffers();

        program
    };

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

    fn compile_shader(file: File, ty: GLenum) -> GLuint {
        unsafe {
            let shader = gl::CreateShader(ty);
            {   // upload shader src
                use std::fs::File;
                use std::io::Read;
                use std::io::BufReader;

                let mut src = String::new();
                BufReader::new(file).read_to_string(&mut src).expect("file read failed");

                let src = CString::new(src.as_bytes()).expect("CString failed");

                gl::ShaderSource(shader, 1, &src.as_ptr(), ptr::null());
            };

            gl::CompileShader(shader);

            // check shader compile errors
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                use std::str::from_utf8;

                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!("{}", from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8"));
            }
            shader
        }
    }
}
