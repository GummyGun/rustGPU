use crate::AAError;
use crate::macros;
use crate::constants;
use crate::errors::messages::SIMPLE_VK_FN;
use crate::errors::messages::GRANTED;
use crate::window::Window;

use crate::logger;
use super::VkDestructor;
use super::VkDestructorArguments;
use super::d_messenger::DMessenger;

use std::ffi::c_char;
use std::ffi::CStr;
use std::collections::HashSet;

use ash::vk;

pub struct Instance {
    pub entry: ash::Entry,
    instance: ash::Instance,
}
macros::impl_deref!(Instance, ash::Instance, instance);

impl Instance {
    
    pub fn create(window:&Window) -> Result<Instance, AAError> {
        
        logger::create!("instance");
        let entry = unsafe {ash::Entry::load().expect(SIMPLE_VK_FN)};
        
        match entry.try_enumerate_instance_version()? {
            // Vulkan 1.1+
            Some(version) => {
                #[allow(deprecated)]
                let major = vk::version_major(version);
                #[allow(deprecated)]
                let minor = vk::version_minor(version);
                #[allow(deprecated)]
                let patch = vk::version_patch(version);
                logger::various_log!("instance", 
                    (logger::Warn, "supported version is: {}.{}.{}", major, minor, patch),
                );
            },
            None => {
                panic!("only version 1.0 is suported");
            },
        }
        
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CStr::from_bytes_with_nul(b"Hello Triangle\0").expect(GRANTED))
            .application_name(CStr::from_bytes_with_nul(b"AdAstra\0").expect(GRANTED))
            .application_version(vk::make_api_version(1, 0, 0, 0))
            .engine_version(vk::make_api_version(1, 0, 0, 0))
            .api_version(vk::API_VERSION_1_3);
        
        let av_extensions = Extensions::get(&entry);
        av_extensions.log();
        let extensions_ptr = av_extensions.handle_logic(window);
        
        
        
        let av_layers = Layers::get(&entry);
        av_layers.log();
        let layers_ptr = av_layers.handle_logic();
        
        
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
    
    pub fn underlying(&self) -> ash::Instance {
        self.instance.clone()
    }
    
}

impl VkDestructor for Instance {
    fn destruct(self, mut args:VkDestructorArguments) {
        logger::destruct!("instance");
        args.unwrap_none();
        unsafe{self.destroy_instance(None)};
    }
    
}



struct Extensions(Vec<vk::ExtensionProperties>);

impl Extensions {
    
    fn get(entry:&ash::Entry) -> Self {
        let extension_list = entry.enumerate_instance_extension_properties(None).expect(SIMPLE_VK_FN);
        Self(
            extension_list
        )
    }
    
    fn log(&self) {
        
        logger::various_log!("instance", 
            (logger::Trace, "Extensions:"),
        );
        for extension in &self.0 {
            /*TODO: there is a bug descrived by https://github.com/ash-rs/ash/issues/830#issue-2010032912 */
            // work arround requires discarting the last char of extension_name
            /*
            let name_len:usize = extension.extension_name.len();
            let u8slice = unsafe { &*(&extension.extension_name[..name_len-1] as *const [i8] as *const [u8]) };
            
            
            let name_holder = std::str::from_utf8(u8slice).expect("after workarround on last char all names should be utf8 valid code should work");
            */
            let name_holder = unsafe{CStr::from_ptr(extension.extension_name.as_ptr())}.to_string_lossy();
            logger::various_log!("instance", 
                (logger::Trace, "\t{}:\t{:?}", extension.spec_version, name_holder),
            );
        }
        
    }
    
    fn validate(&self, window:&Window) -> Result<Vec<*const c_char>, AAError> {
        let window_extensions = window.get_required_instance_extentions();
        let mut set:HashSet<&'static str> = HashSet::from(constants::EXTENSIONS);//(extensions);
        set.extend(&window_extensions[..]);
        let mut holder = Vec::<*const c_char>::with_capacity(set.len());
        
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
    
    fn handle_logic(&self, window:&Window) -> Vec<*const c_char> {
        match self.validate(window) {
            Ok(holder) => {
                logger::various_log!("instance", 
                    (logger::Trace, "All extensions available"),
                );
                holder
            }
            Err(err) => {panic!("Extensions required by window and validation extensions should be available: {:?}", err);}
        }
    }
}


struct Layers(Vec<vk::LayerProperties>);

impl Layers {
    
    fn get(entry:&ash::Entry) -> Self {
        let av_layers = entry.enumerate_instance_layer_properties().expect(SIMPLE_VK_FN);
        Self(av_layers)
    }
    
    fn log(&self) {
        logger::various_log!("instance", 
            (logger::Trace, "Layers:"),
        );
        for layer in &self.0 {
            let name_holder = unsafe{CStr::from_ptr(layer.layer_name.as_ptr())};
            logger::various_log!("instance", 
                (logger::Trace, "\t\t{:?}", name_holder),
            );
        }
    }
    
    fn validate(&self) -> Result<Vec<*const c_char>, AAError> {
        let mut set:HashSet<&'static str> = constants::LAYERS.into_iter().collect();
        
        let mut holder = Vec::<*const c_char>::with_capacity(set.len());
        for layer in &self.0 {
            let name_holder = unsafe{CStr::from_ptr(layer.layer_name.as_ptr())}.to_string_lossy();
            if set.remove(&name_holder as &str) {
                holder.push(layer.layer_name.as_ptr() as *const c_char);
            }
        }
        if set.is_empty() {
            Ok(holder)
        } else {
            Err(AAError::MissingLayers(set))
        }
    }
    
    fn handle_logic(&self) -> Vec<*const c_char> {
        if constants::VALIDATION {
            match self.validate() {
                Ok(holder) => {
                    logger::various_log!("instance", 
                        (logger::Trace, "All layers available"),
                    );
                    holder
                }
                Err(err) => {panic!("all hard coded validation layers should be available: {:?}", err);}
            }
        } else {
            Vec::new()
        }
    }
}
