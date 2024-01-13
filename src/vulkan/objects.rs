use super::Device;
use super::Allocator;
use crate::State;

use std::{
    ops::Deref,
    ops::DerefMut,
};

const DROPED_ERR_TEXT:&'static str = "can't run methods on destroyed objects";
const NON_DEV_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use device_drop";
const NON_ACT_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use active_drop";


pub enum DestructorArguments<'a> {
    None,
    Dev(&'a Device),
    DevAll(&'a Device,&'a Allocator),
}

pub enum DestructorType {
    None,
    Dev,
    DevAll,
}

pub trait VkDestructor {
    fn destruct(&mut self, args:DestructorArguments);
}

pub trait VkDeferedDestructor {
    fn defered_destructor() -> (Box<dyn FnOnce(DestructorArguments)>, DestructorType);
}


/* 
dynamic dispached 
should implement 
*/


pub struct VkWraper<T:VkDestructor>(Option<T>);

impl<T:VkDestructor> VkWraper<T> {
    pub fn new(new:T) -> Self {
        Self(Some(new))
    }
}


impl<T:VkDestructor> Drop for VkWraper<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {eprintln!("{}", NON_ACT_DROPED_ERR_TEXT)}
            None => {}
        }
    }
}

impl<T:VkDestructor> VkDestructor for VkWraper<T> {
    fn destruct(&mut self, args:DestructorArguments) {
        self.0.as_mut().expect(DROPED_ERR_TEXT).destruct(args);
        self.0 = None;
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













pub trait DeviceDestroy {
    fn device_destroy(&mut self, state:&State, device:&Device);
}


pub trait ActiveDestroy {
    fn active_drop(&mut self, state:&State);
}


pub struct VkObjDevDep<T:DeviceDestroy>(Option<T>);


impl<T:DeviceDestroy> Drop for VkObjDevDep<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {eprintln!("{}", NON_DEV_DROPED_ERR_TEXT)}
            None => {}
        }
    }
}

impl<T:DeviceDestroy> DeviceDestroy for VkObjDevDep<T> {
    fn device_destroy(&mut self, state:&State, device:&Device) {
        self.0.as_mut().unwrap().device_destroy(state, device);
        self.0 = None;
    }
}

impl<T:DeviceDestroy> Deref for VkObjDevDep<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect(DROPED_ERR_TEXT)
    }
}

impl<T:DeviceDestroy> DerefMut for VkObjDevDep<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect(DROPED_ERR_TEXT)
    }
}

impl<T:DeviceDestroy> VkObjDevDep<T> {
    pub fn new(new:T) -> Self {
        Self(Some(new))
    }
}

