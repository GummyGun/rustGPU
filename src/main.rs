mod window;
mod vk_bindings;
mod errors;
mod constants;
mod utility;
mod graphics;

#[derive(Default, Debug, Clone, Copy)]
pub struct State {
    verbosity: Verbosity
}

#[allow(dead_code)]
#[derive(Default, Debug, Clone, Copy)]
enum Verbosity {
    Silent,
    #[default]
    Normal,
    Expresive,
    Dump,
}

fn main() {
    let state = State::default();
    
    let mut window = window::Window::init(state);
    let mut v_init = vk_bindings::VInit::init(state, &window);
    println!("===========\n===========");
    //v_init.test();
    while !window.should_close() {
        window.poll_events();
        v_init.draw_frame();
    }
    v_init.wait_idle();
}


impl State {
    
    /*
    fn v_all(&self) -> bool {
        true
    }
    */
    
    fn v_nor(&self) -> bool {
        match self.verbosity {
            Verbosity::Silent => false,
            _ => true
        }
    }
    
    fn v_exp(&self) -> bool {
        match self.verbosity {
            Verbosity::Silent | Verbosity::Normal => false,
            _ => true
        }
    }
    
    fn v_dmp(&self) -> bool {
        match self.verbosity {
            Verbosity::Dump => true,
            _ => false,
        }
    }
    
    /*
    fn v_exp_only(&self) -> bool {
        if let Verbosity::Expresive = self.verbosity {
            true
        } else {
            false
        }
    }
    */
    
}

