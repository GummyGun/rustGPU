mod vk_win;

use super::{
    constants,
    State,
    Verbosity,
    player::{
        Movement,
    },
};

use std::ops::Deref;

//#[derive(Debug)]
pub struct Window {
    sdl: sdl2::Sdl,
    video_subsys: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    event_pump: sdl2::EventPump,
    should_quit: bool,
    /*
    pub state: State,
    pub window: glfw::PWindow,
    pub events_queue: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub main: glfw::Glfw,
    */
}

impl Window {
    pub fn event_pump(&self) -> &sdl2::EventPump {
        &self.event_pump
    }
    
    pub fn underlying(&self) -> &sdl2::video::Window {
        &self.window
    }
    
    pub fn init(state: State) -> Window {
        if let Verbosity::Expresive | Verbosity::Normal = state.verbosity {
            println!("[0]:window");
        }
        
        use sdl2::pixels::Color;
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        use std::time::Duration;
        
        let sdl = sdl2::init().unwrap();
        let mut video_subsys = sdl.video().unwrap();
        let window = Self::create_vulkan_builder(&mut video_subsys).unwrap();
        let event_pump = sdl.event_pump().unwrap();
        
        Self{
            sdl,
            video_subsys,
            window,
            event_pump,
            should_quit: false,
        }
    }
    
    pub fn should_close(&self) -> bool {
        self.should_quit
        //panic!();
    }
    
    pub fn poll_events(&mut self) -> Movement {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown{keycode: Some(Keycode::Escape),.. } => {
                    self.should_quit = true;
                }//break running,
                Event::KeyDown{keycode: Some(Keycode::Q),.. } => {
                    println!("Q");
                    self.should_quit = true;
                }
                _ => {}
            }
        }
        Movement::Up
    }
    
    pub fn get_required_instance_extentions(&self) -> Vec<&'static str> {
        
        /*
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
        */
        self.window.vulkan_instance_extensions().unwrap()
    }
    
    pub fn get_pixel_dimensions(&self) -> (i32,i32) {
        panic!("{:?}", self.window.size());
        panic!();
    }
}

/*
impl Deref for Window {
    type Target = glfw::PWindow;
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
*/

/*
impl Drop for Window {
    fn drop(&mut self) {
        if let Verbosity::Expresive | Verbosity::Normal = self.state.verbosity {
            print!("[0]:deleting Window\n")
        }
    }
}
*/
