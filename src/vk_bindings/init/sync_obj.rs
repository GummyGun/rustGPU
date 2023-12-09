use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    DeviceDrop,
    Device,
};

use crate::{
    State
};


pub struct SyncObjects {
    pub image_available_semaphore: [vk::Semaphore; 1],
    pub render_finished_semaphore: [vk::Semaphore; 1],
    pub in_flight_fence: [vk::Fence; 1],
}

impl SyncObjects {
    pub fn create(state:&State, device:&Device) -> VkResult<Self> {
        if  state.v_exp() {
            println!("\nCREATING:\tSYNC OBJECTS");
        }
        
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
        let image_available_semaphore = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
        let render_finished_semaphore = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
        let in_flight_fence =  unsafe{device.create_fence(&fence_create_info, None)}?;
        
        Ok(Self{
            image_available_semaphore: [image_available_semaphore],
            render_finished_semaphore: [render_finished_semaphore],
            in_flight_fence: [in_flight_fence],
        })
    }
}

impl DeviceDrop for SyncObjects {
    fn device_drop(&mut self, state:&State, device:&Device) {
        if state.v_nor() {
            println!("[0]:deleting semaphores");
        }
        unsafe{device.destroy_semaphore(self.image_available_semaphore[0], None)};
        unsafe{device.destroy_semaphore(self.render_finished_semaphore[0], None)};
        if state.v_nor() {
            println!("[0]:deleting fence");
        }
        unsafe{device.destroy_fence(self.in_flight_fence[0], None)};
        
    }
}
