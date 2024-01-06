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

pub const WIDTH:u32 = 1200;
pub const HEIGTH:u32 = 800;
pub const VALIDATION:bool = true;

#[allow(dead_code)]
pub mod path {
    pub const VERT_SHADER:&str = "res/shaders/sh.vert.spv";
    pub const FRAG_SHADER:&str = "res/shaders/sh.frag.spv";
    pub const TEST_TEXTURE:&str = "res/textures/white.jpg";
    
    
    pub mod viking {
        use crate::graphics::FileType;
        pub const TYPE:FileType = FileType::Obj;
        pub const MODEL:&str = "res/objs/viking.obj";
        pub const TEXTURE:&str = "res/textures/viking.png";
        
        pub const fn metadata() -> (&'static str, &'static str, FileType) {
            (MODEL, TEXTURE, TYPE)
        }
        
    }
    pub mod cube {
        use crate::graphics::FileType;
        pub const TYPE:FileType = FileType::Obj;
        pub const MODEL:&str = "res/objs/cube.obj";
        pub const TEXTURE:&str = "res/textures/cube.png";
        pub const fn metadata() -> (&'static str, &'static str, FileType) {
            (MODEL, TEXTURE, TYPE)
        }
    }
    
    pub mod simple_box{
        use crate::graphics::FileType;
        use crate::graphics::SizeTransformation;
        //use nalgebra::RealField;
        
        pub const TYPE:FileType = FileType::Gltf;
        pub const MODEL:&str = "res/gltf/box/Box.gltf";
        pub const TEXTURE:&str = "res/textures/cube.png";
        
        pub const ROTATIONS_TRANSFORMATION:Option<((f32,f32,f32), f32)> = Some(((1f32, 0f32, 0f32), 1.5707964f32));
        //pub const ROTATIONS_TRANSFORMATION:Option<((f32,f32,f32), f32)> = Some(((1f32, 0f32, 0f32), RealField::frac_pi_2()));
        pub const SIZE_TRANSFORMATION:Option<(SizeTransformation, f32)> = Some((SizeTransformation::Enlarge, 16f32));
        pub const TRANSLATION_TRANSFORMATION:Option<(f32, f32, f32)> = Some((0f32, 0f32, 0f32));
        
        pub const fn metadata() -> (&'static str, &'static str, FileType) {
            (MODEL, TEXTURE, TYPE)
        }
        pub const fn load_transformations() -> (Option<((f32,f32,f32), f32)>, Option<(f32,f32,f32)>, Option<(SizeTransformation, f32)>) {
            (ROTATIONS_TRANSFORMATION, TRANSLATION_TRANSFORMATION, SIZE_TRANSFORMATION)
        }
    }
    
    pub mod suzanne{
        use crate::graphics::FileType;
        use crate::graphics::SizeTransformation;
        
        pub const TYPE:FileType = FileType::Gltf;
        pub const MODEL:&str = "res/gltf/suzanne/Suzanne.gltf";
        pub const TEXTURE:&str = "res/gltf/suzanne/Suzanne_BaseColor.png";
        
        pub const ROTATIONS_TRANSFORMATION:Option<((f32,f32,f32), f32)> = Some(((1f32, 0f32, 0f32), std::f32::consts::FRAC_PI_2));
        pub const SIZE_TRANSFORMATION:Option<(SizeTransformation, f32)> = None;
        pub const TRANSLATION_TRANSFORMATION:Option<(f32, f32, f32)> = None;
        
        pub const fn metadata() -> (&'static str, &'static str, FileType) {
            (MODEL, TEXTURE, TYPE)
        }
        
        pub const fn load_transformations() -> (Option<((f32,f32,f32), f32)>, Option<(f32,f32,f32)>, Option<(SizeTransformation, f32)>) {
            (ROTATIONS_TRANSFORMATION, TRANSLATION_TRANSFORMATION, SIZE_TRANSFORMATION)
        }
    }
    
    pub mod fox{
        use crate::graphics::FileType;
        pub const TYPE:FileType = FileType::Gltf;
        pub const MODEL:&str = "res/gltf/fox/Fox.gltf";
        pub const TEXTURE:&str = "res/gltf/fox/Texture.png";
        pub const fn metadata() -> (&'static str, &'static str, FileType) {
            (MODEL, TEXTURE, TYPE)
        }
    }
    
    pub mod avocado{
        use crate::graphics::FileType;
        use crate::graphics::SizeTransformation;
        
        pub const TYPE:FileType = FileType::Gltf;
        pub const MODEL:&str = "res/gltf/avocado/Avocado.gltf";
        pub const TEXTURE:&str = "res/gltf/avocado/Avocado_baseColor.png";
        
        pub const ROTATIONS_TRANSFORMATION:Option<((f32,f32,f32), f32)> = Some(((1f32, 0f32, 0f32), std::f32::consts::PI/2f32));
        //pub const ROTATIONS_TRANSFORMATION:Option<((f32,f32,f32), f32)> = Some(((1f32, 0f32, 0f32), RealField::frac_pi_2()));
        pub const SIZE_TRANSFORMATION:Option<(SizeTransformation, f32)> = Some((SizeTransformation::Enlarge, 16f32));
        pub const TRANSLATION_TRANSFORMATION:Option<(f32, f32, f32)> = Some((0f32, 0f32, 0f32));
        
        pub const fn metadata() -> (&'static str, &'static str, FileType) {
            (MODEL, TEXTURE, TYPE)
        }
        
        pub const fn load_transformations() -> (Option<((f32,f32,f32), f32)>, Option<(f32,f32,f32)>, Option<(SizeTransformation, f32)>) {
            (ROTATIONS_TRANSFORMATION, TRANSLATION_TRANSFORMATION, SIZE_TRANSFORMATION)
        }
        
    }
}

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


