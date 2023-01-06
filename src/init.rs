use glfw::{WindowHint, OpenGlProfileHint};

use anyhow::Context;

use crate::mpq::Archive;

use crate::gfx::Image;

use crate::gfx::{Bindable, Topology, Format, Filtering};
use crate::gfx::{Shader, Pipeline, VertexArray, Texture};

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
    o_frag = texture(u_texture, fs_in.uv);
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
        glfw.window_hint(WindowHint::OpenGlDebugContext(cfg!(debug_assertions)));
        glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        
        let (mut window, _events) = glfw
            .create_window(SCREEN_WIDTH, SCREEN_HEIGHT, TITLE, glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window")?;

        window.set_key_polling(true);
        window.make_current();

        gl::load_with(|s| glfw.get_proc_address_raw(s));

        let vertex_shader = Shader::vertex(VERTEX_SHADER, None)?;
        let fragment_shader = Shader::fragment(FRAGMENT_SHADER, None)?;

        let pipeline = Pipeline::new(Topology::Triangles, &[ &vertex_shader, &fragment_shader ])?;

        let vertex_array = VertexArray::new();

        let texture = {
            let title_file = self.mpq.get_file("ui_art\\title.pcx")?;
            let title_image = Image::read_pcx(&title_file, None)?;

            Texture::new(
                title_image.width, 
                title_image.height, 
                Format::R8g8b8a8_uint, 
                Filtering::Nearest,
                &title_image.pixels)?
        };

        /*
        let tex_location = unsafe {
            let location = CString::new("u_texture".as_bytes())?;
            gl::GetUniformLocation(program, location.as_ptr())
        };
        */

        while !window.should_close() {
            unsafe {
                gl::Enable(gl::SCISSOR_TEST);
                gl::Disable(gl::CULL_FACE);

                gl::Enable(gl::FRAMEBUFFER_SRGB);

                gl::Enable(gl::BLEND);
                gl::BlendEquation(gl::FUNC_ADD);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                gl::Viewport(0, 0, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
                gl::Scissor(0, 0, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);

                gl::Clear(gl::COLOR_BUFFER_BIT);
                
                pipeline.bind();
                {
                    // gl::Uniform1i(tex_location, 0);
                    
                    texture.bind_at(0);
                    {
                        vertex_array.bind();
                        gl::DrawArrays(gl::TRIANGLES, 0i32, 3i32);
                        vertex_array.unbind();
                    }
                    texture.unbind();
                }
                pipeline.unbind();
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
