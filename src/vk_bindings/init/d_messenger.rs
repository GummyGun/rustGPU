use ash::{
    vk,
    prelude::VkResult,
};

use std::{
    ffi::{
        CStr,
    },
    borrow::Cow,
    ops::Deref,
};

use super::{
    ActiveDrop,
    instance::Instance,
};

use crate::{
    State,
    Verbosity,
};


pub struct DMessenger {
    pub debug_utils: ash::extensions::ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DMessenger {
    
    pub fn create(state:&State, instance:&Instance) -> VkResult<Self> {
        if let Verbosity::Expresive = state.verbosity {
            println!("\nCREATING:\tDEBUG_MESSENGER\nvalidation layers activated");
        }
        
        let debug_utils = ash::extensions::ext::DebugUtils::new(&instance.entry, instance);
        let messenger = unsafe{debug_utils.create_debug_utils_messenger(&Self::populate_create_info(), None)?};
        
        Ok(Self{
            debug_utils:debug_utils,
            messenger:messenger,
        })
    }
    
    pub fn populate_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT  {
        *vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
        .pfn_user_callback(Some(Self::vulkan_debug_callback))
        
    }
    
    unsafe extern "system" 
    fn vulkan_debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
       
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;
        
        let message_id_name = if callback_data.p_message_id_name.is_null() {
            Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };
        
        let message = if callback_data.p_message.is_null() {
            Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };
        
        println!( "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n");
        vk::FALSE
    }
    
    #[inline(always)]
    fn drop_internal(&mut self) {
        unsafe{self.debug_utils.destroy_debug_utils_messenger(self.messenger, None)};
    }
    
}

impl ActiveDrop for DMessenger {
    fn active_drop(&mut self, state:&State) {
        if state.v_nor() {
            println!("[0]:deleting debugMessenger");
        }
        self.drop_internal()
    }
}

impl Drop for DMessenger {
    fn drop(&mut self) {
        self.drop_internal()
    }
}

impl Deref for DMessenger {
    type Target = vk::DebugUtilsMessengerEXT;
    
    fn deref(&self) -> &Self::Target {
        &self.messenger
    }
}
