mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;

use super::{
    window::{
        Window,
    },
    constants,
    State,
};


use objects::{
    VkObj,
    VkObjDevDep,
    DeviceDestroy,
    ActiveDestroy,
};

pub struct VInit {
    
    state: State,
    
    current_frame:usize,
    pub mip_level: usize,
    
    //pub model: Model,
    pub instance: VkObj<Instance>,
    pub messenger: Option<VkObj<DMessenger>>,
    pub surface: VkObj<Surface>,
    pub p_device: PDevice,
    pub device: VkObj<Device>,
    pub depth_buffer: VkObjDevDep<DepthBuffer>,
    pub render_pass: VkObjDevDep<RenderPass>,
    pub swapchain: VkObjDevDep<Swapchain>,
    pub pipeline: VkObjDevDep<Pipeline>,
    pub command_control: VkObjDevDep<CommandControl>,
    pub sync_objects: VkObjDevDep<SyncObjects>,
    pub sampler: VkObjDevDep<Sampler>,
    pub uniform_buffers: VkObjDevDep<UniformBuffers>,
    pub descriptor_control: VkObjDevDep<DescriptorControl>,
    
    pub model_vec: VkObjDevDep<Vec<Model>>,
}


impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        
        
        let state_ref = &state;
        
        
        let instance = vk_create_interpreter(state_ref, Instance::create(state_ref, window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(state_ref, &instance) {
                Ok(messenger) => {
                    if state_ref.v_nor() {
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
        
        let surface =  vk_create_interpreter(state_ref, Surface::create(state_ref, &window, &instance), "surface"); 
        let p_device = vk_create_interpreter(state_ref, PDevice::chose(state_ref, &instance, &surface), "p_device selected"); 
        let device = vk_create_interpreter(state_ref, Device::create(state_ref, &instance, &p_device), "device"); 
        let swapchain_basic = vk_create_interpreter(state_ref, Swapchain::create(state_ref, &window, &instance, &surface, &p_device, &device), "swapchain");
        let depth_buffer = vk_create_interpreter(state_ref, DepthBuffer::create(state_ref, &instance, &p_device, &device, &swapchain_basic), "depth_buffer");
        let render_pass = vk_create_interpreter(state_ref, RenderPass::create(state_ref, &device, &swapchain_basic, &depth_buffer), "render_pass");
        let swapchain = vk_create_interpreter(state_ref, Swapchain::complete(state_ref, &device, swapchain_basic, &depth_buffer, &render_pass), "framebuffer");
        let layout = vk_create_interpreter(state_ref, DescriptorControl::create(state_ref, &device), "descriptor_set_layout");
        let pipeline = vk_create_interpreter(state_ref, Pipeline::create(state_ref, &device, &render_pass, &layout), "pipeline");
        let command_control = vk_create_interpreter(state_ref, CommandControl::create(state_ref, &p_device, &device), "command_control");
        let sync_objects = vk_create_interpreter(state_ref, SyncObjects::create(state_ref, &device), "sync_objects");
        let sampler = vk_create_interpreter(state_ref, Sampler::create(state_ref, &p_device, &device), "sampler");
        let uniform_buffers = vk_create_interpreter(state_ref, UniformBuffers::create(state_ref, &p_device, &device), "uniform_buffer");
        
        let mut model_vec = VkObjDevDep::new(Vec::new());
        let model = vk_create_interpreter(state_ref, Model::vk_load(state_ref, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        /*
        */
        
        /*
        let model = vk_create_interpreter(state_ref, Model::vk_load(state_ref, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        */
        
        let descriptor_control = vk_create_interpreter(state_ref, DescriptorControl::complete(state_ref, &device, layout, &sampler, &mut model_vec[..], &uniform_buffers), "descriptor_control");
        
        
        
        VInit{
            state: state,
            current_frame: 0,
            mip_level: 1,
            //model: viking_house,
            instance: VkObj::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(VkObj::new(holder))}
                None => None
            },
            p_device: p_device,
            surface: VkObj::new(surface),
            device: VkObj::new(device),
            depth_buffer: VkObjDevDep::new(depth_buffer),
            render_pass: VkObjDevDep::new(render_pass),
            pipeline: VkObjDevDep::new(pipeline),
            swapchain: VkObjDevDep::new(swapchain),
            command_control: VkObjDevDep::new(command_control),
            sync_objects: VkObjDevDep::new(sync_objects),
            sampler: VkObjDevDep::new(sampler),
            uniform_buffers: VkObjDevDep::new(uniform_buffers),
            descriptor_control: VkObjDevDep::new(descriptor_control),
            model_vec: model_vec,
        }
    }
    
    #[inline(always)]
    pub fn wait_idle(&self) {
        unsafe{self.device.device_wait_idle()}.expect("waiting for iddle should not fail");
    }
    
    #[inline(always)]
    fn frame_update(&mut self) {
        use constants::fif;
        self.current_frame = (self.current_frame + 1) % fif::USIZE;
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
        
        self.model_vec.device_drop(&self.state, &self.device);
        
        self.descriptor_control.device_drop(&self.state, &self.device);
        self.uniform_buffers.device_drop(&self.state, &self.device);
        self.sampler.device_drop(&self.state, &self.device);
        self.sync_objects.device_drop(&self.state, &self.device);
        self.command_control.device_drop(&self.state, &self.device);
        self.pipeline.device_drop(&self.state, &self.device);
        self.render_pass.device_drop(&self.state, &self.device);
        self.depth_buffer.device_drop(&self.state, &self.device);
        self.swapchain.device_drop(&self.state, &self.device);
        self.device.active_drop(&self.state);
        self.surface.active_drop(&self.state);
        
        match &mut self.messenger {
            Some(ref mut messenger) => {
                messenger.active_drop(&self.state);
            }
            None => {
                if self.state.v_nor() {
                    println!("No Messenger to delete");
                }
            }
        }
        self.instance.active_drop(&self.state);
    }
}

