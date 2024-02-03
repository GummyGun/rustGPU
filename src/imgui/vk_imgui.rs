use crate::logger;

use crate::window::Window;
use crate::vulkan::VInit;
use crate::vulkan::Device;
use crate::vulkan::Swapchain;
use crate::vulkan::memory::Allocator;
use crate::vulkan::vk_create_interpreter;
use crate::vulkan::pipeline;

use super::InputData;

use std::sync::Arc;
use std::sync::Mutex;
use std::mem::ManuallyDrop;

use ash::vk;
use imgui::Context;
use imgui_sdl2::ImguiSdl2;
use imgui_rend::Options;
use imgui_rend::Renderer;
use imgui_rend::DynamicRendering;
use imgui_rs_vulkan_renderer as imgui_rend;
use gpu_allocator::vulkan as gpu_vk;


#[allow(dead_code)]
pub struct Imgui{
    allocator: Arc<Mutex<gpu_vk::Allocator>>,
    pub platform: ImguiSdl2,
    pub context: Context,
    pub renderer: ManuallyDrop<Renderer>,
    pub ui_data: InputData,
}

impl Imgui {
    pub fn init(window:&mut Window, v_init:&mut VInit) -> Self {
        let VInit{
            instance,
            p_device,
            device,
            swapchain,
            command_control,
            ..
        } = v_init;
        
        let imgui_allocator = vk_create_interpreter(Allocator::create(instance, &p_device, device), "allocator").into_inner();
        Self::create(window, device, swapchain, &command_control.pool, imgui_allocator)
    }
    
    pub fn create(
        window: &Window,
        device: &mut Device,
        swapchain: &Swapchain,
        command_pool: &vk::CommandPool,
        allocator: gpu_vk::Allocator,
        
    ) -> Self {
        logger::create!("imgui");
        
        let graphics_queue: vk::Queue = device.queue_handles.graphics;
        
        let mut context = imgui::Context::create();
        
        let options_arg = Some(Options{
            in_flight_frames:2,
            enable_depth_test:false,
            enable_depth_write:false,
        });
        
        let dynamic_info = DynamicRendering{
            color_attachment_format: swapchain.surface_format.format,
            depth_attachment_format: None,
        };
        
        let platform = imgui_sdl2::ImguiSdl2::new(&mut context, window.underlying());
        let allocator = Arc::new(Mutex::new(allocator));
        let renderer = Renderer::with_gpu_allocator(allocator.clone(), device.underlying(), graphics_queue, command_pool.clone(), dynamic_info, &mut context, options_arg).unwrap();
        
        Self{
            allocator: allocator,
            context,
            platform,
            renderer: ManuallyDrop::new(renderer),
            ui_data: InputData::default(),
        }
    }
    
    
    pub fn get_common_mut(&mut self) -> (&mut Context, &mut ImguiSdl2, &mut InputData) {
        (&mut self.context, &mut self.platform, &mut self.ui_data)
    }
    
    pub fn get_ui_data(&self) -> &InputData {
        &self.ui_data
    }
    
    pub fn render(
        &mut self,
        device: &Device,
        cmd: vk::CommandBuffer,
        extent: vk::Extent2D,
        view: vk::ImageView,
        
    ) {
        
        let draw_data = self.context.render();
        
        let color_attachment_info = pipeline::rendering_attachment_info(view, None, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let rendering_info = pipeline::rendering_info(extent, &color_attachment_info, None);
        
        unsafe{device.cmd_begin_rendering(cmd, &rendering_info)};
        self.renderer.cmd_draw(cmd, draw_data).unwrap();
        unsafe{device.cmd_end_rendering(cmd)};
        
    }
    
}

impl Drop for Imgui {
    fn drop(&mut self) {
        logger::destruct!("imgui");
        unsafe{ManuallyDrop::drop(&mut self.renderer)};
    }
}

