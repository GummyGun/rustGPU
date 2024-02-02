use crate::constants::LOGGING;
use ash::vk;

/*
mod base 
mod instance 
mod d_messenger 
mod surface 
mod device 
mod memory 
mod swapchain 
mod image 
*/

#[macro_export]
macro_rules! tmp {
    ($a:literal, $b:expr $(, $r:literal, $s:expr)* $(,)?) => {
        println!("---- hola");
        create!($($r-$s),*);
    };
    () => {
        println!("(((((((())))))))empty");
    };
}
pub(in super::super) use tmp;


#[macro_export]
macro_rules! create {
    ($target:literal) => {
        {
            use convert_case::{Case, Casing};
            log::log!(target:&$target.to_case(Case::Lower), log::Level::Trace, "CREATING {}", $target.to_case(Case::ScreamingSnake));
        }
    };
}
pub(in super::super) use create;


/*
#[macro_export]
macro_rules! __various_log {
    ($b:expr, $a:literal $(, $s:expr, $r:literal)* $(,)?) => {
        println!("hola {}", local);
        log::log!($b, $a);
        __various_log!($(, $r, $s)*);
    };
    () => {
        println!("(((((((())))))))empty");
    };
}
pub(in super::super) use __various_log;
*/

pub(in super::super) use log::Level;
#[macro_export]
macro_rules! various_log {
    ($target:expr, $b:expr, $a:literal $(, $s:expr, $r:literal)* $(,)?) => {
        {
            use convert_case::{Case, Casing};
            log::log!(target:&$target.to_case(Case::Lower), $b, $a);
            various_log!($target $(, $s, $r)*);
        }
    };
    ($target:expr) => {
    };
}
pub use various_log;


    
pub mod base {
    use super::*;
    
    pub fn create(name:&str) {
        if LOGGING {
            log::info!("[0]:{}", name);
        }
    }
    
    pub fn debug_messenger_create() {
        if LOGGING {
            log::info!("[0]:messenger");
        }
    }
    
    pub fn no_debug_messenger_create() {
        if LOGGING {
            log::info!("[X]:messenger");
        }
    }
    
    pub fn debug_messenger_destruct() {
        if LOGGING {
            log::info!("No Messenger to delete");
        }
    }
}


pub mod d_messenger {
    use super::*;
    
    pub fn create() {
        if LOGGING {
            log::info!("\nCREATING:\tDEBUG_MESSENGER\nvalidation layers activated");
        }
    }
    
    pub fn destruct() {
        if LOGGING {
            log::info!("[0]:deleting debug messenger");
        }
    }
}

pub mod surface {
    use super::*;
    
    pub fn create() {
        if LOGGING {
            log::info!("\nCREATING:\tSURFACE");
        }
    }
    
    pub fn destruct() {
        if LOGGING {
            log::info!("[0]:deleting surface");
        }
    }
}

pub mod device {
    use super::*;
    
    pub fn destruct() {
        if LOGGING {
            log::info!("[0]:deleting device");
        }
    }
}

pub mod memory {
    use super::*;
    
    pub mod alloc {
        use super::*;
        
        pub fn create() {
            if LOGGING {
                log::info!("\nCREATING:\tALLOCATOR");
            }
        }
        
        pub fn gpu_allocation(name:&str) {
            if LOGGING {
                log::info!("allocating gpu memory for :\t{}", name);
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::info!("[0]:deleting allocator");
            }
        }
    }
    
}


pub mod swapchain {
    use super::*;
    
    pub fn create() {
        if LOGGING {
            log::info!("\nCREATING:\tSWAPCHAIN");
        }
    }
    
    pub fn destruct(state:bool) {
        if LOGGING {
            if state {
                log::info!("[0]:deleting images");
            } else {
                log::info!("[0]:deleting swapchain");
            }
        }
    }
    
    pub fn format_chossing(surface_formats: &[vk::SurfaceFormatKHR]) {
        if LOGGING {
            log::info!("{:#?}", surface_formats);
        }
    }
    
    pub fn found_format(found: bool, format: vk::SurfaceFormatKHR) {
        if LOGGING {
            if found {
                log::info!("found target {:#?}", format);
            } else {
                log::info!("didn't found target settling for {:#?}", format);
            }
        }
    }
    
