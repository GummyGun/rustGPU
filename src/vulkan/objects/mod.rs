mod wrapper;
pub use wrapper::VkWrapper;

mod d_wrapper;
//pub use d_wrapper::VkDeferedWrapper;

mod d_stack;
pub use d_stack::DestructionStack;


use crate::errors::messages::BAD_DESTRUCTOR;

use super::Device;
use super::Allocator;



pub enum VkDestructorArguments<'a> {
    None,
    Dev(&'a mut Device),
    DevAll(&'a mut Device,&'a mut Allocator),
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum VkDestructorType {
    None,
    Dev,
    DevAll,
}

pub trait VkDestructor {
    fn destruct(self, args:VkDestructorArguments);
}

pub trait VkDeferedDestructor:VkDestructor {
    fn defered_destruct(&mut self) -> (Box<dyn FnOnce(VkDestructorArguments)>, VkDestructorType);
}

pub type VkDynamicDestructor = (Box<dyn FnOnce(VkDestructorArguments)>, VkDestructorType);

impl VkDestructorArguments<'_> {
    
    pub fn unwrap_none(&mut self) {
        if let VkDestructorArguments::None = self {
        } else {
            panic!("{}", BAD_DESTRUCTOR);
        }
    }
    
    pub fn unwrap_dev(&mut self) -> &mut Device {
        if let VkDestructorArguments::Dev(device) = self {
            device
        } else {
            panic!("{}", BAD_DESTRUCTOR);
        }
    }
    
    pub fn unwrap_dev_all(&mut self) -> (&mut Device, &mut Allocator) {
        if let VkDestructorArguments::DevAll(device, allocator) = self {
            (device, allocator)
        } else {
            panic!("{}", BAD_DESTRUCTOR);
        }
    }
    
}

