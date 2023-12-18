mod window;
mod vk_bindings;
mod errors;
mod constants;
mod utility;
mod graphics;

use std::time::SystemTime;

#[derive(Debug, Clone, Copy)]
pub struct State {
    verbosity: Verbosity,
    time: SystemTime
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
    let state = State{time:SystemTime::now(), verbosity:Verbosity::default()};
    
    let mut window = window::Window::init(state);
    let mut v_init = vk_bindings::VInit::init(state, &window);
    
    //let mut last_time = state.secs_from_start();
    println!("===========\n===========");
    //v_init.test();
    while !window.should_close() {
        window.poll_events();
        v_init.tick();
        v_init.draw_frame();
        /*
        let current_time = state.secs_from_start();
        //println!("{:?}", 1f32/(current_time-last_time));
        last_time = current_time;
        */
    }
    println!("===========\n===========");
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
    
    fn secs_from_start(&self) -> f32 {
        self.time.elapsed().unwrap().as_secs_f32()
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

