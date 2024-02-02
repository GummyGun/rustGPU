use crate::window::Window;
use crate::macros;
use crate::AAError;


use super::logger::surface as logger;
use super::VkDestructor;
use super::VkDestructorArguments;
use super::instance::Instance;


use ash::vk;

pub struct Surface {
    pub surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
}
macros::impl_deref!(Surface, ash::extensions::khr::Surface, surface_loader);

impl Surface {
    pub fn create(window:&Window, instance:&mut Instance) -> Result<Self, AAError> {
        logger::create();
        
        let surface = unsafe{window.create_surface(instance)}.unwrap();
        let surface_loader = ash::extensions::khr::Surface::new(&instance.entry, instance);
        
        Ok(Self{
            surface:surface,
            surface_loader:surface_loader,
        })
    }
    
}

impl VkDestructor for Surface {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct();
        args.unwrap_none();
        unsafe{self.destroy_surface(self.surface, None)}
    }
}

