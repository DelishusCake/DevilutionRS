use std::io::BufReader;

use anyhow::Context;

use crate::mpq::Archive;

pub const TITLE: &str = "Diablo";
pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 480;

#[derive(Debug)]
pub struct App {
    // Diablo MPQ archive
    mpq: Archive,
}

impl App {
    pub fn init() -> anyhow::Result<Self> {
        let mpq = Archive::open("data/DIABDAT.MPQ")?;
        
        let title = mpq.get_file("ui_art\\title.pcx")?;

        let reader = pcx::Reader::new(BufReader::with_capacity(title.size(), title))
            .context("Failed to read PCX file")?;
        println!("{:?} {:?}", reader.dimensions(), reader.is_paletted());

        Ok(Self {
            mpq
        })
    }
    
    pub fn run(self) -> anyhow::Result<()> {
        use glfw::Context;

        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)
            .context("Failed to initialize GLFW3")?;
        
        let (mut window, _events) = glfw
            .create_window(SCREEN_WIDTH, SCREEN_HEIGHT, TITLE, glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window")?;

        window.set_key_polling(true);
        window.make_current();

        while !window.should_close() {
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

