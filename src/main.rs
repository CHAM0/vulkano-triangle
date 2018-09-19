extern crate vulkano;
extern crate winit;
extern crate vulkano_win;

use std::sync::Arc;
use vulkano::instance::{ Instance, InstanceExtensions, ApplicationInfo, Version, PhysicalDevice, Features };
use vulkano::device::{ Device, DeviceExtensions, Queue};
use vulkano::swapchain::{ Surface };

use vulkano_win::VkSurfaceBuild;
use winit::{ EventsLoop, WindowBuilder, dpi::LogicalSize, Window };


const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;


#[allow(unused)]
pub struct Triangle {
    instance: Arc<Instance>,
    events_loop: EventsLoop,
    surface: Arc<Surface<Window>>,
    physical_device_index: u32,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl Triangle {
    pub fn initialize() -> Self {
        let instance = Self::create_instance();
        let (events_loop, surface) = Self::create_surface(&instance);
        let physical_device_index = Self::pick_physical_device(&instance);
        let (device, queue) = Self::create_logical_device(&instance, physical_device_index);
        
        Self {
            instance,
            events_loop,
            surface,
            physical_device_index,
            device,
            queue,
        }
    }

    #[allow(unused)]
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
        

        match InstanceExtensions::supported_by_core() {
            Ok(i) => println!("Supportted extensions: {:?}", i),
            Err(err) => panic!("Failed to retreive supported extensions: {:?}", err),
        };
        */

        let app_info = ApplicationInfo {
            application_name: Some("Triangle".into()),
            application_version: Some(Version { major: 1, minor: 0, patch: 0 }),
            engine_name: Some("No engine".into()),
            engine_version: Some(Version {major: 1, minor: 0, patch: 0}),
        };

        let extensions = vulkano_win::required_extensions();
        match Instance::new(Some(&app_info), &extensions, None) {
            Ok(i) => return i,
            Err(err) => panic!("Couldn't build instance: {:?}", err) 
        };
    }

    fn create_surface(instance: &Arc<Instance>) -> (EventsLoop, Arc<Surface<Window>>) {
        let events_loop = EventsLoop::new();
        let surface = WindowBuilder::new()
            .with_title("Vulkan")
            .with_dimensions(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
            .build_vk_surface(&events_loop, instance.clone())
            .expect("Failed to create a window surface");

        (events_loop, surface)
    }

    fn pick_physical_device(instance: &Arc<Instance>) -> u32 {
        match PhysicalDevice::enumerate(&instance).next() {
            Some(device) => return Self::find_queue_families(&device)
                .expect("Couldn't find a graphical queue family"),
            None => panic!("No device available")
        };
    }

    fn find_queue_families(device: &PhysicalDevice) -> Option<u32> {
        for (i, queue_family) in device.queue_families().enumerate() {
            if queue_family.supports_graphics() {
                return Some(i as u32);
            };
        }
        return None
    }

    fn create_logical_device(instance: &Arc<Instance>, physical_device_index: u32)
        -> (Arc<Device>, Arc<Queue>) {
            let physical = PhysicalDevice::from_index(&instance, physical_device_index as usize).unwrap();
            let indices = Self::find_queue_families(&physical).unwrap();

            let queue_family = physical.queue_family_by_id(indices).unwrap();

            let (device, mut queues) = match Device::new(physical, &Features::none(), &DeviceExtensions::none(),
                [(queue_family, 0.5)].iter().cloned()) {
                    Ok(i) => i,
                    Err(err) => panic!("Failed to create device: {:?}", err),
            };

            // Get our queue 
            let queue = queues.next().unwrap();

            (device, queue)
        }       
}

fn create_swap_chain (
    instance: &Arc<Instance>,
    surface: &Arc<Surface<Window>>,
    physical_device_index: u32,
    ) {
        let physical_device = PhysicalDevice::from_index(&instance, physical_device_index as usize).unwrap();
        let caps = surface.capabilities(physical_device)
            .expect("Failed to get surface capabilities");

        let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

         
}





fn main() {
    let mut app = Triangle::initialize();
    app.main_loop();
}
