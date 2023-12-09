use ash::{
    vk,
    prelude::VkResult,
};

use super::{
    ActiveDrop,
    instance::Instance,
};

use crate::{
    State,
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
        if  state.v_exp() {
            println!("\nCREATING:\tSURFACE");
        }
        let surface = unsafe{window.create_surface(instance, None)?};
        let surface_loader = ash::extensions::khr::Surface::new(&instance.entry, instance);
        
        Ok(Self{
            surface:surface,
            surface_loader:surface_loader,
        })
    }
    
    #[inline(always)]
    pub fn drop_internal(&mut self) {
        unsafe{self.destroy_surface(self.surface, None)}
    }
}


impl Deref for Surface {
    type Target = ash::extensions::khr::Surface;
    fn deref(&self) -> &Self::Target {
        &self.surface_loader
    }
}

impl ActiveDrop for Surface {
    fn active_drop(&mut self, state:&State) {
        if state.v_nor() {
            println!("[0]:deleting surface");
        }
        self.drop_internal()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        self.drop_internal()
    }
}
