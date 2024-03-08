mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;
mod helpers;

use crate::logger;
use crate::imgui::InputData;
use crate::errors::messages::SIMPLE_VK_FN;
use crate::errors::messages::VK_UNRECOVERABLE;

use super::window::Window;
use super::constants;

use objects::DestructionStack;
use objects::VkWrapper;
use objects::VkDestructor;
use objects::VkDestructorType;
use objects::VkDeferedWrapper;
use objects::VkDeferedDestructor;
use objects::VkDynamicDestructor;
use objects::VkDestructorArguments;

use ash::vk;
use nalgebra as na;
use na::Vector3;


#[allow(dead_code)]
pub struct VInit {
    
    
    frame_control: FrameControl,
    
    resize_required: bool,

    
    pub instance: VkWrapper<Instance>,
    messenger: Option<VkWrapper<DMessenger>>,
    surface: VkWrapper<Surface>,
    pub p_device: PDevice,
    pub device: VkWrapper<Device>,
    allocator: VkWrapper<Allocator>,
    pub swapchain: VkWrapper<Swapchain>,
    
    pub command_control: VkWrapper<CommandControl>,
    
    
    render_image: VkWrapper<Image>,
    render_extent: vk::Extent2D,
    depth_image: VkWrapper<Image>,
    
    
    ds_layout: VkWrapper<DescriptorLayout>,
    ds_pool: VkWrapper<GDescriptorAllocator>,
    ds_set: vk::DescriptorSet,
    
    compute_effects: VkWrapper<ComputeEffects>,
    
    mesh_pipeline: VkWrapper<GPipeline>,
    mesh_assets: VkWrapper<MeshAssets>,
    
    compute_effect_index: usize,
    mesh_index: usize,
    field_of_view:na::Vector3<f32>,
    downscale_coheficient:f32,
    
    pub render:graphics::Graphics,
    
    frames_data: VkWrapper<graphics::FramesData>,
    
    fuzzy_sampler: VkWrapper<Sampler>,
    pixelated_sampler: VkWrapper<Sampler>,
    
    scene_data: graphics::GPUSceneData,
    
    gpu_scene_layout: VkWrapper<DescriptorLayout>,
}


