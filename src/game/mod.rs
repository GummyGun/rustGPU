use nalgebra as na;
use crate::window::Window;
use crate::logger;

#[derive(Default)]
pub struct Game{
    player: Player,
}

#[derive(Default)]
pub struct Player{
    postion: na::Vector3<f32>,
}



impl Drop for Game {
    fn drop(&mut self) {
        logger::various_log!("Game",
            (logger::Trace, "dropping game")
        );
    }
}

impl Game {
    pub fn init() -> Self {
        Game::default()
    }
    pub fn step(&mut self, window: &mut Window) {
        println!("advancing logic");
        logger::various_log!("Game",
            (logger::Trace, "advancing game logic")
        );
    }
}











