mod vk_win;

use crate::logger;
use crate::errors::messages::SIMPLE_SDL_FN;
use crate::imgui::Imgui;
use crate::player::Movement;

use std::mem::ManuallyDrop;


#[allow(dead_code)]
pub struct Window {
    sdl: ManuallyDrop<sdl2::Sdl>,
    video_subsys: sdl2::VideoSubsystem,
    window: sdl2::video::Window,
    event_pump: sdl2::EventPump,
    should_quit: bool,
}


impl Window {
    pub fn event_pump(&self) -> &sdl2::EventPump {
        &self.event_pump
    }
    
    pub fn underlying(&self) -> &sdl2::video::Window {
        &self.window
    }
    
    pub fn init() -> Window {
        
        logger::create!("window");
        
        let sdl = sdl2::init().expect(SIMPLE_SDL_FN);
        let mut video_subsys = sdl.video().expect(SIMPLE_SDL_FN);
        let window = Self::create_vulkan_builder(&mut video_subsys).unwrap();
        let event_pump = sdl.event_pump().expect(SIMPLE_SDL_FN);
        
        Self{
            sdl: ManuallyDrop::new(sdl),
            video_subsys,
            window,
            event_pump,
            should_quit: false,
        }
    }
    
    pub fn should_close(&self) -> bool {
        self.should_quit
    }
    
    pub fn poll_events(&mut self, imgui:&mut Imgui) -> Movement {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        
        
        for event in self.event_pump.poll_iter() {
            imgui.platform.handle_event(&mut imgui.context, &event);
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
*/
impl Drop for Window {
    fn drop(&mut self) {
        logger::destruct!("Window");
        unsafe{ManuallyDrop::drop(&mut self.sdl)};
        
    }
}
