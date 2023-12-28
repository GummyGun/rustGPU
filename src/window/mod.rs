mod vk_win;

use super::{
    constants,
    State,
    Verbosity,
    player::{
        Movement,
    },
};

use std::{
    ops::Deref,
};

#[derive(Debug)]
pub struct Window {
    pub state: State,
    pub window: glfw::PWindow,
    pub events_queue: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    pub main: glfw::Glfw,
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
        let (mut window, _other_thing_that_i_do_not_know_what_is) = glfw.create_window(constants::WIDTH, constants::HEIGTH, "ADASTRA", glfw::WindowMode::Windowed).unwrap();
        window.set_key_polling(true);
        Window{
            state:state,
            window:window,
            events_queue:_other_thing_that_i_do_not_know_what_is,
            main:glfw,
        }
    }
    
    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }
    
    pub fn poll_events(&mut self) -> Movement {
        use glfw::{Key, flush_messages, Action, WindowEvent};
        
        self.window.glfw.poll_events();
        for (_, event) in flush_messages(&self.events_queue) {
            println!("{:?}", event);
            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                },
                WindowEvent::Key(Key::Q, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                WindowEvent::Key(Key::E, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                WindowEvent::Key(Key::D, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                WindowEvent::Key(Key::F, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                WindowEvent::Key(Key::S, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                WindowEvent::Key(Key::Space, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                WindowEvent::Key(Key::A, _, Action::Press, _) => {
                    self.window.set_should_close(true)
                }
                _ => {},
            }
        }
        Movement::Up

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