    pub fn present_chossing(present: &[vk::PresentModeKHR]) {
        if LOGGING {
            log::info!("{:#?}", present);
        }
    }
    
    pub fn found_present(found: bool) {
        if LOGGING {
            if found {
                log::info!("found target Mailbox");
            } else {
                log::info!("MAILBOX not available settling for FIFO");
            }
        }
    }
    
    pub fn sc_image_view_creates(index: usize) {
        if LOGGING {
            log::info!("creating swapchain image {index}");
        }
    }
    
    pub fn extent_chossing(extent: vk::Extent2D) {
        if LOGGING {
            log::info!("normal display width:{} height:{}", extent.width, extent.height);
        }
    }
    
}

pub mod image {
    use super::*;
    pub fn create(name:Option<&'static str>) {
        if LOGGING {
            match name {
                Some(d_name) => {
                    log::info!("\nCREATING:\tIMAGE\nType: \t{}",d_name);
                }
                None => {
                    log::info!("\nCREATING:\tIMAGE");
                }
            }
        }
    }
    
    
    pub fn transitioning_image(old: vk::ImageLayout, new: vk::ImageLayout) {
        if LOGGING {
            log::trace!("transitioning image from old:{:?} to new:{:?}", old, new);
        }
    }
    
    pub fn destruct() {
        if LOGGING {
            log::info!("[0]:deleting image");
        }
    }
    
}

pub mod sync_objs {
    use super::*;
    
    pub fn destruct() {
        if LOGGING {
            log::info!("[0]:deleting semaphores");
            log::info!("[0]:deleting fence");
        }
    }
}

pub mod command {
    use super::*;
    
    pub fn destruct() {
        if LOGGING {
            log::info!("[0]:deallocating command buffer");
            log::info!("[0]:deleting command pool");
        }
    }
}

pub mod descriptors {
    use super::*;
    
    
    pub fn init() {
        if LOGGING {
            log::info!("\tiniting:\tdescriptor structs");
        }
    }
    
    pub mod dlb {
        use super::*;
        pub fn create() {
            if LOGGING {
                log::info!("\nCREATING:\tLAYOUT_BUILDER");
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::info!("[0]:deleting layout_builder");
            }
        }
    }
    
    pub mod dl {
        use super::*;
        pub fn create() {
            if LOGGING {
                log::info!("\nCREATING:\tDESCRIPTOR_LAYOUT");
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::info!("[0]:deleting descriptor_layout");
            }
        }
    }
    
    pub mod dpa {
        use super::*;
        pub fn create() {
            if LOGGING {
                log::info!("\nCREATING:\tDESCRIPTOR_POOL");
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::info!("[0]:deleting descriptor_pool");
            }
        }
    }
    
}


pub mod pipeline {
    use super::*;
    
    
    pub mod compute {
        use super::*;
        
        pub fn init() {
            if LOGGING {
                log::info!("\tiniting:\tdescriptor structs");
            }
        }
        
        pub fn create(state:bool) {
            if LOGGING {
                if state {
                    log::info!("\nCREATING:\tPIPELINE_LAYOUT");
                } else {
                    log::info!("\nCREATING:\tCOMPUTE_PIPELINE");
                }
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::info!("[0]:deleting compute_pipeline");
                log::info!("[0]:deleting pipeline_layout");
            }
        }
        
    }
    
    pub mod graphics {
        use super::*;
        
        /*
        pub fn init() {
            if LOGGING {
                log::info!("\tiniting:\tdescriptor structs");
            }
        }
        */
        
        pub fn creating_builder() {
            if LOGGING {
                log::info!("\nCREATING:\tGRAPHICS_PIPELINE_BUILDER");
            }
        }
        
        pub fn destruct() {
            if LOGGING {
                log::info!("[0]:deleting graphics_pipeline");
                log::info!("[0]:deleting pipeline_layout");
            }
        }
        
    }
}

/*
pub mod imgui {
    use super::*;
    
    pub fn create() {
        if LOGGING {
            log::info!("\nCREATING:\tIMGUI");
        }
    }
    
    pub fn destruct() {
        if LOGGING {
            log::info!("[0]:deleting imgui");
        }
    }

}
*/
