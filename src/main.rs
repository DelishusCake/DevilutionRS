use glfw::{WindowHint, OpenGlProfileHint};

use cgmath::*;
use anyhow::Context;
use crate::screen::GameScreen;

use diablo::mpq::Archive;
use diablo::gfx::*;
use diablo::game::*;

use diablo::game::screen::TitleScreen;

// Window constants
pub const TITLE: &str = "Diablo";
pub const SCREEN_WIDTH: u32 = RENDER_WIDTH;
pub const SCREEN_HEIGHT: u32 = RENDER_HEIGHT;
// Rendering constants
pub const MAX_INDICES: usize = 1024;
pub const MAX_VERTICES: usize = 1024;

fn main() -> anyhow::Result<()> {
    use glfw::Context;

    // Open the Diablo MPQ archive
    // TODO: Hellfire support?
    let diablo_mpq = Archive::open("data/DIABDAT.MPQ")?;

    // Initalize GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
        .context("Failed to initialize GLFW3")?;
    // Set some window hints to get an OpenGL context
    glfw.window_hint(WindowHint::Resizable(true));
    glfw.window_hint(WindowHint::SRgbCapable(true));
    glfw.window_hint(WindowHint::DoubleBuffer(true));
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlDebugContext(cfg!(debug_assertions)));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    // Create the window and event handler
    let (mut window, events) = glfw
        .create_window(SCREEN_WIDTH, SCREEN_HEIGHT, TITLE, glfw::WindowMode::Windowed)
        .context("Failed to create GLFW window")?;
    // Bind the window
    window.set_aspect_ratio(RENDER_WIDTH, RENDER_HEIGHT);
    window.set_key_polling(true);
    window.make_current();

    // Load the OpenGL function pointers 
    gl::load_with(|s| glfw.get_proc_address_raw(s));

    // Create a geometry-batching renderer
    let mut batch = Batch::new(MAX_VERTICES, MAX_INDICES); 
    // Initialize the rendering materials
    let materials = MaterialMap::new()?;

    // Initialize at the title screen
    // TODO: Intro video
    let mut screen: Box<dyn GameScreen> = Box::new(TitleScreen::new(&diablo_mpq)?);

    let mut frame_timer = 0.0;
    let frame_rate = 1.0 / 60.0;

    let mut last_time = glfw.get_time();
    while !window.should_close() {
        // Calculate the delta time from last frame
        let now_time = glfw.get_time();
        let delta = now_time - last_time;
        last_time = now_time;
        
        frame_timer += delta;
        while frame_timer >= frame_rate {
            // Clear the batch
            batch.clear();
            // Update and render the current screen
            screen.update_and_render(frame_rate, &mut batch);

            frame_timer -= frame_rate;
        }

        // Get the current window size and the rendering aspect ratio
        let window_size = window.get_framebuffer_size();
        let aspect_ratio = RENDER_WIDTH as f32 / RENDER_HEIGHT as f32;
        // Calculate the viewport and projection matrix
        let viewport = Viewport::from_window(aspect_ratio, window_size);
        let projection = {
            let scale_x = window_size.0 as f32 / RENDER_WIDTH as f32;
            let scale_y = window_size.1 as f32 / RENDER_HEIGHT as f32;
            let scale = Matrix4::from_nonuniform_scale(scale_x, scale_y, 1.0);
            let ortho = ortho(0.0, window_size.0 as f32, window_size.1 as f32, 0.0, -1.0, 1.0);
            ortho*scale
        };
        // Flush the batch to the GPU
        batch.flush(projection);

        // Bind some rendering state to the GPU and clear the screen
        unsafe {
            gl::Enable(gl::SCISSOR_TEST);
            gl::Disable(gl::CULL_FACE);

            gl::Enable(gl::FRAMEBUFFER_SRGB);

            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Viewport(viewport.x, viewport.y, viewport.w, viewport.h);
            gl::Scissor(viewport.x, viewport.y, viewport.w, viewport.h);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);    
        }
        // Render the batch to the screen
        batch.render(&materials);
        // Swap the window buffers and poll the events
        window.swap_buffers();
        glfw.poll_events();
        // Handle each event in the loop
        for (_, event) in glfw::flush_messages(&events) {
            screen.handle_event(&mut window, &event);
        }
    }
    Ok(())
}

/// Viewport structure for aspect-ratio correct rendering
#[derive(Debug, Copy, Clone)]
struct Viewport {
    x: i32,
    y: i32,
    w: i32, 
    h: i32, 
}

impl Viewport {
    /// Calculate the viewport from a given window dimension and a desired aspec ratio
    pub fn from_window(aspect_ratio: f32, window_size: (i32, i32)) -> Self {
        let (width, height) = window_size;
        let mut w = width;
        let mut h = (w as f32 / aspect_ratio + 0.5f32) as i32;
        if h > height {
            h = height;
            w = (height as f32 * aspect_ratio + 0.5f32) as i32;
        }
        let x = (width - w) / 2;
        let y = (height - h) / 2;
        Self { x, y, w, h }
    }
}
