mod window;
mod vulkan;
mod imgui;
mod errors;
mod logger;
mod constants;
mod utility;
mod graphics; 
mod player;
mod macros;
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
    imgui: ManuallyDrop<imgui::Imgui>,
}


fn main() {
    //let model = graphics::Model::load_gltf();
    
    let state = State::new();
    
    let mut window = window::Window::init();
    let mut v_init = vulkan::VInit::init(&mut window);
    let imgui = imgui::Imgui::init(&mut window, &mut v_init);
    
    let mut holder_struct = HolderStruct::new(window, v_init, imgui);
    let HolderStruct{
        window,
        v_init,
        imgui,
    } = &mut holder_struct;
    
    println!("===========\n===========");
    while !window.should_close() {
        window.poll_events(imgui);
        imgui.handle_events(window);
        
        imgui.draw_ui(window, v_init.get_compute_effects_metadata());
        v_init.gui_tick(imgui.get_ui_data());
        
        v_init.draw_frame(imgui);
        
        /*
        let current_time = state.secs_from_start();
        //println!("{:?}", 1f32/(current_time-last_time));
        last_time = current_time;
        */
    }
    println!("===========\n===========");
    v_init.wait_idle();
}


impl HolderStruct {
    fn new(window:window::Window, v_init:vulkan::VInit, imgui:imgui::Imgui) -> Self {
        HolderStruct{
            window: ManuallyDrop::new(window),
            v_init: ManuallyDrop::new(v_init),
            imgui: ManuallyDrop::new(imgui),
        }
        
    }
}

impl Drop for HolderStruct {
    fn drop(&mut self) {
        unsafe{ManuallyDrop::drop(&mut self.imgui)};
        unsafe{ManuallyDrop::drop(&mut self.v_init)};
        unsafe{ManuallyDrop::drop(&mut self.window)};
    }
}



#[allow(dead_code)]
impl State {
    
    fn new() -> Self {
        env_logger::init();
        State{time:SystemTime::now(), verbosity:Verbosity::default()}
    }
    
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
    
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}


