use crate::logger;
use crate::errors::messages::LEAKING_OBJECTS;

use super::VkDynamicDestructor;
use super::VkDestructorType;
use super::VkDestructorArguments;

use super::super::Device;
use super::super::Allocator;

use std::collections::VecDeque;

#[derive(Default)]
pub struct DestructionStack {
    order: VecDeque<VkDynamicDestructor>,
}

impl DestructionStack {
//----
    pub fn new() -> Self {
        Self::default()
    }
    
//----
    pub fn push(&mut self, destructor:VkDynamicDestructor) {
        self.order.push_front(destructor);
    }
    
//----
    pub fn dispatch(&mut self, device:&mut Device, allocator:&mut Allocator) {
        
        match self.order.len() {
            1 => {
                logger::various_log!("destruction stack",
                    (logger::Trace, "running: 1 defered destructor")
                );
            }
            size => {
                logger::various_log!("destruction stack",
                    (logger::Trace, "running: {} defered destructors", size)
                );
            }
        }
        
        for (destructor, d_type) in self.order.drain(..) {
            match d_type {
                VkDestructorType::None => {
                    destructor(VkDestructorArguments::None);
                }
                VkDestructorType::Dev => {
                    destructor(VkDestructorArguments::Dev(device));
                }
                VkDestructorType::DevAll => {
                    destructor(VkDestructorArguments::DevAll(device, allocator));
                }
            }
        }
    }
    
//----
    pub fn destruct(&mut self, mut args:VkDestructorArguments) {
        let (device, allocator) = args.unwrap_dev_all();
        self.dispatch(device, allocator);
    }
}


impl Drop for DestructionStack {
    fn drop(&mut self) {
        match self.order.len() {
            0 => {}
            1 => {
                logger::various_log!("destruction stack",
                    (logger::Error, "{} object is leaking", self.order.len())
                );
                panic!("{}", LEAKING_OBJECTS);
            }
            size => {
                logger::various_log!("destruction stack",
                    (logger::Error, "{} objects are leaking", size)
                );
                panic!("{}", LEAKING_OBJECTS);
                
            }
            
        }
    }
}

