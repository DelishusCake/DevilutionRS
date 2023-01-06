use std::ffi::{c_void, CString};

use glfw::{WindowHint, OpenGlProfileHint};

use anyhow::Context;

use crate::mpq::Archive;

pub const TITLE: &str = "Diablo";
pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 480;

pub const VERTEX_SHADER: &str = r#"
#version 330 core
#extension GL_ARB_separate_shader_objects : enable

out VertexData 
{
    vec2 uv;
} vs_out;

void main() {
    float x = -1.0 + float((gl_VertexID & 1) << 2);
    float y = -1.0 + float((gl_VertexID & 2) << 1);

    vs_out.uv.x = (x + 1.0) * 0.5;
    vs_out.uv.y = (y + 1.0) * 0.5;

    gl_Position = vec4(x, y, 0.0, 1.0);
}
"#;

pub const FRAGMENT_SHADER: &str = r#"
#version 330 core
#extension GL_ARB_separate_shader_objects : enable

in VertexData
{
    vec2 uv;
} fs_in;

layout(location=0) out vec4 o_frag;

uniform sampler2D u_texture;

void main()
{
    vec4 c = texture(u_texture, fs_in.uv);
    o_frag = vec4(c.rgb, 1.0);
}
"#;

#[derive(Debug)]
pub struct App {
    // Diablo MPQ archive
    mpq: Archive,
}

impl App {
    pub fn init() -> anyhow::Result<Self> {
        let mpq = Archive::open("data/DIABDAT.MPQ")?;
        Ok(Self {
            mpq
        })
    }
    
    pub fn run(self) -> anyhow::Result<()> {
        use glfw::Context;

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .context("Failed to initialize GLFW3")?;

        glfw.window_hint(WindowHint::Resizable(false));
        glfw.window_hint(WindowHint::SRgbCapable(true));
        glfw.window_hint(WindowHint::DoubleBuffer(true));
        glfw.window_hint(WindowHint::ContextVersion(3, 3));
        glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
        // glfw.window_hint(WindowHint::OpenGlDebugContext(cfg!(debug_assertions)));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        
        let (mut window, _events) = glfw
            .create_window(SCREEN_WIDTH, SCREEN_HEIGHT, TITLE, glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window")?;

        window.set_key_polling(true);
        window.make_current();

        gl::load_with(|s| glfw.get_proc_address_raw(s));

        let vertex_shader = unsafe {
            let shader_str = CString::new(VERTEX_SHADER.as_bytes())?;
            let shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(shader, 1, &shader_str.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);
            shader
        };

        let fragment_shader = unsafe {
            let shader_str = CString::new(FRAGMENT_SHADER.as_bytes())?;
            let shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(shader, 1, &shader_str.as_ptr(), std::ptr::null());
            gl::CompileShader(shader);
            shader
        };

        let program = unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
            program
        };

        let vbo = unsafe {
            let mut buf = 0;
            gl::GenBuffers(1, &mut buf);
            gl::BindBuffer(gl::ARRAY_BUFFER, buf);
            gl::BufferData(gl::ARRAY_BUFFER, 0, std::ptr::null(), gl::STATIC_DRAW);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            buf
        };

        let vao = unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            vao
        };

        let tex_handle = unsafe {
            let title = self.mpq.get_file("ui_art\\title.pcx")?;
            let (width, height, data) = title.read_as_pcx()?;

            let mut handle = 0u32;
            gl::GenTextures(1, &mut handle as *mut u32);
            gl::BindTexture(gl::TEXTURE_2D, handle);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexImage2D(gl::TEXTURE_2D, 
                0, gl::RGB as i32, width as i32, height as i32, 
                0, gl::RGB as u32, gl::UNSIGNED_BYTE, data.as_ptr() as *const c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);

            handle
        };

        let tex_location  = unsafe {
            let location = CString::new("u_texture".as_bytes())?;
            gl::GetUniformLocation(program, location.as_ptr())
        };

        while !window.should_close() {
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
                gl::UseProgram(program);
                {
                    gl::Uniform1i(tex_location, 0);
                    
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, tex_handle);
                    {
                        gl::BindVertexArray(vao);
                        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                        gl::DrawArrays(gl::TRIANGLES, 0i32, 3i32);
                        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                        gl::BindVertexArray(0);
                    }
                    gl::BindTexture(gl::TEXTURE_2D, 0);
                }
                gl::UseProgram(0);
            }

            window.swap_buffers();
            glfw.poll_events();
            
            /*
            for (_, event) in glfw::flush_messages(&events) {
                handle_window_event(&mut window, event);
            }
            */
        }
        Ok(())
    }
}
