use super::Device;
use crate::State;

use std::{
    ops::Deref,
    ops::DerefMut,
};

const DROPED_ERR_TEXT:&'static str = "can't run methods on destroyed objects";
const NON_DEV_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use device_drop";
const NON_ACT_DROPED_ERR_TEXT:&'static str = "dropping non-destroyed object use active_drop";

pub trait DeviceDestroy {
    fn device_drop(&mut self, state:&State, device:&Device);
}

pub trait ActiveDestroy {
    fn active_drop(&mut self, state:&State);
}

pub struct VkObj<T:ActiveDestroy>(Option<T>);
pub struct VkObjDevDep<T:DeviceDestroy>(Option<T>);


impl<T:ActiveDestroy> Drop for VkObj<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {eprintln!("{}", NON_ACT_DROPED_ERR_TEXT)}
            None => {}
        }
    }
}

impl<T:ActiveDestroy> ActiveDestroy for VkObj<T> {
    fn active_drop(&mut self, state:&State) {
        self.0.as_mut().expect(DROPED_ERR_TEXT).active_drop(state);
        self.0 = None;
    }
}

impl<T:ActiveDestroy> Deref for VkObj<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0.as_ref().expect(DROPED_ERR_TEXT)
    }
}

impl<T:ActiveDestroy> DerefMut for VkObj<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect(DROPED_ERR_TEXT)
    }
}


impl<T:ActiveDestroy> VkObj<T> {
    pub fn new(new:T) -> Self {
        Self(Some(new))
    }
}


impl<T:DeviceDestroy> Drop for VkObjDevDep<T> {
    fn drop(&mut self) {
        match self.0.as_mut() {
            Some(_) => {eprintln!("{}", NON_DEV_DROPED_ERR_TEXT)}
            None => {}
        }
    }
}

impl<T:DeviceDestroy> DeviceDestroy for VkObjDevDep<T> {
    fn device_drop(&mut self, state:&State, device:&Device) {
        self.0.as_mut().unwrap().device_drop(state, device);
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

