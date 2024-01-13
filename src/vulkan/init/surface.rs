use crate::State;
use crate::constants;
use crate::window::Window;
use crate::AAError;


use super::logger::surface as logger;
use super::VkDestructor;
use super::DestructorArguments;
use super::instance::Instance;


use std::ops::Deref;


use ash::vk;


pub struct Surface {
    pub surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
}

impl Surface {
    pub fn create(state:&State, window:&Window, instance:&Instance) -> Result<Self, AAError> {
        if state.v_exp() {
            println!("\nCREATING:\tSURFACE");
        }
        
        if state.v_exp() {
            println!("{:?}", constants::EXTENSIONS);
            println!("{:?}", constants::LAYERS);
        }
        
        let surface = unsafe{window.create_surface(instance, None)?};
        let surface_loader = ash::extensions::khr::Surface::new(&instance.entry, instance);
        
        Ok(Self{
            surface:surface,
            surface_loader:surface_loader,
        })
    }
    
}


impl Deref for Surface {
    type Target = ash::extensions::khr::Surface;
    fn deref(&self) -> &Self::Target {
        &self.surface_loader
    }
}

impl VkDestructor for Surface {
    fn destruct(&mut self, _:DestructorArguments) {
        logger::destructor();
        unsafe{self.destroy_surface(self.surface, None)}
    }
}

