use crate::errors::messages::BAD_DESTRUCTOR;

use super::Device;
use super::Allocator;

use std::ops::Deref;
use std::ops::DerefMut;

const DROPED_ERR_TEXT:&'static str = "can't run methods on destroyed objects";
//const NON_DEV_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use device_drop";
const NON_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use active_drop";


pub enum VkDestructorArguments<'a> {
    None,
    Dev(&'a mut Device),
    DevAll(&'a mut Device,&'a mut Allocator),
}

#[allow(dead_code)]
pub enum VkDestructorType {
    None,
    Dev,
    DevAll,
}

pub trait VkDestructor {
    fn destruct(self, args:VkDestructorArguments);
}

pub trait VkDeferedDestructor {
    fn defered_destruct() -> (Box<dyn FnOnce(VkDestructorArguments)>, VkDestructorType);
}


pub struct VkWraper<T:VkDestructor>(Option<T>);

impl<T:VkDestructor> VkWraper<T> {
    pub fn new(new:T) -> Self {
        Self(Some(new))
    }
    
    //will this ever be called
    #[allow(dead_code)]
    pub fn destruct(&mut self, args:VkDestructorArguments) {
        self.0.take().expect(DROPED_ERR_TEXT).destruct(args);
    }
    
    pub fn take(&mut self) -> T {
        self.0.take().expect(DROPED_ERR_TEXT)
    }
}


impl<T:VkDestructor> Drop for VkWraper<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {eprintln!("{}", NON_DROPED_ERR_TEXT)}
            None => {}
        }
    }
}

impl<T:VkDestructor> Deref for VkWraper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect(DROPED_ERR_TEXT)
    }
}

impl<T:VkDestructor> DerefMut for VkWraper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect(DROPED_ERR_TEXT)
    }
}



impl VkDestructorArguments<'_> {
    
    pub fn unwrap_none(&mut self) {
        if let VkDestructorArguments::None = self {
        } else {
            panic!("{}", BAD_DESTRUCTOR);
        }
    }
    
    pub fn unwrap_dev(&mut self) -> &Device {
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

