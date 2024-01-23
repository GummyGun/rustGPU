use ash::{
    vk,
    prelude::VkResult,
};

use super::logger::sync_objs as logger;

use super::VkDestructor;
use super::VkDestructorArguments;
use super::Device;

use crate::{
    State,
    constants,
};


pub struct SyncObjects {
    pub image_available_semaphore: [vk::Semaphore; constants::fif::USIZE],
    pub render_finished_semaphore: [vk::Semaphore; constants::fif::USIZE],
    pub inflight_fence: [vk::Fence; constants::fif::USIZE],
}

impl SyncObjects {
    pub fn create(state:&State, device:&Device) -> VkResult<Self> {
        use constants::fif;
        
        if state.v_exp() {
            println!("\nCREATING:\tSYNC OBJECTS");
        }
        
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);
        
        let mut image_available_semaphore = [vk::Semaphore::null(); fif::USIZE];
        let mut render_finished_semaphore = [vk::Semaphore::null(); fif::USIZE];
        let mut inflight_fence = [vk::Fence::null(); fif::USIZE];
        
        for index in 0..fif::USIZE {
            if state.v_exp() {
                println!("creating sync objects for frame {}", index);
            }
            image_available_semaphore[index] = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
            render_finished_semaphore[index] = unsafe{device.create_semaphore(&semaphore_create_info, None)}?;
            inflight_fence[index] =  unsafe{device.create_fence(&fence_create_info, None)}?;
        }
        
        Ok(Self{
            image_available_semaphore: image_available_semaphore,
            render_finished_semaphore: render_finished_semaphore,
            inflight_fence: inflight_fence,
        })
    }
    
    pub fn get_frame(
        &self,
        frame: usize,
    ) -> (vk::Semaphore, vk::Semaphore, vk::Fence) {
        (self.image_available_semaphore[frame], self.render_finished_semaphore[frame], self.inflight_fence[frame])
        // pub image_available_semaphore: [vk::Semaphore; constants::fif::USIZE],
    }
}


impl VkDestructor for SyncObjects {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct();
        use constants::fif;
        let device = args.unwrap_dev();
        for index in 0..fif::USIZE {
            unsafe{device.destroy_semaphore(self.image_available_semaphore[index], None)};
            unsafe{device.destroy_semaphore(self.render_finished_semaphore[index], None)};
            unsafe{device.destroy_fence(self.inflight_fence[index], None)};
        }
    }
}
/*
impl DeviceDestroy for SyncObjects {
    fn device_destroy(&mut self, state:&State, device:&Device) {
        use constants::fif;
        if state.v_nor() {
        }
        
        for index in 0..fif::USIZE {
            unsafe{device.destroy_semaphore(self.image_available_semaphore[index], None)};
            unsafe{device.destroy_semaphore(self.render_finished_semaphore[index], None)};
            unsafe{device.destroy_fence(self.inflight_fence[index], None)};
        }
        
    }
}
*/
