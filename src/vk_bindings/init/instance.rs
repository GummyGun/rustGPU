use ash::{
    vk,
    prelude::VkResult,
};
use std::{
    ffi::{
        c_char,
        CStr,
    },
    ops::Deref,
    collections::HashSet,
};

use super::{
    d_messenger::DMessenger
};

use crate::{
    constants,
    State,
    window::{
        Window,
    },
    errors::Error as AAError,
};


pub struct Instance {
    pub entry: ash::Entry,
    instance: ash::Instance,
}

impl Instance {
    /*
    TODO: change strings to CStrings
    */
    
    pub fn create(state:&State, window: &Window) -> VkResult<Instance> {
        if  state.v_exp() {
            println!("\nCREATING:\tINSTANCE");
        }
        let entry = unsafe {ash::Entry::load().unwrap()};
        
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CStr::from_bytes_with_nul(b"Hello Triangle\0").unwrap())
            .application_name(CStr::from_bytes_with_nul(b"AdAstra\0").unwrap())
            .application_version(vk::make_api_version(1, 0, 0, 0))
            .engine_version(vk::make_api_version(1, 0, 0, 0))
            .api_version(vk::API_VERSION_1_0);
        
        let av_extensions = Extensions::gets(&entry);
        av_extensions.debug_print(state);
        let extensions_ptr = av_extensions.handle_logic(state, window);
        
        
        let av_layers = Layers::gets(&entry);
        av_layers.debug_print(state);
        let layers_ptr = av_layers.handle_logic(state);
        
        
        let mut create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions_ptr[..])
            .enabled_layer_names(&layers_ptr[..]);
        
        
        
        let mut debug_messenger;
        if constants::VALIDATION {
            debug_messenger = DMessenger::populate_create_info();
            create_info = create_info.push_next(&mut debug_messenger);
        }
        
        
        let instance_holder = unsafe{entry.create_instance(&create_info, None)?};
        
        
        Ok(Self{entry:entry, instance:instance_holder})
    }

}

impl Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Drop for Instance{
    fn drop(&mut self) {
        unsafe{self.destroy_instance(None)};
    }
}

struct Extensions(Vec<vk::ExtensionProperties>);

impl Extensions {
    
    fn gets(entry:&ash::Entry) -> Self {
        let extension_list = entry.enumerate_instance_extension_properties(None).unwrap();
        Self(
            extension_list
        )
    }
    
    fn debug_print(&self, state:&State) {
        
        if state.v_exp() {
            println!("Extensions:");
            for extension in &self.0 {
                /*TODO: there is a bug descrived by https://github.com/ash-rs/ash/issues/830#issue-2010032912 */
                // work arround requires discarting the last char of extension_name
                /*
                let name_len:usize = extension.extension_name.len();
                let u8slice = unsafe { &*(&extension.extension_name[..name_len-1] as *const [i8] as *const [u8]) };
                
                
                let name_holder = std::str::from_utf8(u8slice).expect("after workarround on last char all names should be utf8 valid code should work");
                */
                let name_holder = unsafe{CStr::from_ptr(extension.extension_name.as_ptr())}.to_string_lossy();
                println!("\t{}:\t{:?}", extension.spec_version, name_holder);
                
            }
        }
        
    }
    
