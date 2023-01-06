use diablo::init::App;

fn main() {
    App::init()
        .expect("Failed to initialize application")
        .run()
        .expect("Failed to run application")
}
