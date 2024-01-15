use super::RenderPass;
use super::Instance;
use super::Device;
use super::Allocator;
use super::PDevice;

use std::sync::Arc;
use std::sync::Mutex;

use ash::vk;
use imgui::Context;
use imgui_rs_vulkan_renderer::Options;
use imgui_rs_vulkan_renderer::Renderer;

pub struct Imgui {
    pub context: Context,
    pub renderer: Renderer,
}

impl Imgui {
    pub fn create(
        instance: &mut Instance,
        p_device: &PDevice,
        device: &mut super::device::Device,
        allocator: gpu_allocator::vulkan::Allocator,
        /*
        queue: Queue,
        command_pool: CommandPool,
        render_pass: RenderPass,
        //imgui: &mut Context,
        //options: Option<Options>
        */
        
    ) -> Self {
        let mut context = imgui::Context::create();
        
        let format = vk::Format::B8G8R8A8_UNORM;
        let render_pass = RenderPass::create(device, format).unwrap();
        
        let instance_arg: &ash::Instance = instance;
        let p_device_arg: vk::PhysicalDevice = p_device.underlying();
        let device_arg: ash::Device = device.underlying();
        
        let queue_arg: vk::Queue = device.queue_handles.graphics;
        
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(p_device.queues.graphics_family);
        
        let command_pool_arg = unsafe{device.create_command_pool(&create_info, None)}.unwrap();
        let render_pass_arg = render_pass.into_inner();
        let context_arg: &mut Context = &mut context;
        let options_arg: Option<Options> = Some(Options{
            in_flight_frames:2,
            enable_depth_test:false,
            enable_depth_write:false,
        });
        
        let allocator = Arc::new(Mutex::new(allocator));
        let renderer = Renderer::with_gpu_allocator(allocator, device_arg, queue_arg, command_pool_arg, render_pass_arg, context_arg, options_arg).unwrap();
        
        Self{
            context,
            renderer,
        }
    }
}

