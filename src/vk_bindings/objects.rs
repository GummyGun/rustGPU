use super::Device;
use crate::State;

use std::{
    ops::Deref,
    ops::DerefMut,
    mem::ManuallyDrop,
};

pub trait DeviceDrop {
    fn device_drop(&mut self, state:&State, device:&Device);
}

pub trait ActiveDrop {
    fn active_drop(&mut self, state:&State);
}

pub struct VkObj<T:ActiveDrop>(ManuallyDrop<T>);
pub struct VkObjDevDep<T:DeviceDrop>(T);



impl<T:ActiveDrop> Drop for VkObj<T> {
    fn drop(&mut self) {
        //eprintln!("should be active droped");
    }
}

impl<T:ActiveDrop> ActiveDrop for VkObj<T> {
    fn active_drop(&mut self, state:&State) {
        self.0.active_drop(state)
    }
}

impl<T:ActiveDrop> Deref for VkObj<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T:ActiveDrop> DerefMut for VkObj<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T:ActiveDrop> VkObj<T> {
    pub fn new(new:T) -> Self {
        Self(ManuallyDrop::new(new))
    }
}


impl<T:DeviceDrop> Drop for VkObjDevDep<T> {
    fn drop(&mut self) {
        //eprintln!("VkObjDevDep trivially droped");
    }
}

impl<T:DeviceDrop> DeviceDrop for VkObjDevDep<T> {
    fn device_drop(&mut self, state:&State, device:&Device) {
        self.0.device_drop(state, device);
    }
}

impl<T:DeviceDrop> Deref for VkObjDevDep<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T:DeviceDrop> DerefMut for VkObjDevDep<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T:DeviceDrop> VkObjDevDep<T> {
    pub fn new(new:T) -> Self {
        Self(new)
    }
}


