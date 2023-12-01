use ash::{
    vk,
    prelude::VkResult,
};


use super::{
    instance::Instance,
};

use std::{
    ops::Deref,
};

use crate::{
    State,
    window::{
        Window,
    },
    //errors::Error as AAError,
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
        let holder = unsafe{window.create_surface(instance, None)?};
        let hola = ash::extensions::khr::Surface::new(&instance.entry, instance);
        
        Ok(Self{
            surface:holder,
            surface_loader:hola,
        })
    }
}

impl Deref for Surface {
    type Target = ash::extensions::khr::Surface;
    fn deref(&self) -> &Self::Target {
        &self.surface_loader
    }
}


impl Drop for Surface {
    fn drop(&mut self) {
        unsafe{self.destroy_surface(self.surface, None)};
    }
}
