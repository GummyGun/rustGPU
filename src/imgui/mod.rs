#[cfg(feature = "vulkan")]
mod vk_imgui;
#[cfg(feature = "vulkan")]
pub use vk_imgui::Imgui;

use crate::window::Window;
use crate::graphics::ComputePushConstants;
use crate::graphics::ComputeEffectMetadata;


#[derive(Default, Debug)]
pub struct InputData {
    pub background_index: usize,
    pub push_constants: ComputePushConstants,
}

const PUSH_CONSTANT_MSG:[&str; 4] = [
    "push_constant 1",
    "push_constant 2",
    "push_constant 3",
    "push_constant 4",
];


impl Imgui{
    
    pub fn handle_events(
        &mut self,
        window: &Window,
    ) {
        let (context_holder, platform_holder, _ui_data) = self.get_common_mut();
        platform_holder.prepare_frame(context_holder.io_mut(), window.underlying(), &window.event_pump().mouse_state());
    }
    
    pub fn draw_ui(
        &mut self,
        window: &mut Window,
        compute_effect_metadata: &mut [ComputeEffectMetadata],
    ) {
        
        let (context, platform, ui_data) = self.get_common_mut();
        let ui = context.new_frame();
        {
            let _disabled_token = ui.begin_disabled(false);
            ui.text("Dangerous button");
            
            for (index, effect) in compute_effect_metadata.into_iter().enumerate() {
                ui.radio_button(effect.name, &mut ui_data.background_index, index);
            }
            
            ui_data.push_constants = compute_effect_metadata[ui_data.background_index].data.clone();
            for index in 0..4 {
                let _ = ui.input_float4(PUSH_CONSTANT_MSG[index], &mut ui_data.push_constants[index])
                    .build();
                
            }
            ui.text("Dangerous button");
            
        }
        
        platform.prepare_render(&ui, window.underlying());
    }
}

/*
impl InputData {
    pub fn initial_data() -> Self {
        InputData{
            background_index: 0,
            push_constants:ComputePushConstants([
            Vector4::new(3.1, 3.2, 0.4 ,0.97),
            Vector4::new(0.0, 0.0, 0.0 ,0.0),
            Vector4::new(0.0, 0.0, 0.0 ,0.0),
            Vector4::new(0.0, 0.0, 0.0 ,0.0),
            ]),
        }
    }
}
*/
