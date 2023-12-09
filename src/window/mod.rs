use ash::vk;

use super::{
    constants,
    State,
    Verbosity,
    vk_bindings,
};

use std::{
    ops::Deref,
};

#[derive(Debug)]
pub struct Window {
    state: State,
    window: glfw::PWindow,
}

impl Window {
    pub fn init(state: State) -> Window {
        if let Verbosity::Expresive | Verbosity::Normal = state.verbosity {
            println!("[0]:window");
        }
        let mut glfw = glfw::init_no_callbacks().unwrap();
        glfw.window_hint(glfw::WindowHint::Resizable(false));
        glfw.window_hint(glfw::WindowHint::Resizable(false));
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (window, _other_thing_that_i_do_not_know_what_is) = glfw.create_window(constants::WIDTH, constants::HEIGTH, "ADASTRA", glfw::WindowMode::Windowed).unwrap();
        Window{
            state:state,
            window:window,
        }
    }
    
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }
    
    pub fn poll_events(&mut self) {
        self.window.glfw.poll_events()
    }
    
    pub fn get_required_instance_extentions(&self) -> Vec<&'static str> {
        
        use glfw::ffi;
        use std::ffi::c_uint;
        use std::slice;
        use std::ffi::CStr;
        
        let mut len: c_uint = 0;
        
        let raw_extensions = unsafe{ffi::glfwGetRequiredInstanceExtensions(&mut len as *mut c_uint)};
        
        if raw_extensions.is_null() {
            panic!("glfw should require Extensions");
        }
        unsafe{
            slice::from_raw_parts(raw_extensions, len as usize)
                .into_iter()
                .map(|extensions| CStr::from_ptr(extensions.clone()).to_str().unwrap())
                .collect()
        }
        
    }
    
    pub unsafe fn create_surface(&self, instance:&vk_bindings::Instance, allocator:Option<&vk::AllocationCallbacks>) -> ash::prelude::VkResult<vk::SurfaceKHR> {
        use ash::RawPtr;
        let mut holder:vk::SurfaceKHR = Default::default();
        let instance = instance.handle();
        let holder_ptr = &mut holder as *mut _;
        
        self.create_window_surface(instance, allocator.as_raw_ptr(), holder_ptr).result()?;
        Ok(holder)
    }
    
    pub fn get_pixel_dimensions(&self) -> (i32,i32) {
        self.get_framebuffer_size()
    }
}

impl Deref for Window {
    type Target = glfw::PWindow;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if let Verbosity::Expresive | Verbosity::Normal = self.state.verbosity {
            print!("[0]:deleting Window\n")
        }
    }
}
