mod init;

use super::{
    window::{
        Window,
    },
    constants,
    State,
    Verbosity,
};

use init::{
    instance::*,
    d_messenger::*,
    p_device::*,
};

use std::{
    mem::ManuallyDrop,
};

pub struct VInit {
    state: State,
    pub instance: ManuallyDrop<Instance>,
    pub messenger: Option<ManuallyDrop<DMessenger>>,
}

impl VInit {
    pub fn init(state:State, window:&Window) -> VInit {
        let instance = match Instance::create(&state, window) {
            Ok(instance) => {
                if let Verbosity::Expresive | Verbosity::Normal = state.verbosity {
                    println!("[0]:instance");
                }
                instance
            }
            Err(err) => {panic!("{:?}", err);}
        };
        
        let messenger = if constants::VALIDATION {
            Some(match DMessenger::create(&state, &instance) {
                Ok(messenger) => {
                    if let Verbosity::Expresive | Verbosity::Normal = state.verbosity {
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
        
        let _tmp_var = PhysicalDevice::chose(&state, &instance);
        
        VInit{
            state: state,
            instance: ManuallyDrop::new(instance),
            messenger: match messenger {
                Some(holder) => {Some(ManuallyDrop::new(holder))}
                None => None
            }
        }
    }
}


impl Drop for VInit {
    fn drop(&mut self) {
        match &mut self.messenger {
            Some(ref mut messenger) => {
                if let Verbosity::Expresive | Verbosity::Normal = self.state.verbosity {
                    println!("deleting Messenger");
                }
                unsafe{ManuallyDrop::drop(messenger);}
            }
            None => {
                if let Verbosity::Expresive | Verbosity::Normal = self.state.verbosity {
                    println!("No Messenger to delete");
                }
            }
        }
        
        
        if let Verbosity::Expresive | Verbosity::Normal = self.state.verbosity {
            println!("deleting Instance");
        }
        unsafe{ManuallyDrop::drop(&mut self.instance);}
    }
}

