use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    ActiveDestroy,
    instance::Instance,
};

use crate::{
    State,
    constants,
    window::{
        Window,
    },
};

use std::{
    ops::Deref,
};

pub struct Surface {
    pub surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
}

impl Surface {
    pub fn create(state:&State, window:&Window, instance:&Instance) -> VkResult<Self> {
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

impl ActiveDestroy for Surface {
    fn active_drop(&mut self, state:&State) {
        if state.v_nor() {
            println!("[0]:deleting surface");
        }
        unsafe{self.destroy_surface(self.surface, None)}
    }
}
