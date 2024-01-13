use crate::constants::LOGGING;
use ash::vk;

pub mod instance {
    use super::*;
    
    pub fn destructor() {
        if LOGGING {
            log::trace!("[0]:deleting instance");
        }
    }
}

pub mod d_messenger {
    use super::*;
    
    pub fn destructor() {
        if LOGGING {
            log::trace!("[0]:deleting debug messenger");
        }
    }
}

pub mod surface {
    use super::*;
    
    pub fn destructor() {
        if LOGGING {
            log::trace!("[0]:deleting surface");
        }
    }
}

pub mod device {
    use super::*;
    
    pub fn destructor() {
        if LOGGING {
            log::trace!("[0]:deleting device");
        }
    }
}

pub mod memory {
    use super::*;
    
    pub fn allocator_creation() {
        if LOGGING {
            log::trace!("\nCREATING:\tALLOCATOR");
        }
    }
    
    pub fn allocation_gpu_only(name:&str) {
        if LOGGING {
            log::trace!("allocating gpu memory for :\t{}", name);
        }
    }
}


pub mod image {
    use super::*;
    pub fn creation(name:Option<&'static str>) {
        if LOGGING {
            match name {
                Some(d_name) => {
                    println!("\nCREATING:\tIMAGE\nType: \t{}",d_name);
                }
                None => {
                    println!("\nCREATING:\tIMAGE ");
                }
            }
        }
    }
}


pub mod swapchain {
    use super::*;
    
    pub fn creation() {
        if LOGGING {
            log::trace!("\nCREATING:\tSWAPCHAIN");
        }
    }
    
    pub fn deletion(state:bool) {
        if LOGGING {
            if state {
                log::trace!("[0]:deleting images");
            } else {
                log::trace!("[0]:deleting swapchain");
            }
        }
    }
    
    pub fn format_chossing(surface_formats: &[vk::SurfaceFormatKHR]) {
        if LOGGING {
            log::trace!("{:#?}", surface_formats);
        }
    }
    
    pub fn found_format(found: bool, format: vk::SurfaceFormatKHR) {
        if LOGGING {
            if found {
                log::trace!("found target {:#?}", format);
            } else {
                log::trace!("didn't found target settling for {:#?}", format);
            }
        }
    }
    
    pub fn present_chossing(present: &[vk::PresentModeKHR]) {
        if LOGGING {
            log::trace!("{:#?}", present);
        }
    }
    
    pub fn found_present(found: bool) {
        if LOGGING {
            if found {
                log::trace!("found target Mailbox");
            } else {
                log::trace!("MAILBOX not available settling for FIFO");
            }
        }
    }
    
    pub fn sc_image_view_creations(index: usize) {
        if LOGGING {
            log::trace!("creating swapchain image {index}");
        }
    }
    
    pub fn transitioning_sc_image(old: vk::ImageLayout, new: vk::ImageLayout) {
        if LOGGING {
            log::info!("transitioning swapchain image old:{:?} new:{:?}", old, new);
        }
    }
    
    

    pub fn extent_chossing(extent: vk::Extent2D) {
        if LOGGING {
            println!("normal display width:{} height:{}", extent.width, extent.height);
        }
    }
    
}
