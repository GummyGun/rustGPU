use crate::vulkan;
use crate::AAError;
use crate::constants;



use super::Window;

use ash::vk::Handle;
use ash::vk;
use sdl2::VideoSubsystem;

impl Window {
    pub unsafe fn create_surface(&self, instance:&vulkan::Instance, allocator:Option<&vk::AllocationCallbacks>) -> Result<vk::SurfaceKHR, ()> {
        let handle = Handle::as_raw(instance.handle()) as usize;
        let raw_surface = self.window.vulkan_create_surface(handle).unwrap();
        let holder = ash::vk::SurfaceKHR::from_raw(raw_surface);
        Ok(holder)
        /*
        let raw_surface = self.window.vulkan_create_surface(instance.handle().as_raw() as usize).unwrap();
        let surface = ash::vk::SurfaceKHR::from_raw(raw_surface);
        Ok(surface)
        */
        /*
        use ash::RawPtr;
        let mut holder:vk::SurfaceKHR = Default::default();
        let instance = instance.underlying().handle();
        let holder_ptr = &mut holder as *mut _;
        
        self.create_window_surface(instance, allocator.as_raw_ptr(), holder_ptr).result()?;
        Ok(holder)
        */
    }
    
    pub fn create_vulkan_builder(video:&mut VideoSubsystem) -> Result<sdl2::video::Window, ()> {
        video.window("rust-sdl2 demo", constants::WIDTH, constants::HEIGTH)
            .position_centered()
            .vulkan()
            .build().map_err(|_|())
    }
}
    
