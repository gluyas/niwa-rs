#[macro_use]
extern crate lazy_static;

extern crate gl;
extern crate glutin;

extern crate image;

use std::default::Default;

use std::fs::File;
use std::io::{Read, BufReader};

use std::mem::size_of;
use std::ptr;
use std::ffi::CString;

use std::ops::Index;

use glutin::GlContext;
use gl::types::*;

lazy_static! {
    static ref MAP: TileMap<u8> = TileMap {
        width: 3, height: 3,
        data: Box::new([
            0,  1,  0,
            0,  1,  1,
            1,  1,  0,
        ]),
    };
}

struct TileMap<T> {
    pub data: Box<[T]>,
    pub width: u8,
    pub height: u8,
}

impl<T> Index<(u8, u8)> for TileMap<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: (u8, u8)) -> &Self::Output {
        let index = index.0 * self.width + index.1;
        &self.data[index as usize]
    }
}

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

    let (bg_shader, bg_vao) = unsafe {
        let vert = compile_shader(
            File::open("src/shader/basic2d.vert").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(
            File::open("src/shader/background.frag").unwrap(), gl::FRAGMENT_SHADER);

        let program = link_shaders(&[vert, frag]);
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

        let vao = gen_object(gl::GenVertexArrays);
        gl::BindVertexArray(vao);
        {
            let bg_quad: [GLfloat; 8] = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, 1.0];

            let vbo = gen_object(gl::GenBuffers);
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

        (program, vao)
    };
    let draw_background = || unsafe {
        gl::UseProgram(bg_shader);
        gl::BindVertexArray(bg_vao);

        gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
    };

    let (world_shader, world_vao, world_num_tiles) = unsafe {
        let vert = compile_shader(
            File::open("src/shader/basic2d.vert").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(
            File::open("src/shader/sprite.frag").unwrap(), gl::FRAGMENT_SHADER);

        let program = link_shaders(&[vert, frag]);
        gl::UseProgram(program);
        {
            gl::ActiveTexture(gl::TEXTURE0);
            let texture = gen_object(gl::GenTextures);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            load_sprite_to_bound_texture("./assets/sprites/tile.png");

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            // set uniform
            let name = CString::new("sprite").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform1i(uniform, 0);
        }

        let quad_offsets = make_map_offsets(&MAP, 0.5) as Box<[GLfloat]>;

        let vao = gen_object(gl::GenVertexArrays);
        gl::BindVertexArray(vao);
        {
            let sprite_quad: [GLfloat; 16] = [
                //   x      y       u     v
                0.25, 0.25, 1.0, 0.0,
                0.25, -0.25, 1.0, 1.0,
                -0.25, -0.25, 0.0, 1.0,
                -0.25, 0.25, 0.0, 0.0,
            ];

            let quad_vbo = gen_object(gl::GenBuffers);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
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
                    ptr::null() as *const GLvoid,
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

            let offset_vbo = gen_object(gl::GenBuffers);
            gl::BindBuffer(gl::ARRAY_BUFFER, offset_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<GLfloat>() * quad_offsets.len()) as GLsizeiptr,
                quad_offsets.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW
            );

            {
                let name = CString::new("offset").unwrap();
                let offset = gl::GetAttribLocation(program, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(offset);
                gl::VertexAttribPointer(
                    offset, 2, gl::FLOAT, gl::FALSE,
                    0 as GLsizei,
                    ptr::null() as *const GLvoid,
                );
                gl::VertexAttribDivisor(offset, 1); // use for instanced rendering
            }
        }

        (program, vao, quad_offsets.len() / 2)
    };
    let draw_world = || unsafe {
        gl::UseProgram(world_shader);
        gl::BindVertexArray(world_vao);

        // render using blend for smooth edges
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::DrawArraysInstanced(gl::TRIANGLE_FAN, 0, 4, world_num_tiles as GLsizei);
        gl::Disable(gl::BLEND);
    };

    let mut player_pos: [GLfloat; 2] = [0.0, 0.0];

    let (player_shader, player_vao, player_pos_vbo) = unsafe {
        let vert = compile_shader(
            File::open("src/shader/basic2d.vert").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(
            File::open("src/shader/sprite.frag").unwrap(), gl::FRAGMENT_SHADER);

        let program = link_shaders(&[vert, frag]);
        gl::UseProgram(program);
        {
            gl::ActiveTexture(gl::TEXTURE1);
            let texture = gen_object(gl::GenTextures);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            load_sprite_to_bound_texture("./assets/sprites/avatar.png");

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

            // set uniform
            let name = CString::new("sprite").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform1i(uniform, 1);
        }

        let vao = gen_object(gl::GenVertexArrays);
        gl::BindVertexArray(vao);
        {
            let sprite_quad: [GLfloat; 16] = [
                //   x      y       u     v
                0.25, 0.75, 1.0, 0.0,
                0.25, -0.75, 1.0, 1.0,
                -0.25, -0.75, 0.0, 1.0,
                -0.25, 0.75, 0.0, 0.0,
            ];

            let quad_vbo = gen_object(gl::GenBuffers);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
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
                    ptr::null() as *const GLvoid,
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

        let pos_vbo = gen_object(gl::GenBuffers);
        gl::BindBuffer(gl::ARRAY_BUFFER, pos_vbo);
        {
            let name = CString::new("offset").unwrap();
            let offset = gl::GetAttribLocation(program, name.as_ptr()) as GLuint;
            gl::EnableVertexAttribArray(offset);
            gl::VertexAttribPointer(
                offset, 2, gl::FLOAT, gl::FALSE,
                0 as GLsizei,
                ptr::null() as *const GLvoid,
            );
            gl::VertexAttribDivisor(offset, 1); // use for instanced rendering
        }

        (program, vao, pos_vbo)
    };
    let draw_player = |player_pos: &[GLfloat; 2]| unsafe {
        gl::UseProgram(player_shader);
        gl::BindVertexArray(player_vao);

        // update player coordinates
        gl::BindBuffer(gl::ARRAY_BUFFER, player_pos_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (size_of::<GLfloat>() * player_pos.len()) as GLsizeiptr,
            player_pos.as_ptr() as *const GLvoid,
            gl::DYNAMIC_DRAW,
        );

        // render using blend for smooth edges
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::DrawArraysInstanced(gl::TRIANGLE_FAN, 0, 4, 1);
        gl::Disable(gl::BLEND);
    };

    let render = |player_pos: &[GLfloat; 2]| {
        draw_background();
        draw_world();
        draw_player(&player_pos);
        gl_window.swap_buffers().expect("buffer swap failed");
    };
    render(&player_pos);

    let mut exit = false;
    while !exit {
        use glutin::*;

        events_loop.poll_events(|event| match event {
            Event::WindowEvent{ event, .. } => match event {
                WindowEvent::CloseRequested => exit = true,
                _ => (),
            },
            Event::DeviceEvent{ event, .. } => match event {
                DeviceEvent::Key(KeyboardInput{
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                    .. }) => {
                    match keycode {
                        VirtualKeyCode::W => player_pos[1] += 0.5,
                        VirtualKeyCode::A => player_pos[0] -= 0.5,
                        VirtualKeyCode::S => player_pos[1] -= 0.5,
                        VirtualKeyCode::D => player_pos[0] += 0.5,
                        VirtualKeyCode::Escape => exit = true,
                        _ => (),
                    }
                    render(&player_pos);
                },
                _ => (),
            },
            _ => (),
        });
    }
}

fn make_map_offsets(map: &TileMap<u8>, tile_space: f32) -> Box<[f32]> {
    let mut offsets = Vec::new();
    for y in 0..(map.height) {
        for x in 0..(map.width) {
            if map[(x, y)] != 0 {
                offsets.push(tile_space * (x as f32 - (map.width - 1) as f32 / 2.0));
                offsets.push(tile_space * -(y as f32 - (map.height - 1) as f32 / 2.0));
            }
        }
    }
    offsets.into_boxed_slice()
}

unsafe fn load_sprite_to_bound_texture(sprite_path: impl AsRef<std::path::Path>) {
    use image::*;
    let sprite: RgbaImage = open(sprite_path)
        .expect("failed to read image")
        .to_rgba();

    let (width, height) = sprite.dimensions();
    let sprite: Vec<GLubyte> = sprite.into_raw();

    gl::TexImage2D(
        gl::TEXTURE_2D, 0, gl::RGBA as GLint,
        width as GLsizei, height as GLsizei,
        0, gl::RGBA, gl::UNSIGNED_BYTE,
        sprite.as_ptr() as *const GLvoid,
    );
}

#[inline]
unsafe fn gen_object(gl_gen_callback: unsafe fn (GLsizei, *mut GLuint)) -> GLuint {
    let mut name = GLuint::default();
    gl_gen_callback(1, &mut name);
    name
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
