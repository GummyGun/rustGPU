use crate::AAError;
use crate::constants;
use crate::errors::messages::SIMPLE_VK_FN;


use crate::logger;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::device::Device;
use super::p_device::PDevice;

use std::slice::from_ref;

use ash::vk;

pub struct CommandControl{
    pub pool: vk::CommandPool,
    s_u_buffer: vk::CommandBuffer,
}


impl CommandControl {
    pub fn create(p_device:&PDevice, device:&mut Device) -> Result<Self, AAError> {
        use constants::fif;
        
        logger::create!("command_control");
        
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(p_device.queues.graphics_family);
        
        let command_pool = unsafe{device.create_command_pool(&create_info, None)}?;
        
        let sb_create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        
        let s_u_buffer = unsafe{device.allocate_command_buffers(&sb_create_info)}?;
        
        Ok(Self{
            pool: command_pool,
            s_u_buffer: s_u_buffer[0],
        })
    }
    
    pub fn setup_su_buffer(&self, device:&Device) -> vk::CommandBuffer {
        
        unsafe{device.reset_command_buffer(self.s_u_buffer, vk::CommandBufferResetFlags::empty())}.expect(SIMPLE_VK_FN);
        
        let begin_info = ash::vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        unsafe{device.begin_command_buffer(self.s_u_buffer, &begin_info)}.expect(SIMPLE_VK_FN);
        
        self.s_u_buffer
    }
    
    pub fn submit_su_buffer(&self, device:&Device) {
        
        unsafe{device.end_command_buffer(self.s_u_buffer)}.expect(SIMPLE_VK_FN);
        
        let submit_info = [
            vk::SubmitInfo::builder()
                .command_buffers(from_ref(&self.s_u_buffer))
                .build(),
        ];
        
        unsafe{device.queue_submit(device.queue_handles.graphics, &submit_info[..], vk::Fence::null())}.expect(SIMPLE_VK_FN);
        unsafe{device.device_wait_idle()}.expect(SIMPLE_VK_FN);
    }
    
}

impl VkDestructor for CommandControl {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("command_control");
        let device = args.unwrap_dev();
        unsafe{device.destroy_command_pool(self.pool, None)};
    }
}


