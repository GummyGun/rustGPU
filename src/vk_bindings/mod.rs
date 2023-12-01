mod init;


use super::{
    window::{
        Window,
    },
    constants,
    State,
};

pub use init::{
    instance::*,
    d_messenger::*,
    p_device::*,
    device::*,
    surface::*,
};

use std::{
    mem::ManuallyDrop,
};

pub struct VInit {
    state: State,
    pub instance: ManuallyDrop<Instance>,
    pub messenger: Option<ManuallyDrop<DMessenger>>,
    pub surface: ManuallyDrop<Surface>,
    pub p_device: PhysicalDevice,
    pub device: ManuallyDrop<Device>,
}

impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        
        let instance = vk_create_interpreter(state, Instance::create(&state, window), "instance"); 
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&state, &instance) {
                Ok(messenger) => {
                    if state.v_nor() {
                        println!("[0]:messenger");
                    }
                    messenger
                }
                Err(err) => {panic!("{:?}", err);}
            })
        } else {
            println!("[X]:messenger");
            None
        };
        
        let surface =  vk_create_interpreter(state, Surface::create(&state, &window, &instance), "surface"); 
        let p_device = vk_create_interpreter(state, PhysicalDevice::chose(&state, &instance, &surface), "p_device selected"); 
        let device = vk_create_interpreter(state, Device::create(&state, &instance, &p_device), "device"); 
        
        VInit{
            state: state,
            instance: ManuallyDrop::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(ManuallyDrop::new(holder))}
                None => None
            },
            p_device: p_device,
            device: ManuallyDrop::new(device),
            surface: ManuallyDrop::new(surface),
        }
    }
}


#[inline]
fn vk_create_interpreter<T, A:std::fmt::Debug>(state:State, result:Result<T, A>, name:&'static str) -> T {
    match result {
        Ok(device) => {
            if state.v_nor() {
                println!("[0]:{}", name);
            }
            device
        }
        Err(err) => {panic!("{:?}", err);}
    }
}

#[inline]
fn vk_drop<T>(state:&State, vk_object:&mut ManuallyDrop<T>, name:&'static str) {
    if state.v_nor() {
        println!("deleting {}", name);
    }
    unsafe{ManuallyDrop::drop(vk_object);}
}

/*
#[macro_export]
macro_rules! vk_drop {
    ( $state:expr, $field:expr, $name:expr ) => {
        {
            println!("{:?} {}", $state, state);
        }
    };
}
*/


impl Drop for VInit {
    fn drop(&mut self) {
        
        vk_drop(&self.state, &mut self.device, "device");
        vk_drop(&self.state, &mut self.surface, "surface");
        match &mut self.messenger {
            Some(ref mut messenger) => {
                if self.state.v_nor() {
                    
                    println!("deleting Messenger");
                }
                unsafe{ManuallyDrop::drop(messenger);}
            }
            None => {
                if self.state.v_nor() {
                    println!("No Messenger to delete");
                }
            }
        }
        
        vk_drop(&self.state, &mut self.instance, "instance");
    }
}

