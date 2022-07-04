mod ground;
mod primitives;
mod state;
mod xscreensaver;

use xscreensaver::ScreensaverWindow;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();
    if let Ok(window) = ScreensaverWindow::new() {
        let mut setup = state::State::setup(&window, 30).await;
        loop {
            for event in window.process_events() {
                match event {
                    xscreensaver::Event::Resized { width, height } => setup.resize(width, height),
                }
            }
            setup.render();
            setup.tick();
        }
    }
}
