mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;

mod logger;

use super::window::Window;
use super::constants;
use super::State;

use objects::VkWraper;
use objects::VkObjDevDep;
use objects::DeviceDestroy;

use objects::VkDestructor;
use objects::DestructorType;
use objects::DestructorArguments;

pub struct VInit {
    
    state: State,
    
    frame_control: FrameControl,
    pub mip_level: usize,
    //pub deletion_queue: VecDeque<dyn FnOnce()>,
    
    //pub model: Model,
    pub instance: VkWraper<Instance>,
    pub messenger: Option<VkWraper<DMessenger>>,
    pub surface: VkWraper<Surface>,
    pub p_device: PDevice,
    pub device: VkWraper<Device>,
    //pub allocator: std::mem::ManuallyDrop<Allocator>,
    pub allocator: VkWraper<Allocator>,
    pub depth_buffer: VkObjDevDep<DepthBuffer>,
    pub swapchain: VkObjDevDep<Swapchain>,
    pub pipeline: VkObjDevDep<Pipeline>,
    pub command_control: VkObjDevDep<CommandControl>,
    pub sync_objects: VkObjDevDep<SyncObjects>,
    pub sampler: VkObjDevDep<Sampler>,
    pub uniform_buffers: VkObjDevDep<UniformBuffers>,
    //pub descriptor_control: VkObjDevDep<DescriptorControl>,
    //pub model_vec: VkObjDevDep<Vec<Model>>,
    
}


impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        
        
        let instance = vk_create_interpreter(&state, Instance::create(&state, window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&state, &instance) {
                Ok(messenger) => {
                    if state.v_nor() {
                        println!("[0]:messenger");
                    }
                    messenger
                }
                Err(err) => {panic!("{:?}", err);}
            })
        } else {
            println!("[X]:messenger");
            None
        };
        
        /*
        let hola = Box::new(|| {
            println!("hola closure");
            1
        });
        println!("before closure");
        println!("{:?}", hola());
        use std::collections::VecDeque;
        
        let mut destruction_queue:VecDeque<Box<dyn FnMut()->()>> = VecDeque::new();
        
        destruction_queue.push_back(Box::new(||{
            println!("0");
        }));
        destruction_queue.push_back(Box::new(||{
            println!("1");
        }));
        destruction_queue.push_back(Box::new(||{
            println!("2");
        }));
        
        destruction_queue.pop_back().unwrap()();
        
        panic!();
        */
        
        let surface =  vk_create_interpreter(&state, Surface::create(&state, &window, &instance), "surface"); 
        let p_device = vk_create_interpreter(&state, PDevice::chose(&state, &instance, &surface), "p_device selected"); 
        let device = vk_create_interpreter(&state, Device::create(&state, &instance, &p_device), "device"); 
        let mut allocator = vk_create_interpreter(&state, Allocator::create(&instance, &p_device, &device), "allocator");
        
        let swapchain = vk_create_interpreter(&state, Swapchain::create(&window, &instance, &surface, &p_device, &device), "swapchain");
        
        let depth_buffer = vk_create_interpreter(&state, DepthBuffer::create(&state, &instance, &p_device, &device, &swapchain), "depth_buffer");
        let pipeline = vk_create_interpreter(&state, Pipeline::create(&state, &device/*, &layout*/), "pipeline");
        let command_control = vk_create_interpreter(&state, CommandControl::create(&state, &p_device, &device), "command_control");
        let sync_objects = vk_create_interpreter(&state, SyncObjects::create(&state, &device), "sync_objects");
        let sampler = vk_create_interpreter(&state, Sampler::create(&state, &p_device, &device), "sampler");
        let uniform_buffers = vk_create_interpreter(&state, UniformBuffers::create(&state, &p_device, &device), "uniform_buffer");
        
        let image2 = Image2::create(&device, &mut allocator, swapchain.extent.into(), image2::RENDER);
        
        /*
        let mut model_vec = VkObjDevDep::new(Vec::new());
        let model = vk_create_interpreter(&state, Model::vk_load(&state, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        */
        
        /*
        let model = vk_create_interpreter(&state, Model::vk_load(&state, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        
        let descriptor_control = vk_create_interpreter(&state, DescriptorControl::complete(&state, &device, layout, &sampler, &mut model_vec[..], &uniform_buffers), "descriptor_control");
        */
        
        
        VInit{
            state: state,
            frame_control: FrameControl(0),
            mip_level: 1,
            
            instance: VkWraper::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(VkWraper::new(holder))}
                None => None
            },
            p_device: p_device,
            surface: VkWraper::new(surface),
            device: VkWraper::new(device),
            allocator: VkWraper::new(allocator), 
            depth_buffer: VkObjDevDep::new(depth_buffer),
            pipeline: VkObjDevDep::new(pipeline),
            swapchain: VkObjDevDep::new(swapchain),
            command_control: VkObjDevDep::new(command_control),
            sync_objects: VkObjDevDep::new(sync_objects),
            sampler: VkObjDevDep::new(sampler),
            uniform_buffers: VkObjDevDep::new(uniform_buffers),
            //descriptor_control: VkObjDevDep::new(descriptor_control),
            //model_vec: model_vec,
        }
    }
    
    #[inline(always)]
    pub fn wait_idle(&self) {
        unsafe{self.device.device_wait_idle()}.expect("waiting for iddle should not fail");
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
fn vk_create_interpreter<T, A:std::fmt::Debug>(state:&State, result:Result<T, A>, name:&'static str) -> T {
    match result {
        Ok(device) => {
            if state.v_nor() {
                println!("[0]:{}", name);
            }
            device
        }
        Err(err) => {panic!("error in {} {:?}", name, err);}
    }
}

impl Drop for VInit {
    fn drop(&mut self) {
        
        //self.model_vec.device_destroy(&self.state, &self.device);
        //self.descriptor_control.device_destroy(&self.state, &self.device);
        
        let VInit{allocator:allocator, ..} = self;
        
        self.uniform_buffers.device_destroy(&self.state, &self.device);
        self.sampler.device_destroy(&self.state, &self.device);
        self.sync_objects.device_destroy(&self.state, &self.device);
        self.command_control.device_destroy(&self.state, &self.device);
        self.pipeline.device_destroy(&self.state, &self.device);
        self.depth_buffer.device_destroy(&self.state, &self.device);
        self.swapchain.device_destroy(&self.state, &self.device);
        
        /*
        let swapchain_destructor = self.swapchain.destroy_callback();
        swapchain_destructor.0(DestructorArguments::Dev(&self.device));
        */
        
        allocator.destruct(DestructorArguments::Dev(&self.device));
        
        self.device.destruct(DestructorArguments::None);
        self.surface.destruct(DestructorArguments::None);
        match &mut self.messenger {
            Some(ref mut messenger) => {
                messenger.destruct(DestructorArguments::None);
            }
            None => {
                if self.state.v_nor() {
                    println!("No Messenger to delete");
                }
            }
        }
        self.instance.destruct(DestructorArguments::None);
        
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

