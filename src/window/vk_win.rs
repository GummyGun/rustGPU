use ash::vk;


use super::{
    Window,
};

use crate::{
    vulkan,
};

impl Window {
    pub unsafe fn create_surface(&self, instance:&vulkan::Instance, allocator:Option<&vk::AllocationCallbacks>) -> ash::prelude::VkResult<vk::SurfaceKHR> {
        use ash::RawPtr;
        let mut holder:vk::SurfaceKHR = Default::default();
        let instance = instance.underlying().handle();
        let holder_ptr = &mut holder as *mut _;
        
        self.create_window_surface(instance, allocator.as_raw_ptr(), holder_ptr).result()?;
        Ok(holder)
    }
    
}
    
