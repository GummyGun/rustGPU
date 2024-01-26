use crate::constants::LOGGING;



pub fn create() {
    if LOGGING {
        log::info!("\nCREATING:\tIMGUI");
    }
}

pub fn destruct() {
    if LOGGING {
        log::info!("[0]:deleting imgui");
    }
}

