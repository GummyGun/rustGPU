use crate::window::Window;

use super::logger::imgui as logger;
use super::VkDestructor;
use super::VkDestructorArguments;
use super::Device;
use super::Swapchain;
use super::pipeline;

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
pub struct Imgui {
    allocator: Arc<Mutex<gpu_vk::Allocator>>,
    pub platform: ImguiSdl2,
    pub context: Context,
    pub renderer: ManuallyDrop<Renderer>,
}

impl Imgui {
    pub fn create(
        window: &Window,
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
        
        let platform = imgui_sdl2::ImguiSdl2::new(&mut context, window.underlying());
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
    
    pub fn render(
        &mut self,
        window: &Window,
        device: &Device,
        cmd: vk::CommandBuffer,
        extent: vk::Extent2D,
        view: vk::ImageView,
        floatHolder: &mut [f32;2],
    ) {
        
        let ui = self.context.new_frame();
        {
            let a = ui.begin_disabled(false);
            ui.text("Dangerous button");
            let b = ui.input_float2("test", floatHolder)
                .enter_returns_true(true)
                .build();
            ui.text("Dangerous button");
            
            if b {
                //print!("{:?}", holder);
                print!("Enter\n");
            }
            
            //ui.end_frame_early();
        }
        
        //ui.show_demo_window(&mut true);
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
        
        let color_attachment_info = pipeline::rendering_attachment_info(view, None, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let rendering_info = pipeline::rendering_info(extent, &color_attachment_info, None);
        
        unsafe{device.cmd_begin_rendering(cmd, &rendering_info)};
        self.renderer.cmd_draw(cmd, draw_data).unwrap();
        unsafe{device.cmd_end_rendering(cmd)};
        
    }
}

impl VkDestructor for Imgui {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct();
        let Imgui{
            mut renderer,
            ..
        } = self;
        let _device = args.unwrap_dev();
        unsafe{ManuallyDrop::drop(&mut renderer)};
    }
}

