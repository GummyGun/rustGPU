mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;
mod helpers;

mod logger;
use logger::base as b_logger;

use super::window::Window;
use super::constants;
use super::State;

use objects::VkWraper;
use objects::VkObjDevDep;
use objects::DeviceDestroy;

use objects::VkDestructor;
use objects::DestructorType;
use objects::DestructorArguments;

use ash::vk;

pub struct VInit {
    
    state: State,
    
    frame_control: FrameControl,
    //pub deletion_queue: VecDeque<dyn FnOnce()>,
    
    //pub model: Model,
    pub instance: VkWraper<Instance>,
    pub messenger: Option<VkWraper<DMessenger>>,
    pub surface: VkWraper<Surface>,
    pub p_device: PDevice,
    pub device: VkWraper<Device>,
    //pub allocator: std::mem::ManuallyDrop<Allocator>,
    pub allocator: VkWraper<Allocator>,
    pub swapchain: VkObjDevDep<Swapchain>,
    pub sync_objects: VkObjDevDep<SyncObjects>,
    pub command_control: VkObjDevDep<CommandControl>,
    /*
    pub depth_buffer: VkObjDevDep<DepthBuffer>,
    pub pipeline: VkObjDevDep<Pipeline>,
    pub sampler: VkObjDevDep<Sampler>,
    pub uniform_buffers: VkObjDevDep<UniformBuffers>,
    */
    //pub descriptor_control: VkObjDevDep<DescriptorControl>,
    //pub model_vec: VkObjDevDep<Vec<Model>>,
    
    render_image: VkWraper<Image2>,
    render_extent: vk::Extent2D,
    
    ds_layout_builder: VkWraper<DescriptorLayoutBuilder>,
    ds_layout: VkWraper<DescriptorLayout>,
    ds_pool: VkWraper<DescriptorPoolAllocator>,
    ds_set: vk::DescriptorSet,
    /*
    */
    
    pub render:graphics::Graphics,
    
}


impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        
        let instance = vk_create_interpreter(Instance::create(&state, window), "instance"); 
        
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
        
        
        let render_image = vk_create_interpreter(Image2::create(&mut device, &mut allocator, swapchain.extent.into(), image2::RENDER), "render_image");
        let render_extent = vk::Extent2D::default();
        
        //render_image.destruct(DestructorArguments::DevAll(&mut device, &mut allocator));
        
        
        let mut ds_layout_builder = vk_create_interpreter(DescriptorLayoutBuilder::create(), "descriptor_layout_builder");
        ds_layout_builder.add_binding(0, vk::DescriptorType::STORAGE_IMAGE);
        /*
        descriptor_layout_builder.add_binding(3, vk::DescriptorType::UNIFORM_BUFFER);
        descriptor_layout_builder.add_binding(4, vk::DescriptorType::SAMPLER);
        */
        let (ds_layout, mut types_in_layout) = ds_layout_builder.build(&mut device, vk::ShaderStageFlags::VERTEX).unwrap();
        
        types_in_layout *= 10;//allocate 10 DS
        
        
        let mut ds_pool = DescriptorPoolAllocator::create(&mut device, types_in_layout).unwrap();
        let ds_set = ds_pool.allocate(&mut device, ds_layout).unwrap();
        
        /*
        */
        
        /*
        let mut model_vec = VkObjDevDep::new(Vec::new());
        let model = vk_create_interpreter(Model::vk_load(&state, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        let model = vk_create_interpreter(Model::vk_load(&state, &p_device, &device, &command_control, constants::path::suzanne::metadata(), constants::path::suzanne::load_transformations()), "Model");
        model_vec.push(model);
        let descriptor_control = vk_create_interpreter(DescriptorControl::complete(&state, &device, layout, &sampler, &mut model_vec[..], &uniform_buffers), "descriptor_control");
        */
        
        let render = Graphics::new(&mut device, &mut allocator).unwrap();
        
        //None::<i32>.expect("todo");
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
            swapchain: VkObjDevDep::new(swapchain),
            sync_objects: VkObjDevDep::new(sync_objects),
            command_control: VkObjDevDep::new(command_control),
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
            render: render,
            
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
        
        let (instance, messenger, surface, mut _device, mut _allocator, mut swapchain, mut sync_objects, mut command_control, mut _render_image, ds_layout_builder, ds_pool, ds_layout) = self.destructure();
        let state = self.state;
        
        
        let mut device = &mut _device;
        let allocator = &mut _allocator;
        //let render_image = &mut _render_image;
        
        ds_pool.destruct(DestructorArguments::Dev(&mut device));
        ds_layout.destruct(DestructorArguments::Dev(&mut device));
        ds_layout_builder.destruct(DestructorArguments::None);
        
        _render_image.destruct(DestructorArguments::DevAll(device, allocator));
        
        /*
        self.uniform_buffers.device_destroy(&self.state, device);
        self.sampler.device_destroy(&self.state, device);
        self.pipeline.device_destroy(&self.state, device);
        self.depth_buffer.device_destroy(&self.state, device);
        */
        
        command_control.device_destroy(&state, device);
        sync_objects.device_destroy(&state, device);
        swapchain.device_destroy(&state, device);
        
        /*
        let swapchain_destruct = self.swapchain.destroy_callback();
        swapchain_destruct.0(DestructorArguments::Dev(&self.device));
        */
        
        _allocator.destruct(DestructorArguments::Dev(device));
        _device.destruct(DestructorArguments::None);
        surface.destruct(DestructorArguments::None);
        
        match messenger {
            Some(messenger) => {
                messenger.destruct(DestructorArguments::None);
            }
            None => {
                b_logger::debug_messenger_destruct();
            }
        }
        
        instance.destruct(DestructorArguments::None);
        
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



