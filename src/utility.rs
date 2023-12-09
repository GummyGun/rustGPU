
/*
use std::{
    fs::File,
    io::Error as IoError,
    mem,
};

fn read_file(file:&str) -> Result<Vec<u8>, IoError> {
    let file = File::open(file)?;
    panic!();
}

pub fn vec_transmute_u8_u32(mut vec8:Vec<u8>) -> Vec<u32> {
    if !check_alinment::<_,u32>(&vec8[0]) {
        panic!("vectors are granted to be aligned to 4 like most structs");
    }
    
    let ratio = 4;
    let ptr = vec8.as_mut_ptr() as *mut u32;
    
    let capacity = vec8.capacity() / ratio;
    let length = vec8.len() / ratio;
    
    mem::forget(&vec8);
    
    unsafe{Vec::from_raw_parts(ptr, length, capacity)}
}

pub fn check_alinment<T,U>(value:&T) -> bool {
    let alignment = mem::align_of::<U>();
    let ptr = std::ptr::addr_of!(value);
    let ptr_usize = ptr as usize;
    if (ptr_usize % alignment) == 0 {
        true 
    } else {
        false
    }
}
*/
