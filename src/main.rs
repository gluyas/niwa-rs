extern crate niwa_rs;
use niwa_rs::tilemap::*;

#[macro_use]
extern crate lazy_static;

extern crate gl;
use gl::types::*;

extern crate glutin;
use glutin::GlContext;

extern crate cgmath;
use cgmath::*;

extern crate image;

use std::default::Default;

use std::fs::File;
use std::io::{Read, BufReader};

use std::mem::size_of;
use std::ptr;
use std::ffi::CString;

lazy_static! {
    static ref MAP: TileMap<u8> = TileMap {
        width: 5, height: 4,
        data: Box::new([
            0, 1, 1, 1, 1,
            1, 1, 1, 0, 1,
            1, 0, 1, 1, 1,
            1, 1, 1, 0, 0,
        ]),
    };
}

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const CLIP_NEAR: f32 = 0.1;
const CLIP_FAR: f32 = 100.0;

fn main() {
    // set up gl context/window
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("NIWA")
        .with_dimensions(WIDTH, HEIGHT)
        .with_resizable(false);
    let context = glutin::ContextBuilder::new();
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe { gl_window.make_current().unwrap() };

    // load fn ptrs into gl from context
    gl::load_with(|addr| gl_window.get_proc_address(addr) as *const _);

    #[repr(C)]
    struct Mvp {
        modelview: Matrix4<GLfloat>,
        projection: Matrix4<GLfloat>,
    };

    let (mut mvp, mvp_ubo, mvp_binding_index) = unsafe {
        let mut mvp = Mvp {
            modelview: Matrix4::identity(),
            projection: perspective(
                Deg(60.0), WIDTH as f32 / HEIGHT as f32, CLIP_NEAR, CLIP_FAR
            ),
        };

        let mvp_ubo = gen_object(gl::GenBuffers);
        gl::BindBuffer(gl::UNIFORM_BUFFER, mvp_ubo);
        gl::BufferData(
            gl::UNIFORM_BUFFER,
            size_of::<Mvp>() as GLsizeiptr,
            mvp.modelview.as_ptr() as *const GLvoid,
            gl::DYNAMIC_DRAW,
        );

        let mvp_binding_index: GLuint = 1;
        gl::BindBufferBase(gl::UNIFORM_BUFFER, mvp_binding_index, mvp_ubo);

        (mvp, mvp_ubo, mvp_binding_index)
    };

    let mut camera_azimuth: f32 = -45.0;
    let mut camera_elevation: f32 = 30.0;
    let mut camera_distance: f32 = 10.0;
    let centre_offset = -Vector3::new(MAP.width as f32 - 1.0, MAP.height as f32 - 1.0, 0.0) / 2.0;

    let update_camera = |camera_distance: f32, camera_elevation: f32, camera_azimuth: f32, mvp: &mut Mvp| unsafe {
        gl::BindBuffer(gl::UNIFORM_BUFFER, mvp_ubo);

        let camera_pos = (Quaternion::from_angle_z(Deg(camera_azimuth))
            * Quaternion::from_angle_x(Deg(-camera_elevation)))
            .rotate_point(Point3::new(0.0, -camera_distance, 0.0));

        mvp.modelview = Matrix4::look_at(
            camera_pos,
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ) * Matrix4::from_translation(centre_offset);

        gl::BufferSubData(
            gl::UNIFORM_BUFFER,
            0 as GLintptr,
            size_of::<Matrix4<GLfloat>>() as GLsizeiptr,
            mvp.modelview.as_ptr() as *const GLvoid,
        );
    };

    let (bg_shader, bg_vao) = unsafe {
        let vert = compile_shader(
            File::open("src/shader/sky.vert.glsl").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(
            File::open("src/shader/sky.frag.glsl").unwrap(), gl::FRAGMENT_SHADER);

        let program = link_shaders(&[vert, frag]);
        gl::UseProgram(program);

        // set uniforms
        {
            let name = CString::new("bg_color_top").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 1f32, 1f32, 1f32);
        }
        {
            let name = CString::new("bg_color_high").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 0f32, 1f32, 1f32);
        }
        {
            let name = CString::new("bg_color_low").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 1f32, 0f32, 1f32);
        }
        {
            let name = CString::new("bg_color_bot").unwrap();
            let uniform = gl::GetUniformLocation(program, name.as_ptr());
            gl::Uniform3f(uniform, 0.7f32, 0f32, 0.7f32);
        }
        // bind the uniform block to mvp uniform buffer
        {
            let name = CString::new("Mvp").unwrap();
            let mvp_index = gl::GetUniformBlockIndex(program, name.as_ptr());
            gl::UniformBlockBinding(program, mvp_index, mvp_binding_index);
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
                let name = CString::new("screen_position").unwrap();
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

    let (world_shader, sprite_uniform) = unsafe {
        let vert = compile_shader(
            File::open("src/shader/basic3d.vert.glsl").unwrap(), gl::VERTEX_SHADER);
        let frag = compile_shader(
            File::open("src/shader/sprite.frag.glsl").unwrap(), gl::FRAGMENT_SHADER);

        let program = link_shaders(&[vert, frag]);
        gl::UseProgram(program);

        // bind the uniform block to mvp uniform buffer
        {
            let name = CString::new("Mvp").unwrap();
            let mvp_index = gl::GetUniformBlockIndex(program, name.as_ptr());
            gl::UniformBlockBinding(program, mvp_index, mvp_binding_index);
        }

        let sprite_uniform = {
            let name = CString::new("sprite").unwrap();
            gl::GetUniformLocation(program, name.as_ptr())
        };

        (program, sprite_uniform)
    };

    let (tile_sprite, tile_mesh_len, stage_vao, stage_num_tiles) = unsafe {
        {
            gl::ActiveTexture(gl::TEXTURE0);
            let texture = gen_object(gl::GenTextures);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            load_sprite_to_bound_texture("./assets/textures/warts.png");

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        }

        let tile_offsets = make_map_offsets(&MAP);

        let tile_mesh: &[GLfloat] = &[
            -0.5, -0.5, 0.0, 0.0, 1.0,
            0.5, -0.5, 0.0, 1.0, 1.0,
            -0.5, 0.5, 0.0, 0.0, 0.5,
            0.5, 0.5, 0.0, 1.0, 0.5,
            -0.5, 0.5, -1.0, 0.0, 0.0,
            0.5, 0.5, -1.0, 1.0, 0.0,

            0.5, 0.5, 0.0, 0.0, 0.5,
            0.5, 0.5, -1.0, 0.0, 0.0,
            0.5, -0.5, 0.0, 1.0, 0.5,
            0.5, -0.5, -1.0, 1.0, 0.0,

            0.5, -0.5, 0.0, 0.0, 0.5,
            0.5, -0.5, -1.0, 0.0, 0.0,
            -0.5, -0.5, 0.0, 1.0, 0.5,
            -0.5, -0.5, -1.0, 1.0, 0.0,

            -0.5, -0.5, 0.0, 0.0, 0.5,
            -0.5, -0.5, -1.0, 0.0, 0.0,
            -0.5, 0.5, 0.0, 1.0, 0.5,
            -0.5, 0.5, -1.0, 1.0, 0.0,
        ];

        let vao = gen_object(gl::GenVertexArrays);
        gl::BindVertexArray(vao);
        {
            let quad_vbo = gen_object(gl::GenBuffers);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<GLfloat>() * tile_mesh.len()) as GLsizeiptr,
                tile_mesh.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );

            {
                let name = CString::new("position").unwrap();
                let position = gl::GetAttribLocation(world_shader, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(position);
                gl::VertexAttribPointer(
                    position, 3, gl::FLOAT, gl::FALSE,
                    5 * size_of::<GLfloat>() as GLsizei,
                    ptr::null() as *const GLvoid,
                );
            }

            {
                let name = CString::new("uv_in").unwrap();
                let uv = gl::GetAttribLocation(world_shader, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(uv);
                gl::VertexAttribPointer(
                    uv, 2, gl::FLOAT, gl::FALSE,
                    5 * size_of::<GLfloat>() as GLsizei,
                    ptr::null::<GLfloat>().offset(3) as *const GLvoid,
                );
            }

            let offset_vbo = gen_object(gl::GenBuffers);
            gl::BindBuffer(gl::ARRAY_BUFFER, offset_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (size_of::<Vector2<GLfloat>>() * tile_offsets.len()) as GLsizeiptr,
                tile_offsets.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW
            );

            {
                let name = CString::new("offset").unwrap();
                let offset = gl::GetAttribLocation(world_shader, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(offset);
                gl::VertexAttribPointer(
                    offset, 2, gl::FLOAT, gl::FALSE,
                    0 as GLsizei,
                    ptr::null() as *const GLvoid,
                );
                gl::VertexAttribDivisor(offset, 1); // use for instanced rendering
            }
        }

        (0, tile_mesh.len() / 5, vao, tile_offsets.len())
    };
    let draw_stage = || unsafe {
        gl::UseProgram(world_shader);
        gl::Uniform1i(sprite_uniform, tile_sprite);

        gl::BindVertexArray(stage_vao);

        // render using blend for smooth edges
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, tile_mesh_len as GLsizei, stage_num_tiles as GLsizei);
        gl::Disable(gl::BLEND);
    };

    let mut player_pos: [GLfloat; 2] = [0.0, 0.0];

    let (player_sprite, player_vao, player_pos_vbo) = unsafe {
        {   // move player sprite into texture unit 1
            gl::ActiveTexture(gl::TEXTURE1);
            let texture = gen_object(gl::GenTextures);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            load_sprite_to_bound_texture("./assets/sprites/avatar.png");

            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        }

        let vao = gen_object(gl::GenVertexArrays);
        gl::BindVertexArray(vao);
        {
            let sprite_quad: [GLfloat; 20] = [
                0.5, 0.0, 0.0, 1.0, 0.0,
                0.5, 0.0, 2.0, 1.0, 1.0,
                -0.5, 0.0, 2.0, 0.0, 1.0,
                -0.5, 0.0, 0.0, 0.0, 0.0,
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
                let position = gl::GetAttribLocation(world_shader, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(position);
                gl::VertexAttribPointer(
                    position, 3, gl::FLOAT, gl::FALSE,
                    5 * size_of::<GLfloat>() as GLsizei,
                    ptr::null() as *const GLvoid,
                );
            }

            {
                let name = CString::new("uv_in").unwrap();
                let uv = gl::GetAttribLocation(world_shader, name.as_ptr()) as GLuint;
                gl::EnableVertexAttribArray(uv);
                gl::VertexAttribPointer(
                    uv, 2, gl::FLOAT, gl::FALSE,
                    5 * size_of::<GLfloat>() as GLsizei,
                    ptr::null::<GLfloat>().offset(3) as *const GLvoid,
                );
            }
        }

        let pos_vbo = gen_object(gl::GenBuffers);
        gl::BindBuffer(gl::ARRAY_BUFFER, pos_vbo);
        {
            let name = CString::new("offset").unwrap();
            let offset = gl::GetAttribLocation(world_shader, name.as_ptr()) as GLuint;
            gl::EnableVertexAttribArray(offset);
            gl::VertexAttribPointer(
                offset, 2, gl::FLOAT, gl::FALSE,
                0 as GLsizei,
                ptr::null() as *const GLvoid,
            );
            gl::VertexAttribDivisor(offset, 1); // use for instanced rendering
        }

        (1, vao, pos_vbo)
    };
    let draw_player = |player_pos: &[GLfloat; 2]| unsafe {
        gl::UseProgram(world_shader);
        gl::Uniform1i(sprite_uniform, player_sprite);

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

    let render = |
        camera_distance: f32, camera_elevation: f32, camera_azimuth: f32, mvp: &mut Mvp,
        player_pos: &[GLfloat; 2]
    | unsafe {
        update_camera(camera_distance, camera_elevation, camera_azimuth, mvp);

        gl::Clear(gl::DEPTH_BUFFER_BIT);   // background overwrites color buffer
        gl::Disable(gl::DEPTH_TEST);
        draw_background();

        gl::Enable(gl::DEPTH_TEST); // enable depth test for rendering world
        draw_stage();
        draw_player(&player_pos);

        gl_window.swap_buffers().expect("buffer swap failed");
    };
    render(camera_distance, camera_elevation, camera_azimuth, &mut mvp, &player_pos);

    let mut mouse_down = false;
    let mut mouse_was_dragged = false;
    let mut mouse_last_pos = (0.0, 0.0);

    let mut exit = false;
    while !exit {
        use glutin::*;

        events_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => exit = true,
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_delta_x, delta_y), ..
                } => {
                    camera_distance += -delta_y * 0.5;
                    if camera_distance < 0.0 { camera_distance = 0.0 };
                    render(camera_distance, camera_elevation, camera_azimuth, &mut mvp, &player_pos);
                },
                WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => match state {
                    ElementState::Pressed => { mouse_down = true; },
                    ElementState::Released => {
                        if !mouse_was_dragged {
                            let cursor_ndc = Vector2 {
                                x: mouse_last_pos.0 / WIDTH as f32 - 0.5,
                                y: -mouse_last_pos.1 / HEIGHT as f32 + 0.5,
                            } * 2.0;
                            
                            let mvp_inverse = (mvp.projection * mvp.modelview)
                                .inverse_transform()
                                .expect("mvp inversion failed");
    
                            let ray_origin = mvp_inverse.transform_point(
                                Point3::new(cursor_ndc.x, cursor_ndc.y, 0.0)
                            );
                            let ray_direction = (mvp_inverse.transform_point(
                                Point3::new(cursor_ndc.x, cursor_ndc.y, 1.0)
                            ) - ray_origin).normalize();

                            if ray_direction.z.abs() >= 0.01          // avoid divide by zero
                            && ray_origin.z * ray_direction.z < 0.0 { // discard rays going away from zero
                                let intersect_0 = ray_origin + -ray_origin.z / ray_direction.z * ray_direction;
                                player_pos[0] = intersect_0.x;
                                player_pos[1] = intersect_0.y;
                                
                                render(camera_distance, camera_elevation, camera_azimuth, &mut mvp, &player_pos);
                            }
                        }
                        mouse_was_dragged = false;
                        mouse_down = false; 
                    },
                },
                WindowEvent::CursorMoved { position, .. } => {
                    let position = (position.0 as f32, position.1 as f32);
                    if mouse_down {
                        mouse_was_dragged = true;
                        
                        camera_azimuth += 0.3 * (mouse_last_pos.0 - position.0);
                        camera_elevation -= 0.3 * (mouse_last_pos.1 - position.1);
                        if      camera_elevation < -85.0 { camera_elevation = -85.0; }
                        else if camera_elevation >  85.0 { camera_elevation =  85.0; }
                        
                        render(camera_distance, camera_elevation, camera_azimuth, &mut mvp, &player_pos);
                    }
                    mouse_last_pos = position;
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Key(KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(keycode),
                .. }) => {
                    match keycode {
                        VirtualKeyCode::W => player_pos[1] += 1.0,
                        VirtualKeyCode::A => player_pos[0] -= 1.0,
                        VirtualKeyCode::S => player_pos[1] -= 1.0,
                        VirtualKeyCode::D => player_pos[0] += 1.0,
                        VirtualKeyCode::Q => camera_azimuth -= 5.0,
                        VirtualKeyCode::E => camera_azimuth += 5.0,
                        VirtualKeyCode::Escape => exit = true,
                        _ => (),
                    }
                    render(camera_distance, camera_elevation, camera_azimuth, &mut mvp, &player_pos);
                },
                _ => (),
            },
            _ => (),
        });
    }
}

fn make_map_offsets(map: &TileMap<u8>) -> Box<[Vector2<GLfloat>]> {
    let mut offsets = Vec::new();
    for y in 0..(map.height) {
        for x in 0..(map.width) {
            if map[(x, y)] != 0 {
                offsets.push(Vector2::new(x as f32, y as f32));
            }
        }
    }
    offsets.into_boxed_slice()
}

unsafe fn load_sprite_to_bound_texture(sprite_path: impl AsRef<std::path::Path>) {
    use image::*;
    let sprite: RgbaImage = open(sprite_path)
        .expect("failed to read image")
        .flipv()
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
