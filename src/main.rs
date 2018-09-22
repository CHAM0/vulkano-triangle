#[macro_use]
extern crate vulkano;
extern crate winit;
extern crate vulkano_win;
#[macro_use]
extern crate vulkano_shader_derive;

use std::sync::Arc;

use vulkano_win::VkSurfaceBuild;

use vulkano::sync::now;
use vulkano::sync::{ GpuFuture, SharingMode };
use vulkano::format::Format;
use vulkano::instance::{ Instance, InstanceExtensions, ApplicationInfo, Version, PhysicalDevice, Features };
use vulkano::device::{ Device, DeviceExtensions, Queue};
use vulkano::swapchain::{ Surface, Swapchain, SurfaceTransform, PresentMode, acquire_next_image };
use vulkano::image::{ SwapchainImage, ImageUsage};
use vulkano::buffer::{ BufferUsage, CpuAccessibleBuffer };
use vulkano::pipeline::{ GraphicsPipeline, vertex::BufferlessDefinition, viewport::Viewport, vertex::SingleBufferDefinition, vertex::BufferlessVertices };
use vulkano::framebuffer::{ RenderPassAbstract, Subpass, FramebufferAbstract, Framebuffer };
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::command_buffer::{ AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState };

use winit::{ EventsLoop, WindowBuilder, dpi::LogicalSize, Window };


const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

#[derive(Debug,Clone)]
struct Vertex { position: [f32;2] }
impl_vertex!(Vertex, position);

type ConcreteGraphicsPipeline = GraphicsPipeline<SingleBufferDefinition<Vertex>, std::boxed::Box<vulkano::descriptor::PipelineLayoutAbstract + std::marker::Send + std::marker::Sync>, std::sync::Arc<vulkano::framebuffer::RenderPassAbstract + std::marker::Send + std::marker::Sync>>;


#[allow(unused)]
pub struct Triangle {
    instance: Arc<Instance>,
    events_loop: EventsLoop,
    surface: Arc<Surface<Window>>,
    physical_device_index: u32,
    device: Arc<Device>,
    graphic_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    swapchain_images: Vec<Arc<SwapchainImage<winit::Window>>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    graphics_pipeline: Arc<ConcreteGraphicsPipeline>,
    swapchain_framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
    command_buffers: Vec<Arc<AutoCommandBuffer>>,

}

impl Triangle {
    pub fn initialize() -> Self {
        let instance = Self::create_instance();
        let (events_loop, surface) = Self::create_surface(&instance);
        let physical_device_index = Self::pick_physical_device(&instance);
        let (device, graphic_queue, present_queue) = Self::create_logical_device(&instance, physical_device_index);
        let (swapchain, swapchain_images) = Self::create_swap_chain(&instance, &surface, physical_device_index, &device, &graphic_queue);
        let render_pass = Self::create_render_pass(&device, swapchain.format());
        let (vertex_buffer, graphics_pipeline) = Self::create_graphics_pipeline(&device, swapchain.dimensions(), &render_pass);
        let swapchain_framebuffers = Self::create_framebuffers(&swapchain_images, &render_pass);

        let mut app = Self {
            instance,
            //debug_callback,

            events_loop,
            surface,

            physical_device_index,
            device,

            graphic_queue,
            present_queue,

            swapchain,
            swapchain_images,

            render_pass,
            vertex_buffer,
            graphics_pipeline,

            swapchain_framebuffers,

            command_buffers: vec![],
        };

        app.create_command_buffers();
        app
    }

