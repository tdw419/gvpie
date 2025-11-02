mod input_mux;

use input_mux::InputMultiplexer;
#[cfg(not(feature = "headless"))]
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

struct App {
    #[cfg(not(feature = "headless"))]
    window: Option<Window>,
    input_mux: InputMultiplexer,
}

impl App {
    fn new() -> Self {
        log::info!("Creating App");
        Self {
            #[cfg(not(feature = "headless"))]
            window: None,
            input_mux: InputMultiplexer::new("/tmp/gvpie_input.bin"),
        }
    }
}

#[cfg(not(feature = "headless"))]
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("App resumed");
        if self.window.is_some() {
            return;
        }
        log::info!("Creating window");
        let window_attrs = WindowAttributes::default().with_title("gvpie-bootstrap");
        let window = event_loop.create_window(window_attrs).expect("window");
        self.window = Some(window);
        log::info!("Window created");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::Ime(winit::event::Ime::Commit(ref text)) = event {
            for ch in text.chars() {
                self.process_input(Some(ch));
            }
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {}
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.process_input(None);
    }
}

impl App {
    fn process_input(&mut self, keyboard_char: Option<char>) {
        if let Some(input) = self.input_mux.next_command(keyboard_char) {
            match input {
                input_mux::InputSource::Keyboard(ch) => {
                    println!("Keyboard input: {}", ch);
                }
                input_mux::InputSource::FileCommand(data) => {
                    println!("File command: {:?}", data);
                }
            }
        }
    }
}

fn main() {
    env_logger::init();
    log::info!("Starting up");

    if std::env::var("GVPIE_HEADLESS").is_ok() {
        let mut app = App::new();
        log::info!("App created in headless mode");
        loop {
            app.process_input(None);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    } else {
        #[cfg(not(feature = "headless"))]
        {
            let event_loop = EventLoop::new().unwrap();
            let mut app = App::new();
            log::info!("App created in windowed mode");
            event_loop.run_app(&mut app).unwrap();
        }
    }
}
