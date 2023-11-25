mod window;
mod vk_bindings;
mod errors;

mod constants{
    pub const WIDTH:u32 = 600;
    pub const HEIGTH:u32 = 600;
    pub const VALIDATION:bool = true;
    
    pub const LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

}

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
}

fn main() {
    let state = State::default();
    
    let mut window = window::Window::init(state);
    let _v_init = vk_bindings::VInit::init(state, &window);
    while !window.should_close() {
        window.poll_events();
        break;
    }
}



