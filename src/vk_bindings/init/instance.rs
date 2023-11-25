use ash::{
    prelude::VkResult,
    vk,
};
use std::{
    ffi::{
        c_char,
        CStr,
    },
    ops::Deref,
};

use super::{
    d_messenger::DMessenger
};

use crate::{
    constants,
    State,
    Verbosity,
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
        if let Verbosity::Expresive = state.verbosity {
            println!("\nCREATING:\tINSTANCE");
        }
        let entry = unsafe {ash::Entry::load().unwrap()};
        
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CStr::from_bytes_with_nul(b"Hello Triangle\0").unwrap())
            .application_name(CStr::from_bytes_with_nul(b"AdAstra\0").unwrap())
            .application_version(vk::make_api_version(1, 0, 0, 0))
            .engine_version(vk::make_api_version(1, 0, 0, 0))
            .api_version(vk::API_VERSION_1_0);
        
        
        
        let mut window_extensions = window.get_required_instance_extentions();
        if constants::VALIDATION {
            window_extensions.push(ash::extensions::ext::DebugUtils::name().to_str().unwrap().to_owned());
        } 
        let av_extensions = Self::get_extensions(state, &entry);
        let extensions_ptr = Self::validate_extensions(state, &av_extensions, &mut window_extensions).expect("all extensions should be available");
        
        
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

    fn get_extensions(state:&State, entry: &ash::Entry) -> Vec<vk::ExtensionProperties> {
        let extension_list = entry.enumerate_instance_extension_properties(None).unwrap();
        
        if let Verbosity::Expresive = state.verbosity {
            println!("Extensions:");
            for extension in &extension_list {
                /*TODO: there is a bug descrived by https://github.com/ash-rs/ash/issues/830#issue-2010032912 */
                // work arround requires discarting the last char of extension_name
                let name_len:usize = extension.extension_name.len();
                let u8slice = unsafe { &*(&extension.extension_name[..name_len-1] as *const [i8] as *const [u8]) };
                
                
                let name_holder = std::str::from_utf8(u8slice).expect("after workarround on last char all names should be utf8 valid code should work");
                println!("\t{}:\t{}", extension.spec_version, name_holder);
                
            }
        }
        extension_list
    }
    
    fn validate_extensions(state:&State, av_extensions:&Vec<vk::ExtensionProperties>, window_extensions:&mut Vec<String>) -> Result<Vec<*const c_char>, AAError> {
        let mut required_extensions = window_extensions.len();
        
        let mut holder = Vec::<*const c_char>::new();
        for extension in av_extensions {
            let name_len:usize = extension.extension_name.len();
            let u8slice = unsafe { &*(&extension.extension_name[..name_len-1] as *const [i8] as *const [u8]) };
            
            let name_holder = std::str::from_utf8(u8slice).expect("all strings should be UTF8 valid");
            let mut index = 0;
            while index <window_extensions.len() {
                
                if &name_holder[..window_extensions[index].len()] == window_extensions[index] {
                    window_extensions.remove(index);
                    required_extensions -= 1;
                    holder.push(name_holder.as_bytes().as_ptr() as *const c_char);
                } 
                index += 1;
            }
        }
        if required_extensions != 0 {
            println!("[X] Missing {} extensions:", required_extensions);
            for layer in window_extensions {
                println!("\t{}", layer);
            }
            Err(AAError::MissingExtensions)
        } else {
            if let Verbosity::Expresive = state.verbosity {
                println!("All extensions found");
            }
            Ok(holder)
        }
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

struct Layers(Vec<vk::LayerProperties>);

impl Layers {
    
    fn gets(entry:&ash::Entry) -> Self/*Vec<vk::LayerProperties>*/ {
        let av_layers = entry.enumerate_instance_layer_properties().unwrap();
        Self(av_layers)
    }
    
    fn debug_print(&self, state:&State) {
        if let Verbosity::Expresive = state.verbosity {
            println!("Layers:");
            for layer in &self.0 {
                let u8slice = unsafe { &*(&layer.layer_name as *const [i8] as *const [u8]) };
                let name_holder = std::str::from_utf8(u8slice).expect("all strings should be UTF8 valid");
                println!("\t{}", name_holder);
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
            match (state.verbosity, self.validate()) {
                (Verbosity::Expresive, Ok(holder)) => {
                    println!("All validation layers found");
                    holder
                }
                (_, Ok(holder)) => {holder}
                (_, Err(err)) => {panic!("all hard coded validation layers should be available: {:?}", err);}
            }
        } else {
            Vec::new()
        }
    }
    
}
