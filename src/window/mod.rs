use super::{
    constants,
    State,
    Verbosity,
};

#[derive(Debug)]
pub struct Window {
    state: State,
    window: glfw::PWindow,
}

impl Window {
    pub fn init(state: State) -> Window {
        if let Verbosity::Expresive | Verbosity::Normal = state.verbosity {
            println!("creating Window");
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
    
    pub fn get_required_instance_extentions(&self) -> Vec<String> {
        self.window.glfw.get_required_instance_extensions().unwrap()
        
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if let Verbosity::Expresive | Verbosity::Normal = self.state.verbosity {
            print!("deleting Window\n")
        }
    }
}
