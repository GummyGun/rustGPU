mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;
mod helpers;

mod logger;
use logger::base as b_logger;

use crate::errors::messages::SIMPLE_VK_FN;

use super::window::Window;
use super::constants;
use super::State;

use objects::VkWraper;
use objects::VkDestructor;
use objects::VkDestructorType;
use objects::VkDestructorArguments;

use ash::vk;

pub struct VInit {
    
    state: State,
    
    frame_control: FrameControl,
    //pub deletion_queue: VecDeque<dyn FnOnce()>,
    
    //pub model: Model,
    instance: VkWraper<Instance>,
    messenger: Option<VkWraper<DMessenger>>,
    surface: VkWraper<Surface>,
    p_device: PDevice,
    device: VkWraper<Device>,
    allocator: VkWraper<Allocator>,
    swapchain: VkWraper<Swapchain>,
    sync_objects: VkWraper<SyncObjects>,
    command_control: VkWraper<CommandControl>,
    /*
    pub depth_buffer: VkObjDevDep<DepthBuffer>,
    pub pipeline: VkObjDevDep<Pipeline>,
    pub sampler: VkObjDevDep<Sampler>,
    pub uniform_buffers: VkObjDevDep<UniformBuffers>,
    */
    //pub descriptor_control: VkObjDevDep<DescriptorControl>,
    //pub model_vec: VkObjDevDep<Vec<Model>>,
    
    render_image: VkWraper<Image>,
    render_extent: vk::Extent2D,
    
    ds_layout_builder: VkWraper<DescriptorLayoutBuilder>,
    ds_layout: VkWraper<DescriptorLayout>,
    ds_pool: VkWraper<DescriptorPoolAllocator>,
    ds_set: vk::DescriptorSet,
    cp_pipeline: VkWraper<ComputePipeline>,
    
    imgui: VkWraper<Imgui>,
    
    pub render:graphics::Graphics,
    
}


impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        
        let mut instance = vk_create_interpreter(Instance::create(&state, window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&state, &instance) {
                Ok(messenger) => {
                    b_logger::debug_messenger_create();
                    messenger
                }
                Err(err) => {panic!("{:?}", err);}
            })
        } else {
            b_logger::no_debug_messenger_create();
            None
        };
        
        let surface =  vk_create_interpreter(Surface::create(&state, &window, &instance), "surface"); 
        let p_device = vk_create_interpreter(PDevice::chose(&state, &instance, &surface), "p_device selected"); 
        let mut device = vk_create_interpreter(Device::create(&state, &instance, &p_device), "device"); 
        let mut allocator = vk_create_interpreter(Allocator::create(&instance, &p_device, &device), "allocator");
        let swapchain = vk_create_interpreter(Swapchain::create(&window, &instance, &surface, &p_device, &mut device), "swapchain");
        let sync_objects = vk_create_interpreter(SyncObjects::create(&state, &device), "sync_objects");
        let command_control = vk_create_interpreter(CommandControl::create(&state, &p_device, &device), "command_control");
        
        /*
        let depth_buffer = vk_create_interpreter(DepthBuffer::create(&state, &instance, &p_device, &device, &swapchain), "depth_buffer");
        let pipeline = vk_create_interpreter(Pipeline::create(&state, &device/*, &layout*/), "pipeline");
        let sampler = vk_create_interpreter(Sampler::create(&state, &p_device, &device), "sampler");
        let uniform_buffers = vk_create_interpreter(UniformBuffers::create(&state, &p_device, &device), "uniform_buffer");
        */
        
        
        let render_image = vk_create_interpreter(Image::create(&mut device, &mut allocator, swapchain.extent.into(), image::RENDER), "render_image");
        let render_extent = render_image.get_extent2d();
        
        
        let mut ds_layout_builder = vk_create_interpreter(DescriptorLayoutBuilder::create(), "descriptor_layout_builder");
        
        let (ds_layout, ds_pool, ds_set) = init_descriptors(&mut device, &mut ds_layout_builder, &render_image);
        let cp_pipeline = init_pipeline(&mut device, &ds_layout);
        
        
        let mut imgui_allocator = vk_create_interpreter(Allocator::create(&instance, &p_device, &device), "allocator").into_inner();
        
        let imgui = Imgui::create(window, &mut instance, &p_device, &mut device, &swapchain, &command_control.pool, imgui_allocator);
        
        /*
        let mut model_vec = VkObjDevDep::new(Vec::new());
        let model = vk_create_interpreter(Model::vk_load(&state, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        let model = vk_create_interpreter(Model::vk_load(&state, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        let descriptor_control = vk_create_interpreter(DescriptorControl::complete(&state, &device, layout, &sampler, &mut model_vec[..], &uniform_buffers), "descriptor_control");
        */
        
        let render = Graphics::new(&mut device, &mut allocator).unwrap();
        
        VInit{
            state: state,
            frame_control: FrameControl(0),
            
            instance: VkWraper::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(VkWraper::new(holder))}
                None => None
            },
            p_device: p_device,
            surface: VkWraper::new(surface),
            device: VkWraper::new(device),
            allocator: VkWraper::new(allocator), 
            swapchain: VkWraper::new(swapchain),
            sync_objects: VkWraper::new(sync_objects),
            command_control: VkWraper::new(command_control),
            
            render_image: VkWraper::new(render_image),
            render_extent,
            /*
            depth_buffer: VkObjDevDep::new(depth_buffer),
            pipeline: VkObjDevDep::new(pipeline),
            sampler: VkObjDevDep::new(sampler),
            uniform_buffers: VkObjDevDep::new(uniform_buffers),
            //descriptor_control: VkObjDevDep::new(descriptor_control),
            //model_vec: model_vec,
            */
            ds_layout_builder: VkWraper::new(ds_layout_builder),
            ds_layout: VkWraper::new(ds_layout),
            ds_pool: VkWraper::new(ds_pool),
            ds_set: ds_set,
            cp_pipeline: VkWraper::new(cp_pipeline),
            imgui: VkWraper::new(imgui),
            
            render: render,
        }
    }
    
    pub fn handle_events(
        &mut self,
        window: &Window,
    ) {
        self.imgui.handle_event(window);
    }
    
    #[inline(always)]
    pub fn wait_idle(&self) {
        unsafe{self.device.device_wait_idle()}.expect(SIMPLE_VK_FN);
    }
    
    #[inline(always)]
    fn frame_update(&mut self) {
        self.frame_control.frame_update()
    }
    
    fn get_frame(&self) -> usize {
        self.frame_control.get_frame()
    }
    
    fn get_frame_count(&self) -> usize {
        self.frame_control.get_frame_count()
    }
    
}



