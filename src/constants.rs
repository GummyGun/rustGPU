macro_rules! const_array{
    ($($s:expr),*) => {
        {
            [$(
                cstr_to_str($s)
            ),+]
        }
    }
}


use std::ffi::CStr;

use ash::extensions::khr::Swapchain;
use ash::extensions::ext::DebugUtils;

const FIF:usize = 2;
pub mod fif {
    pub const USIZE:usize = super::FIF as usize;
    pub const U32:u32 = super::FIF as u32;
}

pub const WIDTH:u32 = 600;
pub const HEIGTH:u32 = 600;
pub const VALIDATION:bool = true;


pub const LAYERS:[&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];
pub const EXTENSIONS:[&'static str; EXTENSIONS_LEN_PLUS_VAL] = extension_logic();

pub const DEVICE_EXTENSIONS_CSTR:[&'static CStr; 1] = [Swapchain::name()];
pub const DEVICE_EXTENSIONS:[&'static str; 1] = const_array!(Swapchain::name());


const BASE_EXTENSIONS:[&'static str; 0] = [];
const DEBUG_EXTENSIONS:[&'static str; 1] = const_array!(DebugUtils::name());

const EXTENSIONS_LEN_PLUS_VAL:usize = extencion_len_logic();

const fn extencion_len_logic() -> usize {
    if VALIDATION {
        BASE_EXTENSIONS.len()+DEBUG_EXTENSIONS.len()
    } else {
        BASE_EXTENSIONS.len() 
    }
}

const fn cstr_to_str(value:&'static CStr) -> &'static str {
    let extension:&str = match value.to_str() {
        Ok(data) => {data}
        Err(_) => {panic!("bad const")},
    };
    extension
}

const fn extension_logic() -> [&'static str; EXTENSIONS_LEN_PLUS_VAL] {
    
    let mut holder = [""; EXTENSIONS_LEN_PLUS_VAL];
    let mut elem = 0;
    let base_extension_len = BASE_EXTENSIONS.len();
    while elem<base_extension_len {
        holder[elem] = BASE_EXTENSIONS[elem];
        elem+=1;
    }
    if VALIDATION {
        elem=0;
        while elem<DEBUG_EXTENSIONS.len() {
            holder[base_extension_len+elem] = DEBUG_EXTENSIONS[elem];
            elem+=1;
        }
    }
    holder
}


