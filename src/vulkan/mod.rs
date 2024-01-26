mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;
mod helpers;

mod logger;
use logger::base as b_logger;

use crate::errors::messages::SIMPLE_VK_FN;
use crate::imgui::InputData;

use super::window::Window;
use super::constants;
use super::State;

use objects::VkWraper;
use objects::VkDestructor;
//use objects::VkDestructorType;
use objects::VkDestructorArguments;

use ash::vk;


#[allow(dead_code)]
pub struct VInit {
    
    
    frame_control: FrameControl,
    
    //pub deletion_queue: VecDeque<dyn FnOnce()>,
    
    //pub model: Model,
    
    pub instance: VkWraper<Instance>,
    messenger: Option<VkWraper<DMessenger>>,
    surface: VkWraper<Surface>,
    pub p_device: PDevice,
    pub device: VkWraper<Device>,
    allocator: VkWraper<Allocator>,
    pub swapchain: VkWraper<Swapchain>,
    sync_objects: VkWraper<SyncObjects>,
    pub command_control: VkWraper<CommandControl>,
    
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
    
    //cp_pipeline: VkWraper<ComputePipeline>,
    
    compute_effects: VkWraper<ComputeEffects>,
    
    cp_index: usize,
    test: [f32; 2],
    
    
    //pub imgui: VkWraper<Imgui>,
    
    pub render:graphics::Graphics,
    
}


impl VInit {
    pub fn init(state:State, window:&mut Window) -> VInit {
        
        let mut instance = vk_create_interpreter(Instance::create(&state, window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&instance) {
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
        
        let surface =  vk_create_interpreter(Surface::create(&window, &mut instance), "surface"); 
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
        //let cp_pipeline = pipeline::init_pipeline(&mut device, &ds_layout);
        let compute_effects = pipeline::init_pipelines(&mut device, &ds_layout);
        
        
        //let imgui_allocator = vk_create_interpreter(Allocator::create(&instance, &p_device, &device), "allocator").into_inner();
        //let imgui = Imgui::create(window, &mut device, &swapchain, &command_control.pool, imgui_allocator);
        
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
            compute_effects: VkWraper::new(compute_effects),
            cp_index:0,
            test: Default::default(),
            
            render: render,
        }
    }
    
    pub fn gui_tick(&mut self, data:&InputData) {
        self.cp_index = data.background_index;
        for index in 0..4 {
            self.compute_effects.metadatas[data.background_index].data[index] = data.push_constants[index];
        }
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
    
    pub fn get_compute_effects_metadata(&mut self) ->  &mut [ComputeEffectMetadata] {
        &mut self.compute_effects.metadatas
    }
    
}



#[inline]
pub fn vk_create_interpreter<T, A:std::fmt::Debug>(result:Result<T, A>, name:&str) -> T {
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
        let (instance, messenger, surface, mut _device, mut _allocator, swapchain, sync_objects, command_control, render_image, ds_layout_builder, ds_pool, ds_layout, compute_effects) = self.destructure();
        
        let dev = &mut _device;
        let allocator = &mut _allocator;
        
        
        //imgui.destruct(VkDestructorArguments::Dev(dev));
        
        compute_effects.destruct(VkDestructorArguments::Dev(dev));
        
        //cp_pipeline.destruct(VkDestructorArguments::Dev(dev));
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
    
    #[allow(dead_code)]
    #[inline(always)]
    fn get_frame_count(&self) -> usize {
        self.0
    }
    #[inline(always)]
    fn frame_update(&mut self) {
        self.0 += 1;
    }
}