impl VInit {
    pub fn init(window:&mut Window) -> VInit {
        
        //panic!("{:?}", ImageMetadata::texture("a"));
        
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
        let swapchain = vk_create_interpreter(Swapchain::create(&mut instance, &surface, &p_device, &mut device), "swapchain");
        let mut command_control = vk_create_interpreter(CommandControl::create(&p_device, &mut device), "command_control");
        
        let render_image = vk_create_interpreter(Image::create(&mut device, &mut allocator, swapchain.extent.into(), image::RENDER, None), "render_image");
        let render_extent = render_image.get_extent2d();
        let depth_image = vk_create_interpreter(Image::create(&mut device, &mut allocator, swapchain.extent.into(), image::DEPTH, None), "depth_image");
        
        let (ds_layout, ds_pool, ds_set) = init_descriptors(&mut device, &render_image);
        
        let compute_effects = c_pipeline::init_pipelines(&mut device, &ds_layout);
        
        let render = Graphics::new(&mut device, &mut allocator).unwrap();
        
        let mesh_pipeline = g_pipeline::init_mesh_pipeline(&mut device, &render_image, &depth_image);
        let mesh_assets = load_gltf(&mut device, &mut allocator, &mut command_control,"res/gltf/basicmesh.glb").expect("runtime error");
        
        
        let frames_data = FramesData::create(&p_device, &mut device).unwrap();
        
        let mut ds_layout_builder = DescriptorLayoutBuilder::create().unwrap();
        ds_layout_builder.add_binding(0, vk::DescriptorType::UNIFORM_BUFFER, 1);
        let (gpu_scene_layout, _types_in_layout) = ds_layout_builder.build(&mut device, vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT).unwrap();
        
        
        let mut d_queue = objects::DestructionStack::new();
        
        let white_pixel:[u8; 4] = std::array::from_fn(|_|0xffu8);
        let texture_extent = vk::Extent3D{width:1, height:1, depth:1};
        let mut white_texture = objects::VkDeferedWrapper::new(Image::create_texture(&mut device, &mut allocator, &mut command_control, texture_extent, None, &white_pixel).unwrap());
        //let (dynamic_destructor, _) = white_texture.defered_destruct();
        let dynamic_destructor = white_texture.defered_destruct();
        //white_texture.destruct(VkDestructorArguments::DevAll(&mut device, &mut allocator));
        d_queue.push(dynamic_destructor);
        /*
        */
        
        let mut buffer = objects::VkDeferedWrapper::new(Buffer::create(&mut device, &mut allocator, Some("debug TEST buffer"), 256, vk::BufferUsageFlags::TRANSFER_SRC, gpu_allocator::MemoryLocation::CpuToGpu).unwrap());
        let dynamic_destructor = buffer.defered_destruct();
        //let (dynamic_destructor, _) = buffer.defered_destruct();
        //dynamic_destructor(VkDestructorArguments::DevAll(&mut device, &mut allocator));
        d_queue.push(dynamic_destructor);
        //std::mem::forget(d_queue);
        d_queue.dispatch(&mut device, &mut allocator);
        //buffer.destruct(VkDestructorArguments::DevAll(&mut device, &mut allocator));
        //println!("{:?}", buffer);
        //
        let pixelated_sampler = Sampler::create(&mut device, vk::Filter::NEAREST).unwrap();
        let fuzzy_sampler = Sampler::create(&mut device, vk::Filter::LINEAR).unwrap();
        
        VInit{
            frame_control: FrameControl(0),
            resize_required: false,
            
            instance: VkWrapper::new(instance),
            
            messenger: match messenger {
                Some(holder) => {Some(VkWrapper::new(holder))}
                None => None
            },
            
            p_device: p_device,
            surface: VkWrapper::new(surface),
            device: VkWrapper::new(device),
            allocator: VkWrapper::new(allocator), 
            swapchain: VkWrapper::new(swapchain),
            command_control: VkWrapper::new(command_control),
            
            render_image: VkWrapper::new(render_image),
            render_extent,
            depth_image: VkWrapper::new(depth_image),
            ds_layout: VkWrapper::new(ds_layout),
            ds_pool: VkWrapper::new(ds_pool),
            ds_set: ds_set,
            
            compute_effects: VkWrapper::new(compute_effects),
            compute_effect_index:0,
            
            mesh_pipeline: VkWrapper::new(mesh_pipeline),
            mesh_assets: VkWrapper::new(mesh_assets),
            
            mesh_index: 0,
            
            field_of_view:na::Vector3::new(10000.0,0.01,70.0),
            downscale_coheficient: 1.0,
            
            render: render,
            frames_data: VkWrapper::new(frames_data),
            
            scene_data: GPUSceneData::default(),
            gpu_scene_layout: VkWrapper::new(gpu_scene_layout),
            
            
            pixelated_sampler: VkWrapper::new(pixelated_sampler),
            fuzzy_sampler: VkWrapper::new(fuzzy_sampler)
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
    
    pub fn handle_events(&mut self, window:&Window) {
        if self.resize_required {
            self.wait_idle();
            self.handle_resize(window);
            self.resize_required = false;
        }
    }
    
    pub fn handle_resize(&mut self, window:&Window) {
        let VInit{
            swapchain,
            instance,
            surface,
            p_device,
            device,
            ..
        } = self;
        logger::various_log!("vulkan",
            (logger::Debug, "swapchain and surface rebuild")
        );
        
        let old_surface_holder = surface.take();
        let old_swapchain_holder = swapchain.take();
        
        old_swapchain_holder.destruct(VkDestructorArguments::Dev(device));
        old_surface_holder.destruct(VkDestructorArguments::None);
        
        let new_surface_holder = Surface::create(&window, instance).expect(VK_UNRECOVERABLE);
        let new_swapchaint_holder = Swapchain::create(instance, &new_surface_holder, p_device, device).expect(VK_UNRECOVERABLE);
        
        surface.fill(new_surface_holder);
        swapchain.fill(new_swapchaint_holder);
    }
    
    
    #[inline(always)]
    fn frame_update(&mut self) {
        self.frame_control.frame_update()
    }
    
    fn get_frame(&self) -> usize {
        self.frame_control.get_frame()
    }
    
    pub fn get_compute_effects_metadata(
        &mut self
    ) ->  (
        &mut [ComputeEffectMetadata], 
        &mut [MeshAssetMetadata], 
        &mut usize, 
        &mut Vector3<f32>,
        &mut f32,
    ) {
        (&mut self.compute_effects.metadatas, &mut self.mesh_assets.metadatas, &mut self.mesh_index, &mut self.field_of_view, &mut self.downscale_coheficient)
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
        let VInit{
            instance, 
            messenger, 
            surface, 
            device, 
            allocator, 
            swapchain, 
            command_control, 
            render_image, 
            depth_image, 
            ds_pool, 
            ds_layout, 
            compute_effects, 
            mesh_pipeline, 
            mesh_assets,
            frames_data,
            gpu_scene_layout,
            fuzzy_sampler,
            pixelated_sampler,
            ..
        } = self;
        
        let dev = device;
        let all = allocator;
        
        fuzzy_sampler.destruct(VkDestructorArguments::Dev(dev));
        pixelated_sampler.destruct(VkDestructorArguments::Dev(dev));
        
        gpu_scene_layout.destruct(VkDestructorArguments::Dev(dev));
        frames_data.destruct(VkDestructorArguments::Dev(dev));
        
        mesh_assets.destruct(VkDestructorArguments::DevAll(dev, all));
        mesh_pipeline.destruct(VkDestructorArguments::Dev(dev));
        compute_effects.destruct(VkDestructorArguments::Dev(dev));
        
        ds_pool.destruct(VkDestructorArguments::Dev(dev));
        ds_layout.destruct(VkDestructorArguments::Dev(dev));
        
        render_image.destruct(VkDestructorArguments::DevAll(dev, all));
        depth_image.destruct(VkDestructorArguments::DevAll(dev, all));
        command_control.destruct(VkDestructorArguments::Dev(dev));
        
        swapchain.destruct(VkDestructorArguments::Dev(dev));
        all.destruct(VkDestructorArguments::Dev(dev));
        dev.destruct(VkDestructorArguments::None);
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



