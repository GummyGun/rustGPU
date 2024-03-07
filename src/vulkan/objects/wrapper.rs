use crate::logger;

use crate::errors::messages::ALREADY_DESTROYED;
use crate::errors::messages::NON_DESTROYED;
use crate::errors::messages::NON_EMPTY_WRAPPER;

use super::VkDestructor;
use super::VkDestructorArguments;

use std::ops::Deref;
use std::ops::DerefMut;


pub struct VkWrapper<T:VkDestructor>(Option<T>);

impl<T:VkDestructor> VkWrapper<T> {
    
//----
    pub fn new(new:T) -> Self {
        Self(Some(new))
    }
    
//----
    pub fn destruct(&mut self, args:VkDestructorArguments) {
        self.0.take().expect(ALREADY_DESTROYED).destruct(args);
    }
    
//----
    pub fn take(&mut self) -> T {
        self.0.take().expect(ALREADY_DESTROYED)
    }
    
//----
    pub fn fill(&mut self, new_value:T) {
        match &mut self.0 {
            Some(_) => {
                logger::various_log!("wrapper",
                    (logger::Error, "{}", NON_EMPTY_WRAPPER)
                );
                panic!("{}", NON_EMPTY_WRAPPER);
            }
            reference @ None => {
                *reference = Some(new_value);
            }
        }
    }
}


impl<T:VkDestructor> Drop for VkWrapper<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {
                logger::various_log!("wrapper",
                    (logger::Error, "{}", NON_DESTROYED)
                );
                panic!("{}", NON_DESTROYED);
            }
            None => {}
        }
    }
}

impl<T:VkDestructor> Deref for VkWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect(NON_DESTROYED)
    }
}

impl<T:VkDestructor> DerefMut for VkWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect(ALREADY_DESTROYED)
    }
}


