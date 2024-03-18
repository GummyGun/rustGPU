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

/*
use std::boxed::Box;
use std::error::Error as StdError;
use std::{fs, io};

fn print_tree(node: &gltf::Node, depth: i32) {
    for _ in 0..(depth - 1) {
        print!("  ");
    }
    print!(" -");
    print!(" Node {}", node.index());
    #[cfg(feature = "names")]
    print!(" ({})", node.name().unwrap_or("<Unnamed>"));
    println!();

    for child in node.children() {
        print_tree(&child, depth + 1);
    }
}

fn run(path: &str) -> Result<(), Box<dyn StdError>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let gltf = gltf::Gltf::from_reader(reader)?;
    for scene in gltf.scenes() {
        print!("Scene {}", scene.index());
        #[cfg(feature = "names")]
        print!(" ({})", scene.name().unwrap_or("<Unnamed>"));
        println!();
        for node in scene.nodes() {
            print_tree(&node, 1);
        }
    }
    
    
    let meshes = gltf.meshes();
    println!("{}", meshes.len());
    for mesh in meshes {
        println!("{:?}", mesh.name());
        let primitives = mesh.primitives();
        println!("{}", &primitives.len());
        for primitive in primitives {
            let reader = primitive.reader(|primitive|{Some(&gltf.blob.as_ref().unwrap()[..])});
            
            let positions = reader.read_positions().unwrap();
            println!("vertex ammount{}", positions.len());
            for pos in positions {
                println!("{:?}", pos);
            }
            let normals = reader.read_normals().unwrap();
            println!("vertex ammount{}", normals.len());
            for pos in normals {
                println!("{:?}", pos);
            }
            let color = reader.read_colors(0);
            println!("color {:?}", color);
            for pos in reader.read_colors(1) {
                println!("{:?}", pos);
            }
        }
        break;
    }
    
    
    panic!();
    panic!("{:?}", gltf.blob);
    for mesh in gltf.meshes() {
       println!("Mesh #{}", mesh.index());
       for primitive in mesh.primitives() {
           let reader = primitive.reader();
            for a in reader.read_normals(|buffer| Some(&buffer[buffer.index()])) {
                println!("{:?}", a);
            }
           
           /*
           println!("- Primitive #{}", primitive.index());
           for (semantic, a) in primitive.attributes() {
               println!("-- {:?} {:?}", semantic, a);
           }
           */
       }
    }

    Ok(())
}
*/

fn main() {
    //let model = graphics::Model::load_gltf();
    
    //let mut perspective = nalgebra::Matrix4::new(0.952099, 0.000000, 0.000000, 0.000000, 0.000000, 1.428148, 0.000000, 0.000000, 0.000000, 0.000000, 1.000020, -1.000000, 0.000000, 0.000000, 0.200002, 0.000000);
    //println!("{:?}", perspective);
    //return;
    
    let _state = State::init();
    
    //run("res/gltf/basicmesh.glb").expect("runtime error");
    
    let mut window = window::Window::init();
    let mut v_init = vulkan::VInit::init(&mut window);
    let imgui = imgui::Imgui::init(&mut window, &mut v_init);
    
    let mut holder_struct = HolderStruct::new(window, v_init, imgui);
    let HolderStruct{
        window,
        v_init,
        imgui,
    } = &mut holder_struct;
    
    println!("=====================================================================================================================================================================\n=====================================================================================================================================================================");
    while !window.should_close() {
        window.poll_events(imgui);
        imgui.handle_events(window);
        
        v_init.handle_events(window);
        
        let (static_metadata, modifiable_metadata) = v_init.get_imgui_data();
        imgui.draw_ui(window, static_metadata, modifiable_metadata, |holder|{&holder.name});
        
        v_init.gui_tick(imgui.get_ui_data());
        
        v_init.draw_frame(imgui);
        
        /*
        let current_time = state.secs_from_start();
        //println!("{:?}", 1f32/(current_time-last_time));
        last_time = current_time;
        */
    }
    println!("=====================================================================================================================================================================\n=====================================================================================================================================================================");
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


