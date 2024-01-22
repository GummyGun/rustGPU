use crate::window::Window;

use super::logger::imgui as logger;
use super::VkDestructor;
use super::VkDestructorArguments;
use super::Instance;
use super::Device;
use super::Allocator;
use super::Swapchain;
use super::PDevice;
use super::image;

use std::sync::Arc;
use std::sync::Mutex;
use std::mem::ManuallyDrop;

use ash::vk;
use imgui::Context;
use imgui::Condition;
use imgui_sdl2::ImguiSdl2;
use imgui_rend::Options;
use imgui_rend::Renderer;
use imgui_rend::DynamicRendering;
use imgui_rs_vulkan_renderer as imgui_rend;
use gpu_allocator::vulkan as gpu_vk;

pub struct Imgui {
    allocator: Arc<Mutex<gpu_vk::Allocator>>,
    pub platform: ImguiSdl2,
    pub context: Context,
    pub renderer: ManuallyDrop<Renderer>,
}

impl Imgui {
    pub fn create(
        window: &Window,
        instance: &mut Instance,
        p_device: &PDevice,
        device: &mut Device,
        swapchain: &Swapchain,
        command_pool: &vk::CommandPool,
        allocator: gpu_vk::Allocator,
        
    ) -> Self {
        logger::create();
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
        
        let mut platform = imgui_sdl2::ImguiSdl2::new(&mut context, window.underlying());
        let allocator = Arc::new(Mutex::new(allocator));
        let renderer = Renderer::with_gpu_allocator(allocator.clone(), device.underlying(), graphics_queue, command_pool.clone(), dynamic_info, &mut context, options_arg).unwrap();
        
        Self{
            allocator: allocator,
            context,
            platform,
            renderer: ManuallyDrop::new(renderer),
        }
    }
    
    pub fn handle_event(
        &mut self,
        window: &Window,
    ) {
        self.platform.prepare_frame(self.context.io_mut(), window.underlying(), &window.event_pump().mouse_state());
    }
    
    fn rendering_attachment_info(
        view: vk::ImageView,
        clear_value: Option<vk::ClearValue>,
        layout: vk::ImageLayout,
        
    ) -> vk::RenderingAttachmentInfo {
        let mut holder = ash::vk::RenderingAttachmentInfo::default();
        holder.image_view = view;
        holder.image_layout = layout;
        holder.store_op = vk::AttachmentStoreOp::STORE;
        match clear_value{
            Some(clear) => {
                holder.load_op = vk::AttachmentLoadOp::CLEAR;
                holder.clear_value = clear;
            }
            None => {
                holder.load_op = vk::AttachmentLoadOp::LOAD;
            }
        }
        holder
    }
    
    fn rendering_info(
        extent: vk::Extent2D,
        color_attachment: &vk::RenderingAttachmentInfo,
        depth_attachment: Option<&vk::RenderingAttachmentInfo>,
    ) -> vk::RenderingInfo {
        let holder = vk::RenderingInfo::builder()
            .render_area(vk::Rect2D::from(extent))
            .layer_count(1)
            .color_attachments(std::slice::from_ref(color_attachment));
        holder.build()
    }

    pub fn render(
        &mut self,
        window: &Window,
        device: &Device,
        cmd: vk::CommandBuffer,
        extent: vk::Extent2D,
        view: vk::ImageView,
    ) {
        
        let ui = self.context.frame();
        ui.show_demo_window(&mut true);
        /*
        ui.window("Hello world")
            .position([20.0, 20.0], Condition::Appearing)
            .size([700.0, 80.0], Condition::Appearing)
            .build(|| {
                ui.text_wrapped("Hello world!");
            });
        */
        
        self.platform.prepare_render(&ui, window.underlying());
        
        let draw_data = self.context.render();
        
        let color_attachment_info = Self::rendering_attachment_info(view, None, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let rendering_info = Self::rendering_info(extent, &color_attachment_info, None);
        
        unsafe{device.cmd_begin_rendering(cmd, &rendering_info)};
        self.renderer.cmd_draw(cmd, draw_data).unwrap();
        unsafe{device.cmd_end_rendering(cmd)};
        
    }
}

impl VkDestructor for Imgui {
    fn destruct(mut self, mut args:VkDestructorArguments) {
        logger::destruct();
        let Imgui{
            allocator,
            mut renderer,
            ..
        } = self;
        let _device = args.unwrap_dev();
        unsafe{ManuallyDrop::drop(&mut renderer)};
    }
    
}

