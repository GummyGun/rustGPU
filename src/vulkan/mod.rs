mod init;
pub use init::*;

mod graphics;
use graphics::*;

mod objects;

mod helpers;

mod materials;
use materials::*;

use crate::logger;
use crate::gui::InputData;
use crate::errors::messages::SIMPLE_VK_FN;
use crate::errors::messages::VK_UNRECOVERABLE;

use super::window::Window;
use super::constants;

use objects::DestructionStack;
use objects::VkWrapper;
use objects::VkDestructor;
use objects::VkDestructorType;
use objects::VkDeferedDestructor;
use objects::VkDynamicDestructor;
use objects::VkDestructorArguments;


use ash::vk;
use nalgebra as na;
use na::Vector3;
use arrayvec::ArrayString;


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
    
    canvas: VkWrapper<graphics::Canvas>,
    
    background_image_descriptor_layout: VkWrapper<DescriptorLayout>,
    texture_descriptor_layout: VkWrapper<DescriptorLayout>,
    ds_pool: VkWrapper<GDescriptorAllocator>,
    background_image_ds: vk::DescriptorSet,
    
    compute_effects: VkWrapper<ComputeEffects>,
    
    //mesh_pipeline: VkWrapper<GPipeline>,
    //mesh_assets: VkWrapper<MeshAssets>,
    
    materials: VkWrapper<Materials>,
    mesh_assets: VkWrapper<VkMeshAssets>,
    
    main_draw_context: DrawContext,
    
    compute_effect_index: usize,
    mesh_index: usize,
    field_of_view: na::Vector3<f32>,
    downscale_coheficient: f32,
    
    frames_data: VkWrapper<graphics::FramesData>,
    scene_data: graphics::GPUSceneData,
    gpu_scene_layout: VkWrapper<DescriptorLayout>,
    
    fuzzy_sampler: VkWrapper<Sampler>,
    pixelated_sampler: VkWrapper<Sampler>,
    
    white_texture: VkWrapper<Image>,
    grey_texture: VkWrapper<Image>,
    black_texture: VkWrapper<Image>,
    error_texture: VkWrapper<Image>,
    
    
    destruction_stack: DestructionStack,
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
        
        let mut destruction_stack = objects::DestructionStack::new();
        
        let mut canvas = Canvas::new(&mut device, &mut allocator, swapchain.extent.into()).unwrap();
        let render_image = canvas.get_color();
        
        let (mut ds_pool, background_image_ds, background_image_descriptor_layout, texture_descriptor_layout) = init_descriptors(&mut device, &render_image);
        let compute_effects = c_pipeline::init_pipelines(&mut device, &background_image_descriptor_layout);
        
        
        let frames_data = FramesData::create(&p_device, &mut device).unwrap();
        
        
        let mut ds_layout_builder = DescriptorLayoutBuilder::create();
        ds_layout_builder.add_binding(0, vk::DescriptorType::UNIFORM_BUFFER, 1);
        let (gpu_scene_layout, _types_in_layout) = ds_layout_builder.build(&mut device, vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT).unwrap();
        
        
        let (white_texture, grey_texture, black_texture, error_texture) = init_textures(&mut device, &mut allocator, &mut command_control);
        
        let pixelated_sampler = Sampler::create(&mut device, vk::Filter::NEAREST).unwrap();
        let fuzzy_sampler = Sampler::create(&mut device, vk::Filter::LINEAR).unwrap();
        
        let materials = materials::init_material(&mut device, &mut allocator, &canvas, &mut ds_pool, &mut destruction_stack, &gpu_scene_layout, &white_texture, &fuzzy_sampler).unwrap();
        
        /*
        let Materials{
            metalic,
            metalic_instance
        } = materials;
        std::mem::drop(metalic_instance);
        metalic.destruct(VkDestructorArguments::Dev(&mut device));
        */
        
        
        let (render_image, depth_image) = canvas.get_images();
        let mesh_assets = load_gltf(&mut device, &mut allocator, &mut command_control, "res/gltf/basicmesh.glb").expect("runtime error");
        
        let main_draw_context = DrawContext::default();
        
        
        
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
            
            canvas: VkWrapper::new(canvas),
            
            texture_descriptor_layout: VkWrapper::new(texture_descriptor_layout),
            background_image_descriptor_layout: VkWrapper::new(background_image_descriptor_layout),
            ds_pool: VkWrapper::new(ds_pool),
            background_image_ds: background_image_ds,
            
            compute_effects: VkWrapper::new(compute_effects),
            compute_effect_index:0,
            
            //mesh_pipeline: VkWrapper::new(mesh_pipeline),
            mesh_assets: VkWrapper::new(mesh_assets),
            main_draw_context,
            
            materials: VkWrapper::new(materials),
            
            mesh_index: 0,
            
            field_of_view:na::Vector3::new(10000.0,0.01,70.0),
            downscale_coheficient: 1.0,
            
            frames_data: VkWrapper::new(frames_data),
            
            scene_data: GPUSceneData::default(),
            gpu_scene_layout: VkWrapper::new(gpu_scene_layout),
            
            white_texture: VkWrapper::new(white_texture),
            grey_texture: VkWrapper::new(grey_texture),
            black_texture: VkWrapper::new(black_texture),
            error_texture: VkWrapper::new(error_texture),
            
            pixelated_sampler: VkWrapper::new(pixelated_sampler),
            fuzzy_sampler: VkWrapper::new(fuzzy_sampler),
            
            destruction_stack: destruction_stack,
        }
        
    }
    
    pub fn gui_tick(&mut self, data:&InputData) {
        
        //self.compute_effects.metadatas[data.background_index].data[index] = data.push_constants[index];
        /*
        self.compute_effect_index = data.background_index;
        for index in 0..4 {
            self.compute_effects.metadatas[data.background_index].data[index] = data.push_constants[index];
        }
        */
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
    
    pub fn get_gui_data(
        &mut self
    ) ->  (
        (
            &[ArrayString<64>],
            &[std::rc::Rc<VkMeshAsset>],
        ), (
            &dyn Fn(&ArrayString<64>)->&str,
            &dyn Fn(&std::rc::Rc<VkMeshAsset>)->&str,
        ),(
            &mut usize,
            &mut ComputePushConstants,
            &mut usize, 
            &mut Vector3<f32>,
            &mut f32,
        )
    ) {
        let ComputeEffects{ref names, ref mut push_constants, ..} = *self.compute_effects;
        let index = self.compute_effect_index;
        (
            (names, &self.mesh_assets[..]), 
            (&|holder|{holder}, &|holder|{&holder.name}),
            (&mut self.compute_effect_index, &mut push_constants[index], &mut self.mesh_index, &mut self.field_of_view, &mut self.downscale_coheficient, )
        )
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
            
            canvas,
            
            ds_pool, 
            background_image_descriptor_layout, 
            texture_descriptor_layout, 
            compute_effects, 
            mesh_assets,
            
            materials,
            
            frames_data,
            gpu_scene_layout,
            fuzzy_sampler,
            pixelated_sampler,
            
            white_texture,
            grey_texture,
            black_texture,
            error_texture,
            destruction_stack,
            ..
        } = self;
        
        let dev = device;
        let all = allocator;
        
        destruction_stack.dispatch(dev, all);
        
        fuzzy_sampler.destruct(VkDestructorArguments::Dev(dev));
        pixelated_sampler.destruct(VkDestructorArguments::Dev(dev));
        
        white_texture.destruct(VkDestructorArguments::DevAll(dev, all));
        grey_texture.destruct(VkDestructorArguments::DevAll(dev, all));
        black_texture.destruct(VkDestructorArguments::DevAll(dev, all));
        error_texture.destruct(VkDestructorArguments::DevAll(dev, all));
        
        gpu_scene_layout.destruct(VkDestructorArguments::Dev(dev));
        frames_data.destruct(VkDestructorArguments::DevAll(dev, all));
        
        
        mesh_assets.destruct(VkDestructorArguments::DevAll(dev, all));
        
        materials.destruct(VkDestructorArguments::Dev(dev));
        
        //mesh_assets.destruct(VkDestructorArguments::DevAll(dev, all));
        //mesh_pipeline.destruct(VkDestructorArguments::Dev(dev));
        compute_effects.destruct(VkDestructorArguments::Dev(dev));
        
        
        ds_pool.destruct(VkDestructorArguments::Dev(dev));
        background_image_descriptor_layout.destruct(VkDestructorArguments::Dev(dev));
        texture_descriptor_layout.destruct(VkDestructorArguments::Dev(dev));
        
        
        /*
        render_image.destruct(VkDestructorArguments::DevAll(dev, all));
        depth_image.destruct(VkDestructorArguments::DevAll(dev, all));
        */
        command_control.destruct(VkDestructorArguments::Dev(dev));
        
        canvas.destruct(VkDestructorArguments::DevAll(dev, all));
        
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



        /*
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
        */
            