    #[allow(unused)]
    pub fn main_loop(&mut self) {
        loop {
            self.draw_frame();

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
        -> (Arc<Device>, Arc<Queue>, Arc<Queue>) {
            let physical = PhysicalDevice::from_index(&instance, physical_device_index as usize).unwrap();
            let indices = Self::find_queue_families(&physical).unwrap();

            let queue_family = physical.queue_family_by_id(indices).unwrap();

            let device_ext = DeviceExtensions {
                khr_swapchain: true,
                .. vulkano::device::DeviceExtensions::none()
            };

            let (device, mut queues) = match Device::new(physical, &Features::none(), &device_ext,
                [(queue_family, 0.5)].iter().cloned()) {
                    Ok(i) => i,
                    Err(err) => panic!("Failed to create device: {:?}", err),
            };

            // Get our queue 
            let graphic_queue = queues.next().unwrap();
            let present_queue = queues.next().unwrap_or_else(|| graphic_queue.clone());

            (device, graphic_queue, present_queue)
    }

    fn create_swap_chain (
        instance: &Arc<Instance>,
        surface: &Arc<Surface<Window>>,
        physical_device_index: u32,
        device: &Arc<Device>,
        graphic_queue: &Arc<Queue>,
        ) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
            let physical_device = PhysicalDevice::from_index(&instance, physical_device_index as usize).unwrap();
            let caps = surface.capabilities(physical_device)
                .expect("Failed to get surface capabilities");

            let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            let image_count = caps.min_image_count;
            let image_usage = caps.supported_usage_flags;

            let (swapchain, images) = Swapchain::new(device.clone(), surface.clone(), image_count,
                format, dimensions, 1, image_usage, graphic_queue, caps.current_transform,
                alpha, PresentMode::Fifo, true, None).expect("Failed to create swapchain");
            
            (swapchain, images)
    }

    fn create_graphics_pipeline(device: &Arc<Device>, swap_chain_extent: [u32; 2],
        render_pass: &Arc<RenderPassAbstract + Send + Sync>) -> ( Arc<CpuAccessibleBuffer<[Vertex]>>, Arc<ConcreteGraphicsPipeline>) {

        let vertex_positions = [ 
            Vertex { position: [0.0, -0.5] },
            Vertex { position: [0.5, 0.5] },
            Vertex { position: [-0.5, 0.5] }

        ];

        let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
            vertex_positions
                .into_iter()
                .cloned())
            .expect("Failed to create buffer");

        #[allow(unused)]
        mod vs {
            #[derive(VulkanoShader)]
            #[ty = "vertex"]
            #[path = "src/shaders/vertex_shader.glsl"]
            #[allow(dead_code)]

            struct Dummy;
        }
        #[allow(unused)]
        mod fs {
            #[derive(VulkanoShader)]
            #[ty = "fragment"]
            #[path = "src/shaders/fragment_shader.glsl"]
            #[allow(dead_code)]

            struct Dummy;
        }

        let vs = vs::Shader::load(device.clone())
            .expect("Failed to create shader module");
        let fs = fs::Shader::load(device.clone())
            .expect("Failed to create shader module");

        
        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0 .. 1.0,
        };


        let graphics_pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
        );

        (vertex_buffer, graphics_pipeline)
    }

    fn create_render_pass(device: &Arc<Device>, color_format: Format)
        -> Arc<RenderPassAbstract + Send + Sync> {
        
        Arc::new(single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap())
    }
    
    fn create_framebuffers(swapchain_images: &Vec<Arc<SwapchainImage<Window>>>,
        render_pass: &Arc<RenderPassAbstract + Send + Sync>) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
        swapchain_images.iter()
            .map(|image| {
                let fba: Arc<FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(render_pass.clone())
                    .add(image.clone()).unwrap()
                    .build().unwrap());
                    
                    fba
            }
        ).collect::<Vec<_>>()
    }

    fn create_command_buffers(&mut self) {
        let queue_family = self.graphic_queue.family();
        let physical = PhysicalDevice::from_index(&self.instance, self.physical_device_index as usize).unwrap();
        let caps = self.surface.capabilities(physical)
            .expect("failed to get surface capabilities");
        let dimensions = caps.current_extent.unwrap_or([1024, 768]);
        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]),
            scissors: None,
        };

        self.command_buffers = self.swapchain_framebuffers.iter()
            .map(|framebuffer| {
                //let vertices = BufferlessVertices { vertices: 3, instances: 1 };
                Arc::new(AutoCommandBufferBuilder::primary_simultaneous_use(self.device.clone(), queue_family)
                    .unwrap()
                    .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()])
                    .unwrap()
                    .draw(self.graphics_pipeline.clone(), &dynamic_state,
                        self.vertex_buffer.clone(), (), ())
                    .unwrap()
                    .end_render_pass()
                    .unwrap()
                    .build()
                    .unwrap())
            })
            .collect();       
    }
    
    fn draw_frame(&mut self) {
        let (image_index, acquire_future) = acquire_next_image(self.swapchain.clone(), None).unwrap();

        let command_buffer = self.command_buffers[image_index].clone();

        let future = acquire_future
            .then_execute(self.graphic_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.present_queue.clone(), self.swapchain.clone(), image_index)
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();            
    }
}






fn main() {
    let mut app = Triangle::initialize();
    app.main_loop();
}
