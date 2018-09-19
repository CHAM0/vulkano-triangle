#[macro_use]
extern crate vulkano;
extern crate winit;
extern crate vulkano_win;

use std::sync::Arc;
use vulkano::instance::{ Instance, InstanceExtensions, ApplicationInfo, Version };
use winit::{ EventsLoop, WindowBuilder, dpi::LogicalSize };


const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;


pub struct Triangle {
    instance: Arc<Instance>,
    events_loop: EventsLoop,
}

impl Triangle {
    pub fn initialize() -> Self {
        let events_loop = Self::init_window();
        let instance = Self::create_instance();

        Self {
            events_loop,
            instance,
        }
    }

    pub fn main_loop(&mut self) {
        loop {
            let mut done = false;
            self.events_loop.poll_events(|event| {
                match event {
                    winit::Event::WindowEvent {event: winit::WindowEvent::CloseRequested, .. } => done = true,
                    _ => ()

                }
            });
            if done {
                return;
            }
        }
    }

    fn create_instance() -> Arc<Instance> {
        /*
        for layer in vulkano::instance::layers_list().unwrap() {
            println!("Available layer: {}", layer.name());======
        }
        */

        match InstanceExtensions::supported_by_core() {
            Ok(i) => println!("Supportted extensions: {:?}", i),
            Err(err) => panic!("Failed to retreive supported extensions: {:?}", err),
        };

        let extensions = vulkano_win::required_extensions();
        match Instance::new(None, &extensions, None) {
            Ok(i) => return i,
            Err(err) => panic!("Couldn't build instance: {:?}", err) 
        };
    }

    fn init_window() -> EventsLoop {
        let events_loop = EventsLoop::new();
        let builder = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
            .build(&events_loop);

        events_loop
    }

}






fn main() {
    let mut app = Triangle::initialize();
    app.main_loop();
}