#[inline]
fn vk_create_interpreter<T, A:std::fmt::Debug>(result:Result<T, A>, name:&str) -> T {
    match result {
        Ok(device) => {
            b_logger::create(name);
            device
        }
        Err(err) => {panic!("error in {} {:?}", name, err);}
    }
}

impl Drop for VInit {
    
    fn drop(&mut self) {
        
        let (instance, messenger, surface, mut _device, mut _allocator, mut swapchain, mut sync_objects, mut command_control, render_image, ds_layout_builder, ds_pool, ds_layout, cp_pipeline, imgui) = self.destructure();
        let state = self.state;
        
        
        let dev = &mut _device;
        let allocator = &mut _allocator;
        
        
        imgui.destruct(VkDestructorArguments::Dev(dev));
        cp_pipeline.destruct(VkDestructorArguments::Dev(dev));
        ds_pool.destruct(VkDestructorArguments::Dev(dev));
        ds_layout.destruct(VkDestructorArguments::Dev(dev));
        ds_layout_builder.destruct(VkDestructorArguments::None);
        
        render_image.destruct(VkDestructorArguments::DevAll(dev, allocator));
        
        /*
        self.uniform_buffers.device_destroy(&self.state, dev);
        self.sampler.device_destroy(&self.state, dev);
        self.pipeline.device_destroy(&self.state, dev);
        self.depth_buffer.device_destroy(&self.state, dev);
        */
        
        //command_control.device_destroy(&state, dev);
        command_control.destruct(VkDestructorArguments::Dev(dev));
        
        sync_objects.destruct(VkDestructorArguments::Dev(dev));
        swapchain.destruct(VkDestructorArguments::Dev(dev));
        /*
        let swapchain_destruct = self.swapchain.destroy_callback();
        swapchain_destruct.0(VkDestructorArguments::Dev(&self.dev));
        */
        _allocator.destruct(VkDestructorArguments::Dev(dev));
        _device.destruct(VkDestructorArguments::None);
        surface.destruct(VkDestructorArguments::None);
        
        match messenger {
            Some(messenger) => {
                messenger.destruct(VkDestructorArguments::None);
            }
            None => {
                b_logger::debug_messenger_destruct();
            }
        }
        
        instance.destruct(VkDestructorArguments::None);
        
    }
}

struct FrameControl(usize);

impl FrameControl {
    fn get_frame(&self) -> usize {
        self.0 % constants::fif::USIZE
    }
    fn get_frame_count(&self) -> usize {
        self.0
    }
    #[inline(always)]
    fn frame_update(&mut self) {
        self.0 += 1;
    }
}