    fn validate(&self, window:&Window) -> Result<Vec<*const c_char>, AAError> {
        
        let mut extensions:Vec<&'static str> = constants::EXTENSIONS.into_iter().collect();
        let window_extensions = window.get_required_instance_extentions();
        extensions.extend(&window_extensions[..]);
        let mut required_extensions = extensions.len();
        
        /*
        for extension in &self.0 {
            
            let name_holder = unsafe{CStr::from_ptr(extension.extension_name.as_ptr())};
            let mut index = 0;
            
            while index<extensions.len() {
                
                if &name_holder.to_string_lossy() == extensions[index] {
                    extensions.remove(index);
                    required_extensions -= 1;
                    holder.push(extension.extension_name.as_ptr() as *const c_char);
                } 
                index += 1;
            }
        }
        */
        //let mut set:HashSet<&'static str> = HashSet::new();//(extensions);
        
        let mut holder = Vec::<*const c_char>::with_capacity(required_extensions);
        let mut set:HashSet<&'static str> = HashSet::from(constants::EXTENSIONS);//(extensions);
        set.extend(&window_extensions[..]);
        
        for extension in &self.0 {
            let name_holder = unsafe{CStr::from_ptr(extension.extension_name.as_ptr())}.to_string_lossy();
            if set.remove(&name_holder as &str) {
                holder.push(extension.extension_name.as_ptr() as *const c_char);
            }
        }
        
        
        if set.is_empty() {
            Ok(holder)
        } else {
            Err(AAError::MissingExtensions(set))
        }
    }
    
    fn handle_logic(&self, state:&State, window:&Window) -> Vec<*const c_char> {
        if constants::VALIDATION {
            match (state.v_exp(), self.validate(window)) {
                (true, Ok(holder)) => {
                    println!("All extensions layers found");
                    holder
                }
                (false, Ok(holder)) => {holder}
                (_, Err(err)) => {panic!("Extensions required by window and validation extensions should be available: {:?}", err);}
            }
        } else {
            Vec::new()
        }
    }
}

struct Layers(Vec<vk::LayerProperties>);

impl Layers {
    
    fn gets(entry:&ash::Entry) -> Self {
        let av_layers = entry.enumerate_instance_layer_properties().unwrap();
        Self(av_layers)
    }
    
    fn debug_print(&self, state:&State) {
        if state.v_exp() {
            println!("Layers:");
            for layer in &self.0 {
                /*
                let u8slice = unsafe { &*(&layer.layer_name as *const [i8] as *const [u8]) };
                let name_holder = std::str::from_utf8(u8slice).expect("all strings should be UTF8 valid");
                */
                let name_holder = unsafe{CStr::from_ptr(layer.layer_name.as_ptr())};
                println!("\t{:?}", name_holder);
            }
        }
    }
    
    fn validate(&self) -> Result<Vec<*const c_char>, AAError> {
        
        let mut validation_layers:Vec<&'static str> = constants::LAYERS.into_iter().collect();
        let mut required_layers = validation_layers.len();
        
        let mut holder = Vec::<*const c_char>::new();
        for layer in &self.0 {
            /*
            let u8slice = unsafe { &*(&layer.layer_name as *const [i8] as *const [u8]) };
            let name_holder = std::str::from_utf8(u8slice).expect("all strings should be UTF8 valid");
            //let name_holder = CStr::from_bytes_until_nul(layer.layer_name).expect("all strings should be UTF8 valid");
            //CStr::from_bytes_until_nul(layer.layer_name).expect("all strings should be UTF8 valid");
            */
            let name_holder = unsafe{CStr::from_ptr(layer.layer_name.as_ptr())};
            
            let mut index = 0;
            while index <validation_layers.len() {
                
                if &name_holder.to_string_lossy() == validation_layers[index] {
                    validation_layers.remove(index);
                    required_layers -= 1;
                    holder.push(layer.layer_name.as_ptr() as *const c_char);
                } 
                index += 1;
            }
        }
        
        if required_layers != 0 {
            Err(AAError::MissingLayers(validation_layers))
        } else {
            Ok(holder)
        }
    }
    
    fn handle_logic(&self, state:&State) -> Vec<*const c_char> {
        if constants::VALIDATION {
            match (state.v_exp(), self.validate()) {
                (true, Ok(holder)) => {
                    println!("All validation layers found");
                    holder
                }
                (false, Ok(holder)) => {holder}
                (_, Err(err)) => {panic!("all hard coded validation layers should be available: {:?}", err);}
            }
        } else {
            Vec::new()
        }
    }
}
