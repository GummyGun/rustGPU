use crate::logger;

use crate::errors::messages::NON_DESTROYED;
use crate::errors::messages::ALREADY_DESTROYED;
use crate::errors::messages::REDUNDANT_DEREFED_DESTRUCTOR;
use crate::errors::messages::REDUNDANT_DESTRUCTOR;

use super::VkDestructorType;
use super::VkDeferedDestructor;
use super::VkDestructorArguments;

use std::ops::Deref;
use std::ops::DerefMut;
use std::mem::replace;


#[derive(Debug)]
enum VkHolder<T:VkDeferedDestructor> {
    None,
    Some(T),
    Defered(T),
}

#[derive(Debug)]
pub struct VkDeferedWrapper<T:VkDeferedDestructor>(VkHolder<T>);

impl<T:VkDeferedDestructor> VkDeferedWrapper<T> {
//----
    #[allow(dead_code)]
    pub fn new(new:T) -> Self {
        Self(VkHolder::Some(new))
    }
    
//----
    #[allow(dead_code)]
    pub fn destruct(&mut self, args:VkDestructorArguments) {
        self.0.destruct(args)
    }
    
//----
    #[allow(dead_code)]
    pub fn defered_destruct(&mut self)  -> (Box<dyn FnOnce(VkDestructorArguments)>, VkDestructorType) {
        self.0.defered_destruct()
    }
    
}

impl <T:VkDeferedDestructor> VkHolder<T> {
    
//----
    fn take(&mut self) -> Self {
        replace(self, VkHolder::None)
    }
    
//----
    fn destruct(&mut self, args:VkDestructorArguments) {
        match self.take() {
            VkHolder::None => {
                logger::various_log!("defered wrapper",
                    (logger::Error, "{}", ALREADY_DESTROYED)
                );
                panic!("{}", ALREADY_DESTROYED);
            }
            VkHolder::Defered(_object) => {
                logger::various_log!("defered wrapper",
                    (logger::Error, "{}", REDUNDANT_DESTRUCTOR)
                );
                panic!("{}", REDUNDANT_DESTRUCTOR);
            }
            VkHolder::Some(data) => {
                data.destruct(args);
                
            }
        }
    }
    
//----
    fn defered_destruct(&mut self)  -> (Box<dyn FnOnce(VkDestructorArguments)>, VkDestructorType) {
        let mut object = match self.take() {
            VkHolder::None => {
                logger::various_log!("defered wrapper",
                    (logger::Error, "{}", REDUNDANT_DEREFED_DESTRUCTOR)
                );
                panic!("{}", ALREADY_DESTROYED);
            }
            VkHolder::Defered(_object) => {
                logger::various_log!("defered wrapper",
                    (logger::Error, "{}", REDUNDANT_DEREFED_DESTRUCTOR)
                );
                panic!("{}", REDUNDANT_DEREFED_DESTRUCTOR);
            }
            VkHolder::Some(data) => {
                data
            }
        };
        let holder = object.defered_destruct();
        *self = VkHolder::Defered(object);
        holder
    }
    
//----
    fn get_inner(&self) -> Option<&T> {
        match self {
            VkHolder::None => {
                None
            }
            VkHolder::Some(ref data) | VkHolder::Defered(ref data) => {
                Some(data)
            }
        }
    }
    
//----
    fn get_inner_mut(&mut self) -> Option<&mut T> {
        match self {
            VkHolder::None => {
                None
            }
            VkHolder::Some(ref mut data) | VkHolder::Defered(ref mut data) => {
                Some(data)
            }
        }
    }
    
}

impl<T:VkDeferedDestructor> Deref for VkDeferedWrapper<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        self.0.get_inner().expect(ALREADY_DESTROYED)
    }
}


impl<T:VkDeferedDestructor> DerefMut for VkDeferedWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.get_inner_mut().expect(ALREADY_DESTROYED)
    }
}


impl<T:VkDeferedDestructor> Drop for VkDeferedWrapper<T> {
    fn drop(&mut self) {
        match self.0 {
            VkHolder::Some(_) => {
                logger::various_log!("defered wrapper",
                    (logger::Error, "{}", NON_DESTROYED)
                );
                panic!("{}", NON_DESTROYED);
            }
            _ => {}
        }
    }
}


