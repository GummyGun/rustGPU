mod init;
mod objects;


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
    swapchain::*,
};

use objects::{
    VkObj,
    VkObjDevDep,
    DeviceDrop,
    ActiveDrop,
};

pub struct VInit {
    state: State,
    pub instance: VkObj<Instance>,
    pub messenger: Option<VkObj<DMessenger>>,
    pub surface: VkObj<Surface>,
    pub p_device: PhysicalDevice,
    pub device: VkObj<Device>,
    pub swapchain: VkObjDevDep<Swapchain>,
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
        let swapchain = vk_create_interpreter(state, Swapchain::create(&state, &window, &instance, &surface, &p_device, &device), "swapchain");
        
        VInit{
            state: state,
            instance: VkObj::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(VkObj::new(holder))}
                None => None
            },
            p_device: p_device,
            surface: VkObj::new(surface),
            device: VkObj::new(device),
            swapchain: VkObjDevDep::new(swapchain),
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
        Err(err) => {panic!("error in {} {:?}", name, err);}
    }
}

/*
#[inline]
fn vk_drop<T>(state:&State, vk_object:&mut ManuallyDrop<T>, name:&'static str) {
    if state.v_nor() {
        println!("deleting {}", name);
    }
    unsafe{ManuallyDrop::drop(vk_object);}
}

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
        
        self.swapchain.device_drop(&self.state, &self.device);
        self.device.active_drop(&self.state);
        self.surface.active_drop(&self.state);
        
        match &mut self.messenger {
            Some(ref mut messenger) => {
                messenger.active_drop(&self.state);
            }
            None => {
                if self.state.v_nor() {
                    println!("No Messenger to delete");
                }
            }
        }
        
        self.instance.active_drop(&self.state);
    }
}

