mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;
mod helpers;

use crate::logger;
use crate::imgui::InputData;
use crate::errors::messages::SIMPLE_VK_FN;

use super::window::Window;
use super::constants;

use objects::VkWraper;
use objects::VkDestructor;
//use objects::VkDestructorType;
use objects::VkDestructorArguments;

use ash::vk;
use nalgebra as na;


#[allow(dead_code)]
pub struct VInit {
    
    
    frame_control: FrameControl,
    
    pub instance: VkWraper<Instance>,
    messenger: Option<VkWraper<DMessenger>>,
    surface: VkWraper<Surface>,
    pub p_device: PDevice,
    pub device: VkWraper<Device>,
    allocator: VkWraper<Allocator>,
    pub swapchain: VkWraper<Swapchain>,
    sync_objects: VkWraper<SyncObjects>,
    pub command_control: VkWraper<CommandControl>,
    
    render_image: VkWraper<Image>,
    render_extent: vk::Extent2D,
    depth_image: VkWraper<Image>,
    
    
    ds_layout_builder: VkWraper<DescriptorLayoutBuilder>,
    ds_layout: VkWraper<DescriptorLayout>,
    ds_pool: VkWraper<DescriptorPoolAllocator>,
    ds_set: vk::DescriptorSet,
    
    compute_effects: VkWraper<ComputeEffects>,
    graphics_pipeline: VkWraper<GPipeline>,
    compute_effect_index: usize,
    
    mesh_pipeline: VkWraper<GPipeline>,
    //mesh: VkWraper<GPUMeshBuffers>,
    mesh_assets: VkWraper<MeshAssets>,
    mesh_index: usize,
    
    
    field_of_view:na::Vector3<f32>,
    
    

    
    pub render:graphics::Graphics,
    
}


impl VInit {
    pub fn init(window:&mut Window) -> VInit {
        
        
        let mut instance = vk_create_interpreter(Instance::create(window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&instance) {
                Ok(messenger) => {
                    messenger
                }
                Err(err) => {panic!("{:?}", err);}
            })
        } else {
            logger::various_log!("debug_messenger",
                (logger::Debug, "DEBUG_MESSENGER NOT ENABLED"),
            );
            None
        };
        
        let surface =  vk_create_interpreter(Surface::create(&window, &mut instance), "surface"); 
        let p_device = vk_create_interpreter(PDevice::chose(&instance, &surface), "p_device selected"); 
        let mut device = vk_create_interpreter(Device::create(&mut instance, &p_device), "device"); 
        let mut allocator = vk_create_interpreter(Allocator::create(&mut instance, &p_device, &mut device), "allocator");
        let swapchain = vk_create_interpreter(Swapchain::create(&window, &mut instance, &surface, &p_device, &mut device), "swapchain");
        let sync_objects = vk_create_interpreter(SyncObjects::create(&mut device), "sync_objects");
        let mut command_control = vk_create_interpreter(CommandControl::create(&p_device, &mut device), "command_control");
        
        let render_image = vk_create_interpreter(Image::create(&mut device, &mut allocator, swapchain.extent.into(), image::RENDER), "render_image");
        let render_extent = render_image.get_extent2d();
        let depth_image = vk_create_interpreter(Image::create(&mut device, &mut allocator, swapchain.extent.into(), image::DEPTH), "depth_image");
        let mut ds_layout_builder = vk_create_interpreter(DescriptorLayoutBuilder::create(), "descriptor_layout_builder");
        
        let (ds_layout, ds_pool, ds_set) = init_descriptors(&mut device, &mut ds_layout_builder, &render_image);
        
        let compute_effects = c_pipeline::init_pipelines(&mut device, &ds_layout);
        
        let render = Graphics::new(&mut device, &mut allocator).unwrap();
        
        let graphics_pipeline = g_pipeline::init_pipeline(&mut device, &render_image, &depth_image);
        
