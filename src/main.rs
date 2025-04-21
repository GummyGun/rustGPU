mod window;
mod vulkan;
mod gui;
mod errors;
mod logger;
mod constants;
mod utility;
mod graphics; 
mod player;
mod macros;
mod game;
pub use errors::Error as AAError;

use std::time::SystemTime;
use std::mem::ManuallyDrop;

#[allow(dead_code)]
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

struct HolderStruct {
    window: ManuallyDrop<window::Window>,
    v_init: ManuallyDrop<vulkan::VInit>,
    gui: ManuallyDrop<gui::Gui>,
    game: ManuallyDrop<game::Game>,
}


fn main() {
    
    let _state = State::init();
    
    //run("res/gltf/basicmesh.glb").expect("runtime error");
    
    let mut window = window::Window::init();
    let mut v_init = vulkan::VInit::init(&mut window);
    let gui = gui::Gui::init(&mut window, &mut v_init);
    let game = game::Game::init();
    
    let mut holder_struct = HolderStruct::new(window, v_init, gui, game);
    let HolderStruct{
        window,
        v_init,
        gui,
        game,
    } = &mut holder_struct;
    
    println!("=====================================================================================================================================================================\n=====================================================================================================================================================================");
    while !window.should_close() {
        window.poll_events(gui);
        gui.handle_events(window);
        
        v_init.handle_events(window);
        
        let (static_metadata, transformations, modifiable_metadata) = v_init.get_gui_data();
        gui.draw_ui(window, static_metadata, transformations, modifiable_metadata);
        
        v_init.gui_tick(gui.get_ui_data());
        
        game.step(window);
        
        v_init.draw_frame(gui);
        
    }
    println!("=====================================================================================================================================================================\n=====================================================================================================================================================================");
    v_init.wait_idle();
}


impl HolderStruct {
    fn new(window:window::Window, v_init:vulkan::VInit, gui:gui::Gui, game:game::Game) -> Self {
        HolderStruct{
            window: ManuallyDrop::new(window),
            v_init: ManuallyDrop::new(v_init),
            gui: ManuallyDrop::new(gui),
            game: ManuallyDrop::new(game),
        }
    }
}

impl Drop for HolderStruct {
    fn drop(&mut self) {
        unsafe{ManuallyDrop::drop(&mut self.game)};
        unsafe{ManuallyDrop::drop(&mut self.gui)};
        unsafe{ManuallyDrop::drop(&mut self.v_init)};
        unsafe{ManuallyDrop::drop(&mut self.window)};
    }
}



#[allow(dead_code)]
impl State {
    
    fn init() -> Self {
        env_logger::init();
        State{time:SystemTime::now(), verbosity:Verbosity::default()}
    }
    
/*
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
*/
    
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}


