extern crate gl;
extern crate glutin;

extern crate image;

use std::default::Default;

use std::fs::File;
use std::io::{Read, BufReader};

use std::mem::size_of;
use std::ptr;
use std::ffi::CString;

use glutin::GlContext;
use gl::types::*;

fn main() {
    // set up gl context/window
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("NIWA")
        .with_dimensions(1280, 720)
        .with_resizable(false);
    let context = glutin::ContextBuilder::new();
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().unwrap() };

    // load fn ptrs into gl from context
    gl::load_with(|addr| gl_window.get_proc_address(addr) as *const _);

    unsafe {    // draw background
        let vert = compile_shader(
            File::open("src/shader/basic2d.vert").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(
            File::open("src/shader/background.frag").unwrap(), gl::FRAGMENT_SHADER);

        let program = link_shaders(&[vert, frag, 0]);
        gl::UseProgram(program);

        // set uniforms
        {
            let name = CString::new("bg_color_top").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 0f32, 1f32, 1f32);
        }

        {
            let name = CString::new("bg_color_bot").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 1f32, 0f32, 1f32);
        }

        {
            let name = CString::new("bg_resolution").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform2ui(uniform, 1280, 720);
        }

        {
            let bg_quad: [GLfloat; 8] = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0];

            let mut vao: GLuint = Default::default();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo: GLuint = Default::default();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (std::mem::size_of::<GLfloat>() * bg_quad.len()) as GLsizeiptr,
                bg_quad.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            {
                let name = CString::new("position").unwrap();
                let position = gl::GetAttribLocation(program, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(position);
                gl::VertexAttribPointer(position, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
            }
        }
        gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
    }

    unsafe {    // draw textured quad
        let vert = compile_shader(
            File::open("src/shader/basic2d.vert").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(
            File::open("src/shader/sprite.frag").unwrap(), gl::FRAGMENT_SHADER);

        let program = link_shaders(&[vert, frag, 0]);
        gl::UseProgram(program);

        {
            // load image
            use image::*;
            let sprite: RgbaImage = open("assets/sprites/debug.png")
                .expect("failed to read image").to_rgba();

            let (width, height) = sprite.dimensions();
            let sprite: Vec<u8> = sprite.into_raw();

            // bind and upload texture
            let mut texture = GLuint::default();
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA as GLint,
                width as GLsizei, height as GLsizei,
                0, gl::RGBA, gl::UNSIGNED_BYTE,
                sprite.as_ptr() as *const GLvoid,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // set uniform
            let name = CString::new("sprite").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform1i(uniform, 0);
        }

        {
            let sprite_quad: [GLfloat; 16] = [
            //   x     y       u     v
                 0.5,  0.5,    1.0,  0.0,
                 0.5, -0.5,    1.0,  1.0,
                -0.5, -0.5,    0.0,  1.0,
                -0.5,  0.5,    0.0,  0.0,
            ];

            let mut vao = GLuint::default();
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = GLuint::default();
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<GLfloat>() * sprite_quad.len()) as GLsizeiptr,
                sprite_quad.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            {
                let name = CString::new("position").unwrap();
                let position = gl::GetAttribLocation(program, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(position);
                gl::VertexAttribPointer(
                    position, 2, gl::FLOAT, gl::FALSE,
                    4 * size_of::<GLfloat>() as GLsizei,
                    ptr::null(),
                );
            }

            {
                let name = CString::new("uv_in").unwrap();
                let uv = gl::GetAttribLocation(program, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(uv);
                gl::VertexAttribPointer(
                    uv, 2, gl::FLOAT, gl::FALSE,
                    4 * size_of::<GLfloat>() as GLsizei,
                    ptr::null::<GLfloat>().offset(2) as *const GLvoid,
                );
            }
        }
        // render using blend for smooth edges
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
        gl::Disable(gl::BLEND);
    }

    gl_window.swap_buffers().expect("buffer swap failed");

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

fn compile_shader(file: File, ty: GLenum) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(ty);
        {   // upload shader src
            let mut src = String::new();
            BufReader::new(file).read_to_string(&mut src).expect("file read failed");

            let src = CString::new(src.as_bytes()).expect("CString failed");

            gl::ShaderSource(shader, 1, &src.as_ptr(), ptr::null());
        };

        gl::CompileShader(shader);

        // check shader compile errors
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

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
            panic!(
                "GLSL compile error:\n{}",
                from_utf8(&buf).ok().expect("ShaderInfoLog not valid utf8")
            );
        }
        shader
    }
}

fn link_shaders(shaders: &[GLuint]) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        for &shader in shaders { gl::AttachShader(program, shader); }
        gl::LinkProgram(program);

        {   // check link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
            if status != (gl::TRUE as GLint) {
                use std::str::from_utf8;

                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                panic!(
                    "GLSL link error:\n{}",
                    from_utf8(&buf).ok().expect("ProgramInfoLog not valid utf8")
                );
            }
        }
        program
    }
}