        /*
        let mut buffer = Buffer::create(&mut device, &mut allocator, Some("name"), 255, vk::BufferUsageFlags::INDEX_BUFFER, gpu_allocator::MemoryLocation::CpuToGpu).unwrap();
        println!("{:?}", buffer.get_slice_mut());
        buffer.destruct(VkDestructorArguments::DevAll(&mut device, &mut allocator));
        */
        
        let mesh_pipeline = g_pipeline::init_mesh_pipeline(&mut device, &render_image, &depth_image);
        let mesh_assets = load_gltf(&mut device, &mut allocator, &mut command_control,"res/gltf/basicmesh.glb").expect("runtime error");
        
        
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
            depth_image: VkWraper::new(depth_image),
            ds_layout_builder: VkWraper::new(ds_layout_builder),
            ds_layout: VkWraper::new(ds_layout),
            ds_pool: VkWraper::new(ds_pool),
            ds_set: ds_set,
            
            compute_effects: VkWraper::new(compute_effects),
            compute_effect_index:0,
            
            graphics_pipeline: VkWraper::new(graphics_pipeline),
            
            mesh_pipeline: VkWraper::new(mesh_pipeline),
            mesh_assets: VkWraper::new(mesh_assets),
            mesh_index: 0,
            
            field_of_view:na::Vector3::new(10000.0,0.01,70.0),
            
            render: render,
        }
        
    }
    
    pub fn gui_tick(&mut self, data:&InputData) {
        self.compute_effect_index = data.background_index;
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
    
    pub fn get_compute_effects_metadata(&mut self) ->  (&mut [ComputeEffectMetadata], &mut [MeshAssetMetadata], &mut usize, &mut na::Vector3<f32>,) {
        (&mut self.compute_effects.metadatas, &mut self.mesh_assets.metadatas, &mut self.mesh_index, &mut self.field_of_view)
    }
    
    
    
}



#[inline]
pub fn vk_create_interpreter<T, A:std::fmt::Debug>(result:Result<T, A>, name:&str) -> T {
    match result {
        Ok(device) => {
            device
        }
        Err(err) => {panic!("error in {} {:?}", name, err);}
    }
}

impl Drop for VInit {
    
    fn drop(&mut self) {
        let (
            instance, 
            messenger, 
            surface, 
            mut _device, 
            mut _allocator, 
            swapchain, 
            sync_objects, 
            command_control, 
            render_image, 
            depth_image, 
            ds_layout_builder, 
            ds_pool, 
            ds_layout, 
            compute_effects, 
            graphics_pipeline, 
            mesh_pipeline, 
            mesh_assets,
        ) = self.destructure();
        
        let dev = &mut _device;
        let all = &mut _allocator;
        
        
        mesh_assets.destruct(VkDestructorArguments::DevAll(dev, all));
        mesh_pipeline.destruct(VkDestructorArguments::Dev(dev));
        graphics_pipeline.destruct(VkDestructorArguments::Dev(dev));
        compute_effects.destruct(VkDestructorArguments::Dev(dev));
        
        ds_pool.destruct(VkDestructorArguments::Dev(dev));
        ds_layout.destruct(VkDestructorArguments::Dev(dev));
        ds_layout_builder.destruct(VkDestructorArguments::None);
        
        render_image.destruct(VkDestructorArguments::DevAll(dev, all));
        depth_image.destruct(VkDestructorArguments::DevAll(dev, all));
        command_control.destruct(VkDestructorArguments::Dev(dev));
        
        sync_objects.destruct(VkDestructorArguments::Dev(dev));
        swapchain.destruct(VkDestructorArguments::Dev(dev));
        _allocator.destruct(VkDestructorArguments::Dev(dev));
        _device.destruct(VkDestructorArguments::None);
        surface.destruct(VkDestructorArguments::None);
        
        match messenger {
            Some(messenger) => {
                messenger.destruct(VkDestructorArguments::None);
            }
            None => {
                logger::various_log!("debug_messenger",
                    (logger::Debug, "NO DEBUG_MESSENGER"),
                );
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



